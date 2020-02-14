use crate::AsyncWrite;

use bytes::Buf;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteBuf<'a, W, B> {
    writer: &'a mut W,
    buf: &'a mut B,
}

pub(crate) fn write_buf<'a, W, B>(writer: &'a mut W, buf: &'a mut B) -> WriteBuf<'a, W, B>
where
    W: AsyncWrite,
    B: Buf,
{
    WriteBuf { writer, buf }
}

impl<W, B> Future for WriteBuf<'_, W, B>
where
    W: AsyncWrite,
    B: Buf,
{
    type Output = Result<usize, W::WriteError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<usize, W::WriteError>> {
        unsafe {
            let me = self.get_unchecked_mut();
            Pin::new_unchecked(&mut *me.writer).poll_write_buf(cx, &mut me.buf)
        }
    }
}
