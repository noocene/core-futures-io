use crate::{AsyncRead, AsyncWrite};
use _futures::io::{AsyncRead as FAsyncRead, AsyncWrite as FAsyncWrite, Error, ErrorKind};
use core::{
    pin::Pin,
    task::{Context, Poll},
};

impl<T: Unpin + FAsyncRead> AsyncRead for Compat<T> {
    type Error = Error;

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        FAsyncRead::poll_read(Pin::new(&mut self.0), cx, buf)
    }
}

impl<T: Unpin + FAsyncWrite> AsyncWrite for Compat<T> {
    type WriteError = Error;
    type FlushError = Error;
    type CloseError = Error;

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::WriteError>> {
        FAsyncWrite::poll_write(Pin::new(&mut self.0), cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::FlushError>> {
        FAsyncWrite::poll_flush(Pin::new(&mut self.0), cx)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::CloseError>> {
        FAsyncWrite::poll_close(Pin::new(&mut self.0), cx)
    }
}

pub struct Compat<T>(T);

impl<T> Compat<T> {
    pub fn new(input: T) -> Self {
        Compat(input)
    }
}

impl<T: Unpin + AsyncWrite> FAsyncWrite for Compat<T>
where
    T::WriteError: Into<Box<dyn std::error::Error + Sync + Send>>,
    T::FlushError: Into<Box<dyn std::error::Error + Sync + Send>>,
    T::CloseError: Into<Box<dyn std::error::Error + Sync + Send>>,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncWrite::poll_write(Pin::new(&mut self.0), cx, buf)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.0), cx)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_close(Pin::new(&mut self.0), cx)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

impl<T: Unpin + AsyncRead> FAsyncRead for Compat<T>
where
    T::Error: Into<Box<dyn std::error::Error + Sync + Send>>,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncRead::poll_read(Pin::new(&mut self.0), cx, buf)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
