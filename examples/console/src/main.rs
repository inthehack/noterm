#![no_std]
#![no_main]

use core::pin::Pin;

use embassy_executor::Spawner;
use embassy_stm32 as hal;
use futures::stream::StreamExt;

use hal::bind_interrupts;
use hal::mode::Async;

use {defmt_rtt as _, panic_probe as _};

use noterm::cursor::{Home, MoveToNextLine};
use noterm::events::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use noterm::style::{Color, Print, Stylized as _};
use noterm::terminal::{Clear, ClearType};
use noterm::{Executable as _, Queuable as _};

use noterm::io::blocking::Write as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let periphs = hal::init(Default::default());

    let mut uart = {
        let mut config = hal::usart::Config::default();
        config.baudrate = 115200;

        let device = hal::usart::Uart::new(
            periphs.USART1,
            periphs.PA10,
            periphs.PA9,
            Irqs,
            periphs.GPDMA1_CH10,
            periphs.GPDMA1_CH9,
            config,
        )
        .expect("should be a valid usart1 config");

        Uart::new(device)
    };

    let (tx, rx) = uart.split();

    tx.queue(Clear(ClearType::All))
        .expect("queued")
        .queue(Home)
        .expect("queued")
        .flush()
        .expect("flushed");

    tx.queue(Print("Hello World".bold()))
        .expect("queued")
        .queue(MoveToNextLine(2))
        .expect("queued")
        .queue(Print("Let's start!".fg(Color::Black).bg(Color::White)))
        .expect("queued")
        .flush()
        .expect("flushed");

    let inputs = events::stream(rx);

    while let Some(input) = inputs.next().await {
        let Ok(event) = input else {
            defmt::error!("failed to read input");
            continue;
        };

        defmt::println!("event: {}", event);

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: _,
                kind: _,
            }) => {
                uart.execute(Print(c)).expect("write char");
            }

            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: _,
                kind: _,
            }) => {
                uart.execute(MoveToNextLine(1)).expect("moved");
            }

            _ => {}
        }
    }
}

bind_interrupts!(struct Irqs {
    USART1 => hal::usart::InterruptHandler::<hal::peripherals::USART1>;
});

struct Uart<'a> {
    pub inner: hal::usart::Uart<'a, Async>,
}

impl<'a> Uart<'a> {
    pub fn new(inner: hal::usart::Uart<'a, Async>) -> Self {
        Uart { inner }
    }

    pub fn split(self) -> (UartTx<'a>, UartRx<'a>) {
        let (tx, rx) = self.inner.split();
        (UartTx::new(tx), UartRx::new(rx))
    }
}

impl noterm::io::Read for Uart<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> noterm::io::Result<usize> {
        let amount = self
            .inner
            .read_until_idle(buffer)
            .await
            .map_err(|_| noterm::io::Error::Unknown)?;

        Ok(amount)
    }
}

impl noterm::io::blocking::Write for Uart<'_> {
    fn write(&mut self, buffer: &[u8]) -> noterm::io::Result<usize> {
        let _ = embassy_futures::block_on(self.inner.write(buffer));
        Ok(buffer.len())
    }

    fn flush(&mut self) -> noterm::io::Result<()> {
        if embassy_futures::block_on(self.inner.flush()).is_err() {
            return Err(noterm::io::Error::Unknown);
        }
        Ok(())
    }
}

struct UartRx<'a> {
    pub inner: hal::usart::UartRx<'a, Async>,
}

impl<'a> UartRx<'a> {
    pub fn new(inner: hal::usart::UartRx<'a, Async>) -> Self {
        UartRx { inner }
    }
}

impl noterm::io::Read for UartRx<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> noterm::io::Result<usize> {
        let amount = self
            .inner
            .read_until_idle(buffer)
            .await
            .map_err(|_| noterm::io::Error::Unknown)?;

        Ok(amount)
    }
}

struct UartTx<'a> {
    pub inner: hal::usart::UartTx<'a, Async>,
}

impl<'a> UartTx<'a> {
    pub fn new(inner: hal::usart::UartTx<'a, Async>) -> Self {
        UartTx { inner }
    }
}

impl noterm::io::blocking::Write for UartTx<'_> {
    fn write(&mut self, buffer: &[u8]) -> noterm::io::Result<usize> {
        let _ = embassy_futures::block_on(self.inner.write(buffer));
        Ok(buffer.len())
    }

    fn flush(&mut self) -> noterm::io::Result<()> {
        if embassy_futures::block_on(self.inner.flush()).is_err() {
            return Err(noterm::io::Error::Unknown);
        }
        Ok(())
    }
}
