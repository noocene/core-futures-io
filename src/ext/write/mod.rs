use crate::AsyncWrite;
mod close;
mod flush;
mod write;
mod write_all;
mod write_buf;
mod write_int;
use bytes::Buf;
use close::close;
pub use close::Close;
use flush::flush;
pub use flush::Flush;
use write::write;
pub use write::Write;
use write_all::write_all;
pub use write_all::WriteAll;
use write_buf::write_buf;
pub use write_buf::WriteBuf;
pub use write_int::{WriteI128, WriteI16, WriteI32, WriteI64, WriteI8};
pub use write_int::{WriteU128, WriteU16, WriteU32, WriteU64, WriteU8};

macro_rules! write_impl {
    (
        $(
            $(#[$outer:meta])*
            fn $name:ident(&mut self, n: $ty:ty) -> $($fut:ident)*;
        )*
    ) => {
        $(
            $(#[$outer])*
            fn $name<'a>(&'a mut self, n: $ty) -> $($fut)*<&'a mut Self> where Self: Unpin {
                $($fut)*::new(self, n)
            }
        )*
    }
}

pub trait AsyncWriteExt: AsyncWrite {
    fn write<'a>(&'a mut self, src: &'a [u8]) -> Write<'a, Self>
    where
        Self: Unpin,
    {
        write(self, src)
    }

    fn write_buf<'a, B>(&'a mut self, src: &'a mut B) -> WriteBuf<'a, Self, B>
    where
        Self: Sized,
        B: Buf,
    {
        write_buf(self, src)
    }

    fn write_all<'a>(&'a mut self, src: &'a [u8]) -> WriteAll<'a, Self>
    where
        Self: Unpin,
    {
        write_all(self, src)
    }

    write_impl! {
        fn write_u8(&mut self, n: u8) -> WriteU8;
        fn write_i8(&mut self, n: i8) -> WriteI8;
        fn write_u16(&mut self, n: u16) -> WriteU16;
        fn write_i16(&mut self, n: i16) -> WriteI16;
        fn write_u32(&mut self, n: u32) -> WriteU32;
        fn write_i32(&mut self, n: i32) -> WriteI32;
        fn write_u64(&mut self, n: u64) -> WriteU64;
        fn write_i64(&mut self, n: i64) -> WriteI64;
        fn write_u128(&mut self, n: u128) -> WriteU128;
        fn write_i128(&mut self, n: i128) -> WriteI128;
    }

    fn flush(&mut self) -> Flush<'_, Self>
    where
        Self: Unpin,
    {
        flush(self)
    }

    fn close(&mut self) -> Close<'_, Self>
    where
        Self: Unpin,
    {
        close(self)
    }
}

impl<W: AsyncWrite + ?Sized> AsyncWriteExt for W {}
