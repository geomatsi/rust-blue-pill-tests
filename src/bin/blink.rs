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

use core::fmt::Write;

#[entry]
fn main() -> ! {
    let mut stdout = hio::hstdout().unwrap();

    let dp = stm32f103xx::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    loop {
        writeln!(stdout, "Hello World!").unwrap();

        led.set_high();
        delay(5000);
        led.set_low();
        delay(500);
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
