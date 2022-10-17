#![no_std]
#![no_main]

use core::num::Wrapping;

use bsp::entry;
use rp_pico as bsp;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use embedded_hal::{blocking::spi::Write, digital::v2::OutputPin};

use fugit::*;

use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    info!("Program start");
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

    let mut _card_cs = pins.gpio26.into_push_pull_output();
    let mut lcd_backlight = pins.gpio15.into_push_pull_output();
    let mut lcd_reset = pins.gpio28.into_push_pull_output();
    let mut lcd_dc = pins.gpio27.into_push_pull_output();

    let (_stx, _srx, _sck, _cs) = (
        pins.gpio19.into_mode::<bsp::hal::gpio::FunctionSpi>(),
        pins.gpio16.into_mode::<bsp::hal::gpio::FunctionSpi>(),
        pins.gpio18.into_mode::<bsp::hal::gpio::FunctionSpi>(),
        pins.gpio17.into_mode::<bsp::hal::gpio::FunctionSpi>(),
    );

    let spi = bsp::hal::Spi::<_, _, 8>::new(pac.SPI0);
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        32_u32.MHz(),
        &embedded_hal::spi::MODE_0,
    );

    lcd_backlight.set_high().unwrap();
    lcd_dc.set_high().unwrap();

    lcd_reset.set_low().unwrap();
    delay.delay_ms(50);
    lcd_reset.set_high().unwrap();

    let mut disp = st7735_lcd::ST7735::new(spi, lcd_dc, lcd_reset, true, false, 160, 128);

    disp.init(&mut delay).unwrap();
    disp.set_orientation(&st7735_lcd::Orientation::Landscape)
        .unwrap();
    boot_pattern(&mut disp, 0);

    let mut game: conway::Conway<160, 128> = conway::Conway::new();
    game.randomize(7789432);
    disp.clear(Rgb565::BLACK).unwrap();

    loop {
        for x in 0..160 {
            for y in 0..128 {
                match game.counts[game.buf][x][y] & Wrapping(0x80) {
                    Wrapping(0x80) => disp.set_pixel(x as u16, y as u16, 0xFFFF).unwrap(),
                    _ => disp.set_pixel(x as u16, y as u16, 0).unwrap(),
                }
            }
        }
        game.step();
    }
}

fn boot_pattern<A: Write<u8>, B: OutputPin, C: OutputPin>(
    disp: &mut st7735_lcd::ST7735<A, B, C>,
    modifier: u16,
) {
    for x in 0..160 {
        for y in 0..128 {
            let xr = x & 0b11111;
            let yr = y & 0b11111;

            let xs = x >> 3;
            let ys = y >> 3;
            let l = xr ^ yr;

            let r = l ^ xs ^ modifier;
            let g = l ^ ys ^ modifier;
            let b = l ^ xs ^ ys ^ modifier;

            let c: u16 = r | (g << 6) | (b << 11);
            disp.set_pixel(x, y, c).unwrap();
        }
    }
}
