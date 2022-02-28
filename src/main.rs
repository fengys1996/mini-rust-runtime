use std::{
    pin::Pin,
    sync::{Arc, Mutex},
};

use crossbeam::channel;
use futures::Future;

fn main() {
    println!("hello mini-rust-runtime!")
}

struct MiniRust {
    scheduled: channel::Receiver<Arc<Task>>,

    sender: channel::Sender<Arc<Task>>,
}

impl MiniRust {
    fn new() -> MiniRust {
        let (sender, scheduled) = channel::unbounded();
        MiniRust { scheduled, sender }
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send,
    {
    }
}

struct Task {
    // todo 1. avoid the mutex by using unsafe code
    // todo 2. the box is also avoid
    // todo 3. use the better data structure
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,

    // when a task is notied, is is queued into this channel
    executor: channel::Sender<Arc<Task>>,
}

impl Task {
    fn spawn<F>(future: F, sender: &channel::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // Q: Why can not be written below?
        // let future = Mutex::new(Box::pin(future));
        // let task = Arc::new(Task {
        //     future,
        //     executor: sender.clone(),
        // });
        // 
        // A: Because future is not trait object, need convert to trait object
        let task = Arc::new(Task {
            // Q: Why need 'F: 'static'
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });
        let _ = sender.send(task);
    }
}
