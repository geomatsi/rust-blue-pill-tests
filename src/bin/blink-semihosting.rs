#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt as rt;
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::OutputPin;
use hal::prelude::*;
use panic_semihosting as _;
use rt::entry;
use stm32f1xx_hal as hal;

#[entry]
fn main() -> ! {
    let dp = hal::stm32::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    let _clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(16.mhz())
        .pclk1(4.mhz())
        .adcclk(2.mhz())
        .freeze(&mut flash.acr);

    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    loop {
        hprintln!("Hello World!").unwrap();

        led.set_high().unwrap();
        delay(5000);
        led.set_low().unwrap();
        delay(500);
    }
}

fn delay(count: u32) {
    for _ in 0..count {
        cm::asm::nop();
    }
}
