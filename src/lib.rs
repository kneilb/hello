use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// num_threads is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if num_threads is 0.
    pub fn new(num_threads: usize) -> ThreadPool {
        assert!(num_threads > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num_threads);

        for id in 0..num_threads {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Execute a function (or closure) in a thread from the pool.
    ///
    pub fn run<F>(&self, func: F)
    where
        F: FnOnce() -> (),
        F: Send + 'static,
    {
        let job = Box::new(func);

        // I originally did this with if let, but the book uses as_ref, which makes sense!
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Closing sender...");

        // Close the sender; all receivers will get an error.
        drop(self.sender.take());

        println!("Waiting for all workers to exit...");

        for w in &mut self.workers {
            println!("Waiting for worker {}.", w.id);

            if let Some(thread) = w.thread.take() {
                thread.join().unwrap();
            }
        }

        println!("All workers deaded.");
    }
}

// A Worker thread
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Mutex guard dropped immediately after this statement.
            // If we used while let, the lock would be held for the whole scope...!
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Worker {id} got a Job - running!");
                    // (*job)() gives E0161 "the size of `dyn FnOnce() + Send` cannot be statically determined", but this "just works"
                    job();
                    println!("Worker {id} finished the Job!");
                }
                Err(_) => {
                    println!("Worker {id} disconnected - exiting!");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// A Job that we send to a Worker
type Job = Box<dyn FnOnce() + Send + 'static>;
