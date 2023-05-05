#![allow(dead_code)]

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
    pub fn close(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(t) = worker.thread.take() {
                t.join().unwrap();
            }
        }
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("thread_pool release");
        self.close();
    }
}
struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let f = receiver.lock().unwrap().recv();
            match f {
                Ok(job) => {
                    println!("worker:{} doing job", id);
                    job();
                }
                Err(_) => {
                    println!("worker:{id} shutting down;");
                    break;
                }
            }
        });
        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test() {
        let pool = ThreadPool::new(2);
        pool.execute(|| {
            println!("task one");
            thread::sleep(Duration::from_secs(5));
        });
        pool.execute(|| {
            println!("task two");
            thread::sleep(Duration::from_secs(5));
        });
        pool.execute(|| {
            println!("task three");
            thread::sleep(Duration::from_secs(5));
        });
        thread::sleep(Duration::from_secs(1000000));
    }
}
