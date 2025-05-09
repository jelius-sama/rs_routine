//! # gorust
//!
//! Lightweight goroutines for Rust, inspired by Go.
//!
//! ## Example
//!
//! ```rust
//! use rs_routine::{go, yield_now};
//!
//! fn main() {
//!     for i in 0..5 {
//!         go(move || {
//!             println!("Hello from goroutine {}", i);
//!             yield_now();
//!         });
//!     }
//!     // Allow background tasks to run
//!     std::thread::sleep(std::time::Duration::from_millis(100));
//! }
//! ```

mod scheduler;

/// Spawn a new lightweight goroutine executing the given closure.
/// Automatically initializes the runtime on first invocation.
///
/// The closure must be `'static + Send`.
pub fn go<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    scheduler::init_runtime();
    scheduler::spawn(Box::new(f));
}

/// Yields execution of the current goroutine, allowing others to run.
pub fn yield_now() {
    scheduler::yield_now();
}

/// Initialize the global runtime (optional).
/// Called automatically on first `go`, but can be called explicitly.
pub fn init() {
    scheduler::init_runtime();
}
