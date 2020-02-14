use crate::AsyncWrite;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct Close<'a, A: ?Sized> {
    a: &'a mut A,
}

pub(super) fn close<A>(a: &mut A) -> Close<'_, A>
where
    A: AsyncWrite + Unpin + ?Sized,
{
    Close { a }
}

impl<A> Future for Close<'_, A>
where
    A: AsyncWrite + Unpin + ?Sized,
{
    type Output = Result<(), A::CloseError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let me = &mut *self;
        Pin::new(&mut *me.a).poll_close(cx)
    }
}
