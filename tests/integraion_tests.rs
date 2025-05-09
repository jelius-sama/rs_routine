use rs_routine::{go, init};
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn smoke_test() {
    // Optionally initialize explicitly
    init();

    let (tx, rx) = mpsc::channel();
    for i in 0..10 {
        let tx = tx.clone();
        go(move || {
            tx.send(i).unwrap();
        });
    }

    std::thread::sleep(Duration::from_millis(100));

    let mut results: Vec<_> = rx.try_iter().collect();
    results.sort();
    assert_eq!(results, (0..10).collect::<Vec<_>>());
}
