use std::thread::{spawn, JoinHandle};
use std::ops::Deref;

/// Schrödinger's threads.
///
/// Threads in ThreadAbstraction can be either be started or not started, via the `spawn_if`
/// method.
///
/// # Why?
///
/// This exists due to a dilemma Encore once had, it boils down to this:
///
/// 1. Spawn thread
/// 1. Thread (usually with a compile time cfg!) checks if said feature is enabled
/// 1. Worst case, thread immedately `return`s
/// 1. Thread ends up being spawned, resulting in an additonal syscall, and possibly increased
///    binary size
///
/// Instead, ThreadAbstraction prevents the thread from starting if a compile time condition says
/// so, but still allows regular ol `spawn`ing of threads.
///
/// # Disadvantages
///
/// It does not provide full `JoinThread<T>` parity. Only few methods are implemented.
pub struct ThreadAbstraction (Option<JoinHandle<()>>);

impl ThreadAbstraction {
    /// Spawn a thread.
    ///
    /// Feels exactly the same as `std::thread::spawn()`, but returns a ThreadAbstraction. It does
    /// not provide full `JoinThread<T>` parity.
    ///
    /// # Example
    ///
    /// ```rust
    /// spawn(move || { println!("this will run!") });
    /// ```
    pub fn spawn<F: std::marker::Send + FnOnce() + 'static>(f: F) -> ThreadAbstraction {
        ThreadAbstraction(Some(spawn(f)))
    }

    /// Spawns a thread if a condition is true.
    ///
    /// Almost the same as `std::thread::spawn()`, but you need to provide a bool. No thread will
    /// be started if such bool is `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// spawn_if(move || { println!("this will run!") }, true);
    /// ```
    ///
    /// ```rust
    /// spawn_if(move || { println!("this will not run!") }, false);
    /// ```
    ///
    /// ```rust
    /// spawn_if(move || { println!("this will only run if built in debug!"); }, cfg!(debug_assertions));
    /// ```
    pub fn spawn_if<F: std::marker::Send + FnOnce() + 'static>(f: F, condition: bool) -> ThreadAbstraction {
        if condition {
            ThreadAbstraction(Some(spawn(f)))
        } else {
            ThreadAbstraction(None)
        }
    }

    /// Wait for thread to finish
    ///
    /// If a thread is not started (see `spawn_if`), it will return immedately, otherwise, it will
    /// block the current thread otherwise. It works _very similarly_ to `JoinThread.join()`, but
    /// it will not return anything.
    ///
    /// # Panics
    ///
    /// In debug builds, if a thread panics, the value will be unwrapped. It will not panic in
    /// release builds
    pub fn join(self) {
        if self.0.is_some() {
            if cfg!(debug_assertions) {
                // in debug, crash if thread panicked
                self.0.unwrap().join().unwrap();
                return;
            }
            let _ = self.0.unwrap().join();
        }
    }
}

impl Deref for ThreadAbstraction {
    type Target = Option<JoinHandle<()>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
