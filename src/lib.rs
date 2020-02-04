#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod read;
pub use read::AsyncRead;
mod write;
pub use write::AsyncWrite;
mod ext;
pub use ext::{AsyncReadExt, AsyncWriteExt};

#[cfg(feature = "tokio")]
mod tokio;
#[cfg(feature = "tokio")]
pub use self::tokio::Compat as TokioCompat;

#[cfg(feature = "futures")]
mod futures;
#[cfg(feature = "futures")]
pub use self::futures::Compat as FuturesCompat;
