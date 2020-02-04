use crate::{AsyncRead, AsyncWrite};
use _futures::io::{AsyncRead as FAsyncRead, AsyncWrite as FAsyncWrite, Error};
use core::{
    pin::Pin,
    task::{Context, Poll},
};

impl<T: FAsyncRead> AsyncRead for T {
    type Error = Error;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        FAsyncRead::poll_read(self, cx, buf)
    }
}

impl<T: FAsyncWrite> AsyncWrite for T {
    type Error = Error;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        FAsyncWrite::poll_write(self, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        FAsyncWrite::poll_flush(self, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        FAsyncWrite::poll_close(self, cx)
    }
}

pub struct Compat<T>(T);

impl<T: AsyncWrite> FAsyncWrite for Compat<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncWrite::poll_write(self, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_flush(self, cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_shutdown(self, cx)
    }
}

impl<T: AsyncRead> FAsyncRead for Compat<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncRead::poll_read(self, cx, buf)
    }
}
