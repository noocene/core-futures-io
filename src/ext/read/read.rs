use crate::AsyncRead;
use core::{
    future::Future,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) fn read<'a, R>(reader: &'a mut R, buf: &'a mut [u8]) -> Read<'a, R>
where
    R: AsyncRead + Unpin + ?Sized,
{
    Read { reader, buf }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Read<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut [u8],
}

impl<R> Future for Read<'_, R>
where
    R: AsyncRead + Unpin + ?Sized,
{
    type Output = Result<usize, R::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<usize, R::Error>> {
        let me = &mut *self;
        Pin::new(&mut *me.reader).poll_read(cx, me.buf)
    }
}
