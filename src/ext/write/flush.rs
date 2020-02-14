use crate::AsyncWrite;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct Flush<'a, A: ?Sized> {
    a: &'a mut A,
}

pub(super) fn flush<A>(a: &mut A) -> Flush<'_, A>
where
    A: AsyncWrite + Unpin + ?Sized,
{
    Flush { a }
}

impl<A> Future for Flush<'_, A>
where
    A: AsyncWrite + Unpin + ?Sized,
{
    type Output = Result<(), A::FlushError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let me = &mut *self;
        Pin::new(&mut *me.a).poll_flush(cx)
    }
}
