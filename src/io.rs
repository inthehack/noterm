#![allow(async_fn_in_trait)]

use core::fmt::Debug;

pub mod blocking;

/// Error.
#[derive(Clone, Debug, thiserror::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Unknown error.
    #[error("unknown error")]
    Unknown,
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Reader trait.
pub trait Read {
    async fn read(&mut self, data: &mut [u8]) -> Result<usize>;

    async fn read_all(&mut self, mut data: &mut [u8]) -> Result<()> {
        while !data.is_empty() {
            let count = self.read(data).await?;
            data = &mut data[count..];
        }
        Ok(())
    }
}

impl<ReaderTy: Read> Read for &mut ReaderTy {
    #[inline]
    async fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        ReaderTy::read(*self, data).await
    }
}

/// Writer trait.
pub trait Write {
    async fn write(&mut self, data: &[u8]) -> Result<usize>;
    async fn flush(&mut self) -> Result<()>;

    async fn write_all(&mut self, mut data: &[u8]) -> Result<()> {
        while !data.is_empty() {
            let count = self.write(data).await?;
            data = &data[count..];
        }
        Ok(())
    }
}

impl<WriterTy: Write> Write for &mut WriterTy {
    #[inline]
    async fn write(&mut self, data: &[u8]) -> Result<usize> {
        WriterTy::write(*self, data).await
    }

    #[inline]
    async fn flush(&mut self) -> Result<()> {
        WriterTy::flush(*self).await
    }
}
