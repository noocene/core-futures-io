use _futures::ready;
use bytes::Buf;
use core::ops::DerefMut;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

pub trait AsyncWrite {
    type WriteError;
    type FlushError;
    type CloseError;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::WriteError>>;

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::FlushError>>;

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::CloseError>>;

    fn poll_write_buf<B: Buf>(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut B,
    ) -> Poll<Result<usize, Self::WriteError>>
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

macro_rules! deref_async_write {
    () => {
        type WriteError = T::WriteError;
        type FlushError = T::FlushError;
        type CloseError = T::CloseError;

        fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8])
            -> Poll<Result<usize, Self::WriteError>>
        {
            Pin::new(&mut **self).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::FlushError>> {
            Pin::new(&mut **self).poll_flush(cx)
        }

        fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::CloseError>> {
            Pin::new(&mut **self).poll_close(cx)
        }
    }
}

#[cfg(feature = "alloc")]
mod boxed {
    use super::*;
    use alloc::boxed::Box;

    impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for Box<T> {
        deref_async_write!();
    }
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for &mut T {
    deref_async_write!();
}

impl<P> AsyncWrite for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncWrite,
{
    type WriteError = <P::Target as AsyncWrite>::WriteError;
    type FlushError = <P::Target as AsyncWrite>::FlushError;
    type CloseError = <P::Target as AsyncWrite>::CloseError;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::WriteError>> {
        self.get_mut().as_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::FlushError>> {
        self.get_mut().as_mut().poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::CloseError>> {
        self.get_mut().as_mut().poll_close(cx)
    }
}

#[cfg(all(feature = "alloc", not(any(feature = "tokio", feature = "futures"))))]
mod vec {
    use super::*;
    use alloc::vec::Vec;
    use void::Void;

    impl AsyncWrite for Vec<u8> {
        type WriteError = Void;
        type FlushError = Void;
        type CloseError = Void;

        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context,
            buf: &[u8],
        ) -> Poll<Result<usize, Self::WriteError>> {
            self.get_mut().extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut Context,
        ) -> Poll<Result<(), Self::FlushError>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _cx: &mut Context,
        ) -> Poll<Result<(), Self::CloseError>> {
            Poll::Ready(Ok(()))
        }
    }
}
