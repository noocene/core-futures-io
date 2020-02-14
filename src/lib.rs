#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

use _futures::ready;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use void::Void;

mod read;
pub use read::AsyncRead;
mod write;
pub use write::AsyncWrite;
mod ext;
pub use ext::*;

#[cfg(feature = "tokio")]
mod tokio;
#[cfg(feature = "tokio")]
pub use self::tokio::Compat as TokioCompat;

#[cfg(feature = "futures")]
mod futures;
#[cfg(feature = "futures")]
pub use self::futures::Compat as FuturesCompat;

#[derive(Debug)]
pub struct Empty;

impl AsyncRead for Empty {
    type Error = Void;

    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context,
        _: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        Poll::Ready(Ok(0))
    }
}

#[derive(Debug)]
pub struct Sink;

impl AsyncWrite for Sink {
    type WriteError = Void;
    type FlushError = Void;
    type CloseError = Void;

    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::WriteError>> {
        Poll::Ready(Ok(buf.len()))
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Self::FlushError>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Self::CloseError>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
pub struct Repeat(u8);

impl Repeat {
    pub fn new(item: u8) -> Self {
        Repeat(item)
    }
}

impl AsyncRead for Repeat {
    type Error = Void;

    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        for byte in &mut *buf {
            *byte = self.0;
        }
        Poll::Ready(Ok(buf.len()))
    }
}

#[cfg(not(feature = "alloc"))]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Copy<'a, R: ?Sized, W: ?Sized> {
    reader: &'a mut R,
    read_done: bool,
    writer: &'a mut W,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: [u8; 2048],
}

#[cfg(feature = "alloc")]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Copy<'a, R: ?Sized, W: ?Sized> {
    reader: &'a mut R,
    read_done: bool,
    writer: &'a mut W,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: alloc::boxed::Box<[u8]>,
}

#[cfg(feature = "alloc")]
pub fn copy<'a, R, W>(reader: &'a mut R, writer: &'a mut W) -> Copy<'a, R, W>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    Copy {
        reader,
        read_done: false,
        writer,
        amt: 0,
        pos: 0,
        cap: 0,
        buf: alloc::boxed::Box::new([0; 2048]),
    }
}

#[cfg(not(feature = "alloc"))]
pub fn copy<'a, R, W>(reader: &'a mut R, writer: &'a mut W) -> Copy<'a, R, W>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    Copy {
        reader,
        read_done: false,
        writer,
        amt: 0,
        pos: 0,
        cap: 0,
        buf: [0; 2048],
    }
}

#[derive(Debug)]
pub enum CopyError<R: AsyncRead + ?Sized, W: AsyncWrite + ?Sized> {
    Read(R::Error),
    Write(W::WriteError),
    Flush(W::FlushError),
    WriteZero,
}

impl<R, W> Future for Copy<'_, R, W>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    type Output = Result<u64, CopyError<R, W>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<u64, CopyError<R, W>>> {
        loop {
            if self.pos == self.cap && !self.read_done {
                let me = &mut *self;
                let n = ready!(Pin::new(&mut *me.reader).poll_read(cx, &mut me.buf))
                    .map_err(CopyError::Read)?;
                if n == 0 {
                    self.read_done = true;
                } else {
                    self.pos = 0;
                    self.cap = n;
                }
            }

            while self.pos < self.cap {
                let me = &mut *self;
                let i = ready!(Pin::new(&mut *me.writer).poll_write(cx, &me.buf[me.pos..me.cap]))
                    .map_err(CopyError::Write)?;
                if i == 0 {
                    return Poll::Ready(Err(CopyError::WriteZero));
                } else {
                    self.pos += i;
                    self.amt += i as u64;
                }
            }

            if self.pos == self.cap && self.read_done {
                let me = &mut *self;
                ready!(Pin::new(&mut *me.writer).poll_flush(cx)).map_err(CopyError::Flush)?;
                return Poll::Ready(Ok(self.amt));
            }
        }
    }
}

pub trait Split: AsyncRead + AsyncWrite {
    type ReadHalf: AsyncRead;
    type WriteHalf: AsyncWrite;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf);
}

pub struct Join<T: AsyncRead, U: AsyncWrite> {
    read: T,
    write: U,
}

impl<T: AsyncRead, U: AsyncWrite> Join<T, U> {
    pub fn new(read: T, write: U) -> Self {
        Join { read, write }
    }
}

impl<T: AsyncRead, U: AsyncWrite> AsyncRead for Join<T, U>
where
    T: Unpin,
    U: Unpin,
{
    type Error = T::Error;

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        Pin::new(&mut self.read).poll_read(cx, buf)
    }
}

impl<T: AsyncRead, U: AsyncWrite> AsyncWrite for Join<T, U>
where
    T: Unpin,
    U: Unpin,
{
    type WriteError = U::WriteError;
    type FlushError = U::FlushError;
    type CloseError = U::CloseError;

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::WriteError>> {
        Pin::new(&mut self.write).poll_write(cx, buf)
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::FlushError>> {
        Pin::new(&mut self.write).poll_flush(cx)
    }
    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<(), Self::CloseError>> {
        Pin::new(&mut self.write).poll_close(cx)
    }
}

impl<T: AsyncRead, U: AsyncWrite> Split for Join<T, U>
where
    T: Unpin,
    U: Unpin,
{
    type ReadHalf = T;
    type WriteHalf = U;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf) {
        (self.read, self.write)
    }
}
