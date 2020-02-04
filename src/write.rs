use _futures::ready;
use bytes::Buf;
#[cfg(not(any(feature = "tokio", feature = "futures")))]
use core::ops::DerefMut;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

pub trait AsyncWrite {
    type Error;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn poll_write_buf<B: Buf>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<Result<usize, Self::Error>>
    where
        Self: Sized,
    {
        if !buf.has_remaining() {
            return Poll::Ready(Ok(0));
        }

        let n = ready!(self.poll_write(cx, buf.bytes()))?;
        buf.advance(n);
        Poll::Ready(Ok(n))
    }
}

#[cfg(not(any(feature = "tokio", feature = "futures")))]
macro_rules! deref_async_write {
    () => {
        type Error = T::Error;

        fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8])
            -> Poll<Result<usize, Self::Error>>
        {
            Pin::new(&mut **self).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut **self).poll_flush(cx)
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut **self).poll_shutdown(cx)
        }
    }
}

#[cfg(all(feature = "alloc", not(any(feature = "tokio", feature = "futures"))))]
mod boxed {
    use super::*;
    use alloc::boxed::Box;

    impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for Box<T> {
        deref_async_write!();
    }
}

#[cfg(not(any(feature = "tokio", feature = "futures")))]
impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for &mut T {
    deref_async_write!();
}

#[cfg(not(any(feature = "tokio", feature = "futures")))]
impl<P> AsyncWrite for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncWrite,
{
    type Error = <P::Target as AsyncWrite>::Error;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        self.get_mut().as_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().as_mut().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().as_mut().poll_shutdown(cx)
    }
}

#[cfg(all(feature = "alloc", not(any(feature = "tokio", feature = "futures"))))]
mod vec {
    use super::*;
    use alloc::vec::Vec;
    use void::Void;

    impl AsyncWrite for Vec<u8> {
        type Error = Void;

        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, Self::Error>> {
            self.get_mut().extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }
}
