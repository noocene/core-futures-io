use crate::AsyncWrite;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Write<'a, W: ?Sized> {
    writer: &'a mut W,
    buf: &'a [u8],
}

pub(crate) fn write<'a, W>(writer: &'a mut W, buf: &'a [u8]) -> Write<'a, W>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    Write { writer, buf }
}

impl<W> Future for Write<'_, W>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    type Output = Result<usize, W::WriteError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<usize, W::WriteError>> {
        let me = &mut *self;
        Pin::new(&mut *me.writer).poll_write(cx, me.buf)
    }
}
