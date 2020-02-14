use crate::AsyncRead;

use _futures::ready;
use core::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};
use pin_project_lite::pin_project;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    #[cfg_attr(docsrs, doc(cfg(feature = "io-util")))]
    pub struct Chain<T, U> {
        #[pin]
        first: T,
        #[pin]
        second: U,
        done_first: bool,
    }
}

pub(super) fn chain<T, U>(first: T, second: U) -> Chain<T, U>
where
    T: AsyncRead,
    U: AsyncRead,
{
    Chain {
        first,
        second,
        done_first: false,
    }
}

impl<T, U> Chain<T, U>
where
    T: AsyncRead,
    U: AsyncRead,
{
    pub fn get_ref(&self) -> (&T, &U) {
        (&self.first, &self.second)
    }

    pub fn get_mut(&mut self) -> (&mut T, &mut U) {
        (&mut self.first, &mut self.second)
    }

    pub fn get_pin_mut(self: Pin<&mut Self>) -> (Pin<&mut T>, Pin<&mut U>) {
        let me = self.project();
        (me.first, me.second)
    }

    pub fn into_inner(self) -> (T, U) {
        (self.first, self.second)
    }
}

impl<T, U> fmt::Debug for Chain<T, U>
where
    T: fmt::Debug,
    U: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chain")
            .field("t", &self.first)
            .field("u", &self.second)
            .finish()
    }
}

#[derive(Debug)]
pub enum ChainError<T, U> {
    Left(T),
    Right(U),
}

impl<T, U> AsyncRead for Chain<T, U>
where
    T: AsyncRead,
    U: AsyncRead,
{
    type Error = ChainError<T::Error, U::Error>;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, ChainError<T::Error, U::Error>>> {
        let me = self.project();

        if !*me.done_first {
            match ready!(me.first.poll_read(cx, buf).map_err(ChainError::Left))? {
                0 if !buf.is_empty() => *me.done_first = true,
                n => return Poll::Ready(Ok(n)),
            }
        }
        me.second.poll_read(cx, buf).map_err(ChainError::Right)
    }
}
