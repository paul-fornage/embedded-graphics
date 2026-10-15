//! Drives futures for the synchronous in-memory test and benchmark targets.

extern crate std;

use core::{
    future::Future,
    task::{Context, Poll, Waker},
};
use std::{sync::Arc, task::Wake};

struct Noop;

impl Wake for Noop {
    fn wake(self: Arc<Self>) {}
}

pub(crate) trait FutureExt: Future + Sized {
    /// In-memory drawing must finish on its first poll.
    fn ready(self) -> Self::Output {
        let waker = Waker::from(Arc::new(Noop));
        let mut future = core::pin::pin!(self);
        match future.as_mut().poll(&mut Context::from_waker(&waker)) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("in-memory drawing unexpectedly suspended"),
        }
    }
}

impl<F: Future> FutureExt for F {}
