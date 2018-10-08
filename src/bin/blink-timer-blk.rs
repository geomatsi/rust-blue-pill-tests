#![no_main]
#![no_std]

extern crate cortex_m_rt as rt;
use rt::entry;
use rt::exception;
use rt::ExceptionFrame;

extern crate cortex_m as cm;

extern crate cortex_m_semihosting as sh;
use sh::hio;

extern crate panic_semihosting;

extern crate stm32f103xx_hal as hal;
use hal::prelude::*;
use hal::stm32f103xx;
use hal::timer::Timer;

use core::fmt::Write;

#[macro_use(block)]
extern crate nb;

#[entry]
fn main() -> ! {
    let mut c: u8 = 0;

    let mut stdout = hio::hstdout().unwrap();

    let dp = stm32f103xx::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let mut flash = dp.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.mhz())
        .pclk1(8.mhz())
        .freeze(&mut flash.acr);

    let mut tmr = Timer::tim3(dp.TIM3, 1.hz(), clocks, &mut rcc.apb1);

    loop {
        c = c + 1;
        writeln!(stdout, "cycle {}", c).unwrap();

        led.set_high();
        tmr.start(10.hz());
        block!(tmr.wait()).unwrap();
        led.set_low();
        tmr.start(1.hz());
        block!(tmr.wait()).unwrap();
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
