use crate::AsyncWrite;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct Shutdown<'a, A: ?Sized> {
    a: &'a mut A,
}

pub(super) fn shutdown<A>(a: &mut A) -> Shutdown<'_, A>
where
    A: AsyncWrite + Unpin + ?Sized,
{
    Shutdown { a }
}

impl<A> Future for Shutdown<'_, A>
where
    A: AsyncWrite + Unpin + ?Sized,
{
    type Output = Result<(), A::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = &mut *self;
        Pin::new(&mut *me.a).poll_shutdown(cx)
    }
}
