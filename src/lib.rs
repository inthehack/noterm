//! noterm, a no-std crate for interacting with terminal.

#![cfg_attr(not(test), no_std)]

use core::fmt;

mod macros;

pub mod cursor;
pub mod io;
pub mod terminal;

/// Command trait.
pub trait Command {
    fn write(&self, writer: &mut impl fmt::Write) -> fmt::Result;
}

pub trait Queuable {
    fn queue(&mut self, command: impl Command) -> io::Result<&mut Self>;
}

pub trait Executable {
    fn execute(&mut self, command: impl Command) -> io::Result<&mut Self>;
}

impl<WriterTy: io::Write> Queuable for WriterTy {
    fn queue(&mut self, command: impl Command) -> io::Result<&mut Self> {
        command_write_ansi(self, command)?;
        Ok(self)
    }
}

impl<WriterTy: io::Write> Executable for WriterTy {
    fn execute(&mut self, command: impl Command) -> io::Result<&mut Self> {
        self.queue(command)?;
        self.flush()?;
        Ok(self)
    }
}

fn command_write_ansi<WriterTy: io::Write, CommandTy: Command>(
    writer: &mut WriterTy,
    command: CommandTy,
) -> io::Result<()> {
    struct Adapter<Ty> {
        inner: Ty,
        result: io::Result<()>,
    }

    impl<Ty: io::Write> fmt::Write for Adapter<Ty> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            match self.inner.write_all(s.as_bytes()) {
                Ok(_) => Ok(()),
                Err(err) => {
                    self.result = Err(err);
                    Err(fmt::Error)
                }
            }
        }
    }

    let mut adapter = Adapter {
        inner: writer,
        result: Ok(()),
    };

    command
        .write(&mut adapter)
        .map_err(|_| match adapter.result {
            Ok(()) => panic!("command write incorrectly errored"),
            Err(err) => err,
        })
}
