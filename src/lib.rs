use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
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
        let job = Box::new(func);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending Terminate message to all workers.");

        // A worker will only ever read one Terminate message
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for w in &mut self.workers {
            println!("Shutting down worker {}.", w.id);

            if let Some(thread) = w.thread.take() {
                thread.join().unwrap();
            }
        }

        println!("All workers deaded.");
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // Mutex guard dropped immediately after this statement.
                // If we used while let, the lock would be held for the whole scope...!
                let msg = receiver.lock().unwrap().recv().unwrap();
                match msg {
                    Message::NewJob(job) => {
                        println!("Worker {} got a NewJob - running!", id);
                        job.call_box();
                        println!("Worker {} finished the Job!", id);
                    }
                    Message::Terminate => {
                        println!("Worker {} got Terminate - exiting!", id);
                        break;
                    }
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
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
