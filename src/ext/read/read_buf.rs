use crate::AsyncRead;
use bytes::BufMut;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) fn read_buf<'a, R, B>(reader: &'a mut R, buf: &'a mut B) -> ReadBuf<'a, R, B>
where
    R: AsyncRead,
    B: BufMut,
{
    ReadBuf { reader, buf }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadBuf<'a, R, B> {
    reader: &'a mut R,
    buf: &'a mut B,
}

impl<R, B> Future for ReadBuf<'_, R, B>
where
    R: AsyncRead,
    B: BufMut,
{
    type Output = Result<usize, R::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<usize, R::Error>> {
        unsafe {
            let me = self.get_unchecked_mut();
            Pin::new_unchecked(&mut *me.reader).poll_read_buf(cx, &mut me.buf)
        }
    }
}
