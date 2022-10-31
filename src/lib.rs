//! Library for the GC9A01A display driver
#![no_std]

mod registers;

use embedded_hal::blocking::delay;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

use registers::*;

pub enum Error<S, P> {
    /// Error in SPI communication
    Spi(S),
    /// Error in GPIO manipulation
    Gpio(P),
    /// Error while using delay
    Delay,
}

pub struct GC9A01A<SPI, PIN> {
    /// Spi communication channel.
    spi: SPI,
    /// Data/Command selection output pin.
    dc: PIN,
    /// Reset pin.
    rst: PIN,
    /// Backlight pin.
    bl: PIN,
}

impl<SPI, PIN, SE, PE> GC9A01A<SPI, PIN>
where
    SPI: spi::Write<u8, Error = SE>,
    PIN: OutputPin<Error = PE>,
{
    pub fn new(spi: SPI, dc: PIN, rst: PIN, bl: PIN) -> Self {
        Self { spi, dc, rst, bl }
    }

    pub fn initialize<D>(&mut self, delay: &mut D) -> Result<(), Error<SE, PE>>
    where
        D: delay::DelayMs<u32>,
    {
        for o in INIT_SEQ {
            match o {
                InitOp::Cmd(c) => {
                    self.send_command(c.cmd)?;
                    self.send_data(c.data)?;
                }
                InitOp::Delay(d) => {
                    delay.delay_ms(d);
                }
            }
        }

        Ok(())
    }

    pub fn reset(&mut self) {}

    fn send_command(&mut self, command: u8) -> Result<(), Error<SE, PE>> {
        self.dc.set_low().map_err(Error::Gpio)?;
        self.spi.write(&[command]).map_err(Error::Spi)?;
        Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Error<SE, PE>> {
        self.dc.set_high().map_err(Error::Gpio)?;
        self.spi.write(data).map_err(Error::Spi)?;
        Ok(())
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
