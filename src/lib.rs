use std::{sync::{mpsc::{self, Receiver}, Arc, Mutex}, thread::{self, JoinHandle}};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// Size: Number of thread(s) in the pool.
    /// 
    /// # Panic(s)
    /// 
    /// Size of Thread(s) cannot be less than 1.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver: Arc<Mutex<Receiver<Job>>> = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker> = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));

        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f:F) 
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Box<F> = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread: JoinHandle<()> = thread::spawn(move || loop { 
            let job = receiver
                .lock()
                .unwrap()
                .recv()
                .unwrap();

            job();
            println!("New Job:\nWorker: {}", id);            
         });

        Worker { id, thread }
    }
}