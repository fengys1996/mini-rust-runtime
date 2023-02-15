mod arc_wake;
pub mod delay;

use crossbeam::channel;
use futures::{
    task::{self, ArcWake},
    Future,
};
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::Context,
};

pub struct MiniRust {
    scheduled: channel::Receiver<Arc<Task>>,

    sender: channel::Sender<Arc<Task>>,
}

impl MiniRust {
    pub fn new() -> MiniRust {
        let (sender, scheduled) = channel::unbounded();
        MiniRust { scheduled, sender }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    pub fn run(&self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}

pub struct Task {
    // todo 1. avoid the mutex by using unsafe code
    // todo 2. the box is also avoid
    // todo 3. use the better data structure
    // flag1: same with: future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
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
        let future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>> = Mutex::new(Box::pin(future));
        let task = Arc::new(Task {
            // Q: Why need 'F: 'static'
            // A: https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes
            // http://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch19-02-advanced-lifetimes.html#inference-of-trait-object-lifetimes
            // please see flag1
            future,
            executor: sender.clone(),
        });
        let _ = sender.send(task);
    }

    fn poll(self: Arc<Self>) {
        // Get a waker referencing the task
        let waker = task::waker(self.clone());

        // Initialize the task context with the waker
        let mut cx = Context::from_waker(&waker);

        // This will never block as only a single thread
        let mut future = self.future.try_lock().unwrap();

        // Poll the future
        let _ = future.as_mut().poll(&mut cx);
    }
}

// Q: how to work?
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Push task to executor channel.
        // The executor recvices from the executor channel and polls tasks
        let _ = arc_self.executor.send(arc_self.clone());
    }
}
