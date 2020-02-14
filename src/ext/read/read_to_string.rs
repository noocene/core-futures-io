use super::read_to_end::read_to_end_internal;
use crate::AsyncRead;
use _futures::ready;
use alloc::{
    str::{self, Utf8Error},
    string::String,
    vec::Vec,
};
use core::{
    future::Future,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadToString<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut String,
    bytes: Vec<u8>,
    start_len: usize,
}

#[derive(Debug)]
pub enum ReadToStringError<T> {
    Read(T),
    Utf8(Utf8Error),
}

pub(crate) fn read_to_string<'a, R>(reader: &'a mut R, buf: &'a mut String) -> ReadToString<'a, R>
where
    R: AsyncRead + ?Sized + Unpin,
{
    let start_len = buf.len();
    ReadToString {
        reader,
        bytes: unsafe { mem::replace(buf.as_mut_vec(), Vec::new()) },
        buf,
        start_len,
    }
}

fn read_to_string_internal<R: AsyncRead + ?Sized>(
    reader: Pin<&mut R>,
    cx: &mut Context,
    buf: &mut String,
    bytes: &mut Vec<u8>,
    start_len: usize,
) -> Poll<Result<usize, ReadToStringError<R::Error>>> {
    let ret =
        ready!(read_to_end_internal(reader, cx, bytes, start_len)).map_err(ReadToStringError::Read);
    if let Err(e) = str::from_utf8(&bytes) {
        Poll::Ready(ret.and_then(|_| Err(ReadToStringError::Utf8(e))))
    } else {
        debug_assert!(buf.is_empty());
        mem::swap(unsafe { buf.as_mut_vec() }, bytes);
        Poll::Ready(ret)
    }
}

impl<A> Future for ReadToString<'_, A>
where
    A: AsyncRead + ?Sized + Unpin,
{
    type Output = Result<usize, ReadToStringError<A::Error>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let Self {
            reader,
            buf,
            bytes,
            start_len,
        } = &mut *self;
        read_to_string_internal(Pin::new(reader), cx, buf, bytes, *start_len)
    }
}
