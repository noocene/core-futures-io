#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod read;
pub use read::AsyncRead;
mod write;
pub use write::AsyncWrite;
