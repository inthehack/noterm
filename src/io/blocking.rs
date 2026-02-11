//! Blocking I/O.

use super::Result;

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

#[cfg(feature = "std")]
impl Write for String {
    fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.push_str(str::from_utf8(data).unwrap());
        Ok(data.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
