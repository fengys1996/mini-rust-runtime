use futures::{task, Future};
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Instant,
};

pub struct Delay {
    // when to complete the delay
    pub when: Instant,
}

impl Future for Delay {
    type Output = ();

    // Q: why use Pin?
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> task::Poll<Self::Output> {
        let now = Instant::now();
        if now >= self.when {
            return Poll::Ready(());
        }

        let when = self.when;
        let wake = Arc::new(Mutex::new(cx.waker().clone()));
        std::thread::spawn(move || {
            std::thread::sleep(when - now);
            let waker = wake.lock().unwrap();
            waker.wake_by_ref();
        });

        Poll::Pending
    }
}
