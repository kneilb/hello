use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
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

        ThreadPool { workers, sender }
    }

    /// Execute a function (or closure) in a thread from the pool.
    ///
    pub fn run<F>(&self, func: F)
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        // TODO!
        // thread::spawn(func);
        let job = Box::new(func);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // Mutex guard dropped immediately after this statement.
                // If we used while let, the lock would be held for the whole scope...!
                let job = receiver.lock().unwrap().recv().unwrap();

                println!("Worker {} got a Job - executing!", id);

                job.call_box();
            }
        });
        Worker { id, thread }
    }
}

// Workaround for weirdness in compiler, which was apparently solved in 1.35 (but is broken in 1.74!)
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

// Ideally this would "just work", and above weirdness wouldn't be needed.
// type Job = Box<dyn FnOnce() + Send + 'static>;
