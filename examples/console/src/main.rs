#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32 as hal;

use hal::bind_interrupts;
use hal::mode::Async;

use {defmt_rtt as _, panic_probe as _};

use noterm::cursor::{Home, MoveToNextLine};
use noterm::style::{Color, Print};
use noterm::terminal::{Clear, ClearType};

use noterm::Queuable as _;
use noterm::io::blocking::Write as _;
use noterm::style::Stylized as _;

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

    uart.queue(Clear(ClearType::All))
        .expect("queued")
        .queue(Home)
        .expect("queued")
        .flush()
        .expect("flushed");

    uart.queue(Print("Hello World".bold()))
        .expect("queued")
        .queue(MoveToNextLine(2))
        .expect("queued")
        .queue(Print("Let's start!".fg(Color::Black).bg(Color::White)))
        .expect("queued")
        .flush()
        .expect("flushed");

    let mut buffer = [0u8; 32];
    let mut rpos = 0;
    let mut wpos = 0;

    loop {
        let Ok(amount) = uart.inner.read_until_idle(&mut buffer[wpos..]).await else {
            continue;
        };

        if amount == 0 {
            continue;
        }

        wpos += amount;

        let Ok(mut input) = str::from_utf8(&buffer[rpos..wpos]) else {
            wpos = 0;
            rpos = 0;
            continue;
        };

        loop {
            if input.is_empty() {
                wpos = 0;
                rpos = 0;
                break;
            }

            let event = match noterm::events::parse(input) {
                Ok((rest, event)) => {
                    rpos += input.len() - rest.len();

                    input = rest;
                    event
                }

                Err(nom::Err::Incomplete(_)) => {
                    break;
                }

                Err(nom::Err::Error(_)) => {
                    break;
                }

                Err(nom::Err::Failure(_)) => {
                    wpos = 0;
                    rpos = 0;
                    break;
                }
            };

            defmt::println!("event: {:?}", defmt::Debug2Format(&event));
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
