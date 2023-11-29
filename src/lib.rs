use std::thread;

pub struct ThreadPool;

impl ThreadPool {
    pub fn new(num_threads: u32) -> ThreadPool {
        ThreadPool {}
    }

    pub fn run<F, T>(&self, func: F)
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        // TODO!
        thread::spawn(func);
    }
}