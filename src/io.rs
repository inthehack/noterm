use core::fmt::Debug;

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
    fn read(&mut self, data: &mut [u8]) -> Result<usize>;

    fn read_all(&mut self, mut data: &mut [u8]) -> Result<()> {
        while !data.is_empty() {
            let count = self.read(data)?;
            data = &mut data[count..];
        }
        Ok(())
    }
}

impl<ReaderTy: Read> Read for &mut ReaderTy {
    #[inline]
    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        ReaderTy::read(*self, data)
    }
}

/// Writer trait.
pub trait Write {
    fn write(&mut self, data: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, mut data: &[u8]) -> Result<()> {
        while !data.is_empty() {
            let count = self.write(data)?;
            data = &data[count..];
        }
        Ok(())
    }
}

impl<WriterTy: Write> Write for &mut WriterTy {
    #[inline]
    fn write(&mut self, data: &[u8]) -> Result<usize> {
        WriterTy::write(*self, data)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        WriterTy::flush(*self)
    }
}
