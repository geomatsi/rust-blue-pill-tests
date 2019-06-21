#![no_main]
#![no_std]

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

extern crate nb;
use nb::block;

#[entry]
fn main() -> ! {
    let mut c: u8 = 0;

    let dp = hal::stm32::Peripherals::take().unwrap();
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
        c += 1;
        hprintln!("cycle {}", c).unwrap();

        led.set_high();
        block!(tmr.wait()).ok();
        led.set_low();
        block!(tmr.wait()).ok();
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
