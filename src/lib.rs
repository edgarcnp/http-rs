use std::{sync::{mpsc::{self, Receiver}, Arc, Mutex}, thread::{self, JoinHandle}};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

enum Message {
    NewJob(Job),
    Terminate,
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
        let receiver: Arc<Mutex<Receiver<Message>>> = Arc::new(Mutex::new(receiver));
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
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Terminate Instruction for all workers.");
        
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        
        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread: JoinHandle<()> = thread::spawn(move || loop { 
            let message = receiver
                .lock()
                .unwrap()
                .recv()
                .unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("New Job:\nWorker: {}", id);
                    job();
                }
                Message::Terminate => {
                    println!("Terminate Job:\nWorker: {}", id);
                    break;
                }
            }
         });

        Worker { id, thread: Some(thread) }
    }
}