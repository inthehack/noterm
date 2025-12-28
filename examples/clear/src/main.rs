use std::io::stdout;

use noterm::{Queuable, io::Write, terminal::Action};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    for _ in 0..10 {
        println!("Hello World!");

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let mut adapter = WriteAdapter::new(stdout());
        adapter.queue(Action::ClearScreen)?.flush()?;
    }

    Ok(())
}

struct WriteAdapter<WriterTy>(WriterTy);

impl<WriterTy: std::io::Write> WriteAdapter<WriterTy> {
    pub fn new(writer: WriterTy) -> Self {
        WriteAdapter(writer)
    }
}

impl<WriterTy: std::io::Write> noterm::io::Write for WriteAdapter<WriterTy> {
    fn write(&mut self, data: &[u8]) -> noterm::io::Result<usize> {
        let count = self.0.write(data).map_err(|_| noterm::io::Error::Unknown)?;
        Ok(count)
    }

    fn flush(&mut self) -> noterm::io::Result<()> {
        self.0.flush().map_err(|_| noterm::io::Error::Unknown)?;
        Ok(())
    }
}
