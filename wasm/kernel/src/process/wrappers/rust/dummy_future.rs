use std::pin::Pin;

use futures::Future;
use futures_task::{Context, Poll};

pub enum DummyFuture {
    Started,
    End,
}

impl Future for DummyFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = self.get_mut();
        match s {
            Self::Started => {
                *s = Self::End;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Self::End => Poll::Ready(()),
        }
    }
}
