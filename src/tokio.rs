use crate::{AsyncRead, AsyncWrite};
use _tokio::io::{AsyncRead as TAsyncRead, AsyncWrite as TAsyncWrite, Error};
use core::{
    pin::Pin,
    task::{Context, Poll},
};

impl<T: TAsyncRead> AsyncRead for T {
    type Error = Error;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        TAsyncRead::poll_read(self, cx, buf)
    }
}

impl<T: TAsyncWrite> AsyncWrite for T {
    type Error = Error;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        TAsyncWrite::poll_write(self, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        TAsyncWrite::poll_flush(self, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        TAsyncWrite::poll_shutdown(self, cx)
    }
}

pub struct Compat<T>(T);

impl<T: AsyncWrite> TAsyncWrite for Compat<T> {
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

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_shutdown(self, cx)
    }
}

impl<T: AsyncRead> TAsyncRead for Compat<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncRead::poll_read(self, cx, buf)
    }
}
