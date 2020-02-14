use _futures::ready;
use bytes::BufMut;
use core::ops::DerefMut;
use core::{
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};
use genio::Read;

pub trait AsyncRead {
    type Error;

    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [MaybeUninit<u8>]) -> bool {
        for x in buf {
            *x.as_mut_ptr() = 0;
        }
        true
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_read_buf<B: BufMut>(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut B,
    ) -> Poll<Result<usize, Self::Error>>
    where
        Self: Sized,
    {
        if !buf.has_remaining_mut() {
            return Poll::Ready(Ok(0));
        }

        unsafe {
            let n = {
                let b = buf.bytes_mut();

                self.prepare_uninitialized_buffer(b);

                let b = &mut *(b as *mut [MaybeUninit<u8>] as *mut [u8]);

                let n = ready!(self.poll_read(cx, b))?;
                assert!(n <= b.len(), "Bad AsyncRead implementation, more bytes were reported as read than the buffer can hold");
                n
            };

            buf.advance_mut(n);
            Poll::Ready(Ok(n))
        }
    }
}

macro_rules! deref_async_read {
    () => {
        type Error = T::Error;

        unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [MaybeUninit<u8>]) -> bool {
            (**self).prepare_uninitialized_buffer(buf)
        }

        fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8])
            -> Poll<Result<usize, Self::Error>>
        {
            Pin::new(&mut **self).poll_read(cx, buf)
        }
    }
}

#[cfg(feature = "alloc")]
mod boxed {
    use super::*;
    use alloc::boxed::Box;

    impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for Box<T> {
        deref_async_read!();
    }
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for &mut T {
    deref_async_read!();
}

impl<P> AsyncRead for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncRead,
{
    type Error = <P::Target as AsyncRead>::Error;

    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [MaybeUninit<u8>]) -> bool {
        (**self).prepare_uninitialized_buffer(buf)
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        self.get_mut().as_mut().poll_read(cx, buf)
    }
}

impl AsyncRead for &[u8] {
    type Error = <Self as Read>::ReadError;

    unsafe fn prepare_uninitialized_buffer(&self, _buf: &mut [MaybeUninit<u8>]) -> bool {
        false
    }

    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        Poll::Ready(Read::read(self.get_mut(), buf))
    }
}
