use crate::Result;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub trait ThreadPool {
    ///Creates a new thread pool, immediately spawning the specified number of threads.
    ///
    /// Returns an error if any thread fails to spawn. All previously-spawned
    /// threads are terminated.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    /// Spawn a function into the threadpool.
    ///
    /// Spawning always succeeds, but if the function panics the threadpool continues to operate with the
    /// same number of threads — the thread count is not reduced nor is
    /// the thread pool destroyed, corrupted or invalidated.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;
type Receiver = Arc<Mutex<mpsc::Receiver<Message>>>;

#[derive(Clone)]
struct JobReceiver(Receiver);

impl Drop for JobReceiver {
    fn drop(&mut self) {
        if thread::panicking() {
            let rx = self.clone();

            if let Err(e) = thread::Builder::new().spawn(move || execute_job(rx)) {
                eprint!("Failed to spawn a thread: {}", e);
            }
        }
    }
}

fn execute_job(worker: JobReceiver) {
    loop {
        if let Ok(rx) = worker.0.lock() {
            if let Ok(msg) = rx.recv() {
                match msg {
                    Message::NewJob(job) => job(),
                    Message::Terminate => break,
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

pub struct SharedQueueThreadPool {
    size: u32,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(size: u32) -> Result<Self> {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel::<Message>();

        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..size as usize {
            let rx = receiver.clone();

            thread::Builder::new().spawn(move || execute_job(JobReceiver(rx)))?;
        }

        Ok(SharedQueueThreadPool { sender, size })
    }

    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .send(Message::NewJob(job))
            .expect("The thread pool has no thread.");
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.size {
            match self.sender.send(Message::Terminate) {
                Ok(_) => println!("Worker terminated!"),
                Err(e) => eprintln!("{}", e),
            }
        }
    }
}
