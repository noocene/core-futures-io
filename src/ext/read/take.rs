use crate::AsyncRead;

use _futures::ready;
use core::{
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "streams do nothing unless you `.await` or poll them"]
    #[cfg_attr(docsrs, doc(cfg(feature = "io-util")))]
    pub struct Take<R> {
        #[pin]
        inner: R,
        limit_: u64,
    }
}

pub(super) fn take<R: AsyncRead>(inner: R, limit: u64) -> Take<R> {
    Take {
        inner,
        limit_: limit,
    }
}

impl<R: AsyncRead> Take<R> {
    pub fn limit(&self) -> u64 {
        self.limit_
    }

    pub fn set_limit(&mut self, limit: u64) {
        self.limit_ = limit
    }

    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut R> {
        self.project().inner
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: AsyncRead> AsyncRead for Take<R> {
    type Error = R::Error;

    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [MaybeUninit<u8>]) -> bool {
        self.inner.prepare_uninitialized_buffer(buf)
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        if self.limit_ == 0 {
            return Poll::Ready(Ok(0));
        }

        let me = self.project();
        let max = core::cmp::min(buf.len() as u64, *me.limit_) as usize;
        let n = ready!(me.inner.poll_read(cx, &mut buf[..max]))?;
        *me.limit_ -= n as u64;
        Poll::Ready(Ok(n))
    }
}
