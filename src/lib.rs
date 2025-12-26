//! noterm, a no-std crate for interacting with terminal.

#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]

use core::fmt;

mod macros;

pub mod terminal;

/// Reader trait.
pub trait Read {
    type Error: fmt::Debug;

    async fn read(&mut self, data: &mut [u8]) -> Result<usize, Self::Error>;

    async fn read_all(&mut self, mut data: &mut [u8]) -> Result<(), Self::Error> {
        while !data.is_empty() {
            let count = self.read(data).await?;
            data = &mut data[count..];
        }
        Ok(())
    }
}

impl<ValueTy: Read> Read for &mut ValueTy {
    type Error = ValueTy::Error;

    async fn read(&mut self, data: &mut [u8]) -> Result<usize, Self::Error> {
        ValueTy::read(*self, data).await
    }
}

/// Writer trait.
pub trait Write {
    type Error: fmt::Debug;

    async fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error>;

    async fn write_all(&mut self, mut data: &[u8]) -> Result<(), Self::Error> {
        while !data.is_empty() {
            let count = self.write(data).await?;
            data = &data[count..];
        }
        Ok(())
    }
}

impl<ValueTy: Write> Write for &mut ValueTy {
    type Error = ValueTy::Error;

    async fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
        ValueTy::write(*self, data).await
    }
}

/// Command trait.
pub trait Command {
    async fn write<WriterTy: Write>(&self, writer: &mut WriterTy) -> Result<(), WriterTy::Error>;
}
