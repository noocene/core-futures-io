use crate::AsyncRead;
use bytes::BufMut;
mod chain;
mod read;
mod read_buf;
mod read_exact;
mod read_int;
#[cfg(feature = "alloc")]
mod read_to_end;
#[cfg(feature = "alloc")]
mod read_to_string;
mod take;
#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};
use chain::chain;
pub use chain::Chain;
use read::read;
pub use read::Read;
use read_buf::read_buf;
pub use read_buf::ReadBuf;
use read_exact::read_exact;
pub use read_exact::ReadExact;
pub use read_int::{ReadI128, ReadI16, ReadI32, ReadI64, ReadI8};
pub use read_int::{ReadU128, ReadU16, ReadU32, ReadU64, ReadU8};
#[cfg(feature = "alloc")]
use read_to_end::read_to_end;
#[cfg(feature = "alloc")]
pub use read_to_end::ReadToEnd;
#[cfg(feature = "alloc")]
use read_to_string::read_to_string;
#[cfg(feature = "alloc")]
pub use read_to_string::ReadToString;
use take::take;
pub use take::Take;

macro_rules! read_impl {
    (
        $(
            $(#[$outer:meta])*
            fn $name:ident(&mut self) -> $($fut:ident)*;
        )*
    ) => {
        $(
            $(#[$outer])*
            fn $name<'a>(&'a mut self) -> $($fut)*<&'a mut Self> where Self: Unpin {
                $($fut)*::new(self)
            }
        )*
    }
}

pub trait AsyncReadExt: AsyncRead {
    fn chain<R>(self, next: R) -> Chain<Self, R>
    where
        Self: Sized,
        R: AsyncRead,
    {
        chain(self, next)
    }

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Read<'a, Self>
    where
        Self: Unpin,
    {
        read(self, buf)
    }

    fn read_buf<'a, B>(&'a mut self, buf: &'a mut B) -> ReadBuf<'a, Self, B>
    where
        Self: Sized,
        B: BufMut,
    {
        read_buf(self, buf)
    }

    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadExact<'a, Self>
    where
        Self: Unpin,
    {
        read_exact(self, buf)
    }

    read_impl! {
        fn read_u8(&mut self) -> ReadU8;
        fn read_i8(&mut self) -> ReadI8;
        fn read_u16(&mut self) -> ReadU16;
        fn read_i16(&mut self) -> ReadI16;
        fn read_u32(&mut self) -> ReadU32;
        fn read_i32(&mut self) -> ReadI32;
        fn read_u64(&mut self) -> ReadU64;
        fn read_i64(&mut self) -> ReadI64;
        fn read_u128(&mut self) -> ReadU128;
        fn read_i128(&mut self) -> ReadI128;
    }

    #[cfg(feature = "alloc")]
    fn read_to_end<'a>(&'a mut self, buf: &'a mut Vec<u8>) -> ReadToEnd<'a, Self>
    where
        Self: Unpin,
    {
        read_to_end(self, buf)
    }

    #[cfg(feature = "alloc")]
    fn read_to_string<'a>(&'a mut self, dst: &'a mut String) -> ReadToString<'a, Self>
    where
        Self: Unpin,
    {
        read_to_string(self, dst)
    }

    fn take(self, limit: u64) -> Take<Self>
    where
        Self: Sized,
    {
        take(self, limit)
    }
}

impl<R: AsyncRead + ?Sized> AsyncReadExt for R {}
