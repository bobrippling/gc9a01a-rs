//! Draw a square, circle and triangle on the screen using the embedded_graphics library
//! over a 4 wire SPI interface.
//!
//! This is an example for the rp2040, more specifically the Raspberry Pi Pico.
//!
//! Wiring connections are as follows:
//!
//! ```
//! 3V3    -> VCC
//! GND    -> GND
//! GPIO8  -> D/C
//! GPIO9  -> CS
//! GPIO10 -> CLK
//! GPIO11 -> DIN
//! GPIO12 -> RST
//! GPIO13 -> BL
//! ```
//!
//! These default settings should also work when using the [RoundyPi](https://github.com/sbcshop/RoundyPi).
//! If you have a waveshare *RP2040 MCU Board*, you should use gpio 25 for backlight control (`BL`).

#![no_std]
#![no_main]

use panic_halt as _;
use rp_pico as bsp;

use bsp::entry;
use fugit::RateExtU32;

use display_interface_spi::SPIInterface;
use embedded_graphics::prelude::*;
use embedded_graphics::{
    pixelcolor::Rgb565,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac, pwm,
    sio::Sio,
    spi,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_sclk = pins.gpio10.into_mode::<gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio11.into_mode::<gpio::FunctionSpi>();
    let spi_cs = pins.gpio9.into_push_pull_output();

    // Create an SPI driver instance for the SPI1 device
    let spi = spi::Spi::<_, _, 8>::new(pac.SPI1);

    // Exchange the uninitialised SPI driver for an initialised one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        8_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let dc_pin = pins.gpio8.into_push_pull_output();
    let rst_pin = pins.gpio12.into_push_pull_output();

    let spi_interface = SPIInterface::new(spi, dc_pin, spi_cs);

    // initialize PWM for backlight
    let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Configure PWM6
    let mut pwm = pwm_slices.pwm6;
    pwm.set_ph_correct();
    pwm.enable();

    // Output channel B on PWM6 to GPIO 13
    let mut channel = pwm.channel_b;
    channel.output_to(pins.gpio13);

    // Create display driver
    let mut display = gc9a01a::GC9A01A::new(spi_interface, rst_pin, channel);
    // Bring out of reset
    display.reset(&mut delay).unwrap();
    // Turn on backlight
    display.set_backlight(55000);
    // Initialize registers
    display.initialize(&mut delay).unwrap();
    // Fill screen with single color
    display.clear(Rgb565::CSS_FOREST_GREEN).unwrap();

    let yoffset = 100;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(2)
        .stroke_color(Rgb565::CSS_RED)
        .build();

    // screen outline for the round 1.28 inch Waveshare display
    Circle::new(Point::new(1, 1), 238)
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    // triangle
    Triangle::new(
        Point::new(50, 32 + yoffset),
        Point::new(50 + 32, 32 + yoffset),
        Point::new(50 + 8, yoffset),
    )
    .into_styled(style)
    .draw(&mut display)
    .unwrap();

    // square
    Rectangle::new(Point::new(110, yoffset), Size::new_equal(32))
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    // circle
    Circle::new(Point::new(170, yoffset), 32)
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    exit()
}

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
