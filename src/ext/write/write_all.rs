use crate::AsyncWrite;
use _futures::ready;
use core::{
    future::Future,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteAll<'a, W: ?Sized> {
    writer: &'a mut W,
    buf: &'a [u8],
}

pub(crate) fn write_all<'a, W>(writer: &'a mut W, buf: &'a [u8]) -> WriteAll<'a, W>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    WriteAll { writer, buf }
}

#[derive(Debug)]
pub enum WriteAllError<T> {
    WriteZero,
    Write(T),
}

impl<T> From<T> for WriteAllError<T> {
    fn from(input: T) -> Self {
        WriteAllError::Write(input)
    }
}

impl<W> Future for WriteAll<'_, W>
where
    W: AsyncWrite + Unpin,
{
    type Output = Result<(), WriteAllError<W::WriteError>>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), WriteAllError<W::WriteError>>> {
        let me = &mut *self;
        while !me.buf.is_empty() {
            let n = ready!(Pin::new(&mut me.writer).poll_write(cx, me.buf))?;
            {
                let (_, rest) = mem::replace(&mut me.buf, &[]).split_at(n);
                me.buf = rest;
            }
            if n == 0 {
                return Poll::Ready(Err(WriteAllError::WriteZero));
            }
        }

        Poll::Ready(Ok(()))
    }
}
