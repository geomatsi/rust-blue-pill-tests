//
// I2C bitbang for LM75A temperature sensor
//

#![no_std]
#![no_main]

use bitbang_hal;
use cortex_m as cm;
use cortex_m_rt as rt;
use cortex_m_semihosting::hprintln;
use hal::prelude::*;
use hal::timer::Timer;
use lm75;
use lm75::{Lm75, SlaveAddr};
use panic_semihosting as _;
use rt::entry;
use stm32f1xx_hal as hal;

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

    let tmr = Timer::tim3(dp.TIM3, &clocks, &mut rcc.apb1).start_count_down(200.khz());
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
