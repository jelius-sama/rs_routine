use crossbeam_deque::{Injector, Steal, Stealer, Worker};
use once_cell::sync::OnceCell;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

/// A boxed task—`FnOnce()` wrapped for Send.
type Task = Box<dyn FnOnce() + Send + 'static>;

/// Global coroutine runtime state.
struct Runtime {
    injector: Arc<Injector<Task>>,
    notify: Arc<(Mutex<()>, Condvar)>,
}

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

/// Initialize the global runtime with one worker thread per core.
pub fn init_runtime() {
    RUNTIME.get_or_init(|| {
        let num_workers = num_cpus::get();
        // Specify generic for Injector to satisfy inference
        let injector = Arc::new(Injector::<Task>::new());
        let notify = Arc::new((Mutex::new(()), Condvar::new()));

        // Prepare worker-local deques and collect their stealers
        let mut workers: Vec<Worker<Task>> = Vec::with_capacity(num_workers);
        let mut stealers: Vec<Stealer<Task>> = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            let w: Worker<Task> = Worker::new_lifo();
            stealers.push(w.stealer());
            workers.push(w);
        }

        let stealers_shared = stealers.clone();
        let notify_shared = notify.clone();

        // Spawn OS threads running the scheduler loop
        for worker in workers {
            let injector = Arc::clone(&injector);
            let stealers = stealers_shared.clone();
            let notify = notify_shared.clone();
            thread::spawn(move || {
                loop {
                    // 1) Try local work
                    if let Some(task) = worker.pop() {
                        (task)();
                        continue;
                    }
                    // 2) Try injector
                    if let Steal::Success(task) = injector.steal() {
                        (task)();
                        continue;
                    }
                    // 3) Try other workers
                    let mut found = false;
                    for st in &stealers {
                        if let Steal::Success(task) = st.steal() {
                            (task)();
                            found = true;
                            break;
                        }
                    }
                    if found {
                        continue;
                    }
                    // 4) No work: park thread
                    let (lock, cvar) = &*notify;
                    let guard = lock.lock().unwrap();
                    // let _ = cvar.wait(guard).unwrap();
                    drop(cvar.wait(guard).unwrap());
                }
            });
        }

        Runtime { injector, notify }
    });
}

/// Spawn a task into the global runtime (after ensuring it's initialized).
pub fn spawn(task: Task) {
    let rt = RUNTIME.get().expect("Runtime should be initialized");
    rt.injector.push(task);
    // Wake one parked worker
    let (lock, cvar) = &*rt.notify;
    // let _ = lock.lock().unwrap();
    drop(lock.lock().unwrap());
    cvar.notify_one();
}

/// Yield execution cooperatively.
pub fn yield_now() {
    thread::yield_now();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_spawn_test() {
        init_runtime();
        let (tx, rx) = std::sync::mpsc::channel();
        spawn(Box::new(move || tx.send(99).unwrap()));
        let val = rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
        assert_eq!(val, 99);
    }
}
