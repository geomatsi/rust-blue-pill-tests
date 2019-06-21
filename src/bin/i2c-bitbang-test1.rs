#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
use rt::entry;
use rt::exception;
use rt::ExceptionFrame;

extern crate cortex_m as cm;

extern crate cortex_m_semihosting as sh;
use sh::hprintln;

extern crate panic_semihosting;

extern crate stm32f1xx_hal as hal;
use hal::prelude::*;
use hal::timer::Timer;

use bitbang_hal;

extern crate lm75;
use lm75::{Lm75, SlaveAddr};

#[entry]
fn main() -> ! {
    let dp = hal::stm32::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    let mut flash = dp.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(32.mhz())
        .pclk1(16.mhz())
        .freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.sysclk(8.mhz()).pclk1(8.mhz()).freeze(&mut flash.acr);

    let mut tmr = Timer::tim3(dp.TIM3, 300.khz(), clocks, &mut rcc.apb1);
    let scl = gpioa.pa1.into_open_drain_output(&mut gpioa.crl);
    let sda = gpioa.pa2.into_open_drain_output(&mut gpioa.crl);

    let i2c = bitbang_hal::i2c::I2cBB::new(scl, sda, tmr);
    let mut sensor = Lm75::new(i2c, SlaveAddr::default());

    loop {
        let temp = sensor.read_temperature().unwrap();
        hprintln!("T: {}", temp).unwrap();
        delay(5000);
    }
}

fn delay(count: u32) {
    for _ in 0..count {
        cm::asm::nop();
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
