//! Library for the GC9A01A display driver
#![no_std]

mod registers;

use embedded_hal::blocking::delay;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;

use registers::*;

#[derive(Debug)]
pub enum Error<S, P> {
    /// Error in SPI communication
    Spi(S),
    /// Error in GPIO manipulation
    Gpio(P),
    /// Error while using delay
    Delay,
}

#[derive(Debug)]
pub struct GC9A01A<Spi, PinCS, PinDC, PinRst, Pwm> {
    /// Spi communication channel.
    spi: Spi,
    /// Chip select pin.
    cs: PinCS,
    /// Data/Command selection output pin.
    dc: PinDC,
    /// Reset pin.
    rst: PinRst,
    /// Backlight pin, pulse-width modulated.
    bl: Pwm,
}

impl<Spi, PinCS, PinDC, PinRst, Pwm, SE, PE> GC9A01A<Spi, PinCS, PinDC, PinRst, Pwm>
where
    Spi: spi::Write<u8, Error = SE>,
    PinCS: OutputPin<Error = PE>,
    PinDC: OutputPin<Error = PE>,
    PinRst: OutputPin<Error = PE>,
    Pwm: PwmPin,
{
    pub const WIDTH: u8 = 240;
    pub const HEIGHT: u8 = 240;

    pub fn new(spi: Spi, cs: PinCS, dc: PinDC, rst: PinRst, bl: Pwm) -> Self {
        Self {
            spi,
            cs,
            dc,
            rst,
            bl,
        }
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

    pub fn reset<D>(&mut self, delay: &mut D) -> Result<(), Error<SE, PE>>
    where
        D: delay::DelayMs<u32>,
    {
        self.rst.set_high().map_err(Error::Gpio)?;
        delay.delay_ms(100);
        self.rst.set_low().map_err(Error::Gpio)?;
        delay.delay_ms(100);
        self.rst.set_high().map_err(Error::Gpio)?;
        delay.delay_ms(100);
        Ok(())
    }

    pub fn set_backlight(&mut self, duty: Pwm::Duty) {
        self.bl.set_duty(duty);
    }

    pub fn clear(&mut self, color: u16) -> Result<(), Error<SE, PE>> {
        self.set_windows()?;
        self.dc.set_high().map_err(Error::Gpio)?;
        self.cs.set_low().map_err(Error::Gpio)?;
        for _ in 0..Self::WIDTH {
            for _ in 0..Self::HEIGHT {
                self.spi
                    .write(&u16::to_be_bytes(color))
                    .map_err(Error::Spi)?;
            }
        }
        self.cs.set_high().map_err(Error::Gpio)?;
        Ok(())
    }

    fn set_windows(&mut self) -> Result<(), Error<SE, PE>> {
        self.send_command(GC9A01A_CASET)?;
        self.send_data(&[0x00, 0x00, 0x00, Self::WIDTH - 1])?;
        self.send_command(GC9A01A_PASET)?;
        self.send_data(&[0x00, 0x00, 0x00, Self::HEIGHT - 1])?;
        self.send_command(GC9A01A_RAMWR)
    }

    fn send_command(&mut self, command: u8) -> Result<(), Error<SE, PE>> {
        self.dc.set_low().map_err(Error::Gpio)?;
        self.with_cs_low(|g| g.spi.write(&[command]).map_err(Error::Spi))
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Error<SE, PE>> {
        self.dc.set_high().map_err(Error::Gpio)?;
        self.with_cs_low(|g| g.spi.write(data).map_err(Error::Spi))
    }

    fn with_cs_low<F, T>(&mut self, f: F) -> Result<T, Error<SE, PE>>
    where
        F: FnOnce(&mut Self) -> Result<T, Error<SE, PE>>,
    {
        self.cs.set_low().map_err(Error::Gpio)?;
        let result = f(self);
        self.cs.set_high().map_err(Error::Gpio)?;

        result
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
