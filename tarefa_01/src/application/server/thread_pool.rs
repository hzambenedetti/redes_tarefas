use std::{
    sync::{mpsc, Arc, Mutex}, thread,
};

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool{
    pub fn new(size: usize) -> ThreadPool{
       let mut workers = Vec::with_capacity(size);
        
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        //init threads
        for i in 0..size{
            workers.push(Worker::new(i, Arc::clone(&receiver))); 
        }
        ThreadPool{workers, sender}
    }

    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        match self.sender.send(job){
            Err(e) => {dbg!(e);},
            _ => {}
        };
    }
}

struct Worker{
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker{
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker{
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {id} got a job; executing");
                job();
            }
        });
        Worker{
            id,
            thread
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
