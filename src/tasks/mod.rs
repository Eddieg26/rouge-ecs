use std::{
    sync::mpsc::Sender,
    thread::{JoinHandle, ScopedJoinHandle},
};

pub mod barrier;

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, thread: JoinHandle<()>) -> Self {
        Self {
            id,
            thread: Some(thread),
        }
    }
}

pub struct TaskPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

type Job = Option<Box<dyn FnOnce() + Send + 'static>>;

impl TaskPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let receiver = receiver.clone();
            let thread = std::thread::spawn(move || loop {
                let job: Job = receiver.lock().unwrap().recv().unwrap();

                match job {
                    Some(job) => job(),
                    None => break,
                }
            });

            workers.push(Worker::new(id, thread));
        }

        Self { workers, sender }
    }

    pub fn execute(&self, f: impl FnOnce() + Send + 'static) {
        self.sender.send(Some(Box::new(f))).unwrap();
    }

    pub fn join(&mut self) {
        for worker in &mut self.workers {
            self.sender.send(None).unwrap();
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        self.join();
    }
}

pub struct ScopedWorker<'a> {
    id: usize,
    thread: Option<ScopedJoinHandle<'a, ()>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> ScopedWorker<'a> {
    fn new(id: usize, thread: ScopedJoinHandle<'a, ()>) -> Self {
        Self {
            id,
            thread: Some(thread),
            _marker: std::marker::PhantomData,
        }
    }
}

type ScopedJob<'a> = Option<Box<dyn FnOnce() + Send + 'a>>;

pub struct ScopedTaskPool<'a> {
    sender: Sender<ScopedJob<'a>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> ScopedTaskPool<'a> {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));

        std::thread::scope(|s| {
            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                let receiver = receiver.clone();
                let thread = s.spawn(move || loop {
                    let job: ScopedJob<'a> = receiver.lock().unwrap().recv().unwrap();

                    match job {
                        Some(job) => job(),
                        None => break,
                    }
                });

                workers.push(ScopedWorker::new(id, thread));
            }
        });

        Self {
            sender,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn execute(&self, f: impl FnOnce() + Send + 'a) {
        self.sender.send(Some(Box::new(f))).unwrap();
    }

    pub fn join(&mut self) {
        self.sender.send(None).unwrap();
    }
}

impl<'a> Drop for ScopedTaskPool<'a> {
    fn drop(&mut self) {
        self.join();
    }
}
