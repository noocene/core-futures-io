use crate::AsyncRead;

use _futures::ready;
use core::{
    future::Future,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) fn read_exact<'a, A>(reader: &'a mut A, buf: &'a mut [u8]) -> ReadExact<'a, A>
where
    A: AsyncRead + Unpin + ?Sized,
{
    ReadExact {
        reader,
        buf,
        pos: 0,
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadExact<'a, A: ?Sized> {
    reader: &'a mut A,
    buf: &'a mut [u8],
    pos: usize,
}

#[derive(Debug)]
pub enum ReadExactError<T> {
    Eof,
    Read(T),
}

impl<T> From<T> for ReadExactError<T> {
    fn from(input: T) -> Self {
        ReadExactError::Read(input)
    }
}

impl<A> Future for ReadExact<'_, A>
where
    A: AsyncRead + Unpin + ?Sized,
{
    type Output = Result<usize, ReadExactError<A::Error>>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<usize, ReadExactError<A::Error>>> {
        loop {
            if self.pos < self.buf.len() {
                let me = &mut *self;
                let n = ready!(Pin::new(&mut *me.reader).poll_read(cx, &mut me.buf[me.pos..]))?;
                me.pos += n;
                if n == 0 {
                    return Err(ReadExactError::Eof).into();
                }
            }

            if self.pos >= self.buf.len() {
                return Poll::Ready(Ok(self.pos));
            }
        }
    }
}
