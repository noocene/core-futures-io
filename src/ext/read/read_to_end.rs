use crate::AsyncRead;

use _futures::ready;
use alloc::vec::Vec;
use core::{
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[cfg_attr(docsrs, doc(cfg(feature = "io-util")))]
pub struct ReadToEnd<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut Vec<u8>,
    start_len: usize,
}

pub(crate) fn read_to_end<'a, R>(reader: &'a mut R, buf: &'a mut Vec<u8>) -> ReadToEnd<'a, R>
where
    R: AsyncRead + Unpin + ?Sized,
{
    let start_len = buf.len();
    ReadToEnd {
        reader,
        buf,
        start_len,
    }
}

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

pub(super) fn read_to_end_internal<R: AsyncRead + ?Sized>(
    mut rd: Pin<&mut R>,
    cx: &mut Context,
    buf: &mut Vec<u8>,
    start_len: usize,
) -> Poll<Result<usize, R::Error>> {
    let mut g = Guard {
        len: buf.len(),
        buf,
    };
    let ret;
    loop {
        if g.len == g.buf.len() {
            unsafe {
                g.buf.reserve(32);
                let capacity = g.buf.capacity();
                g.buf.set_len(capacity);

                let b = &mut *(&mut g.buf[g.len..] as *mut [u8] as *mut [MaybeUninit<u8>]);

                rd.prepare_uninitialized_buffer(b);
            }
        }

        match ready!(rd.as_mut().poll_read(cx, &mut g.buf[g.len..])) {
            Ok(0) => {
                ret = Poll::Ready(Ok(g.len - start_len));
                break;
            }
            Ok(n) => g.len += n,
            Err(e) => {
                ret = Poll::Ready(Err(e));
                break;
            }
        }
    }

    ret
}

impl<A> Future for ReadToEnd<'_, A>
where
    A: AsyncRead + ?Sized + Unpin,
{
    type Output = Result<usize, A::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = &mut *self;
        read_to_end_internal(Pin::new(&mut this.reader), cx, this.buf, this.start_len)
    }
}
