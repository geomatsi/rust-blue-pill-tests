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
use hal::adc::Adc;
use hal::prelude::*;
use hal::stm32;

#[entry]
fn main() -> ! {
    let p = stm32::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(8.mhz()).pclk1(2.mhz()).adcclk(1.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(16.mhz()).pclk1(4.mhz()).adcclk(2.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(24.mhz()).pclk1(9.mhz()).adcclk(3.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(24.mhz()).pclk1(12.mhz()).adcclk(4.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(24.mhz()).pclk1(12.mhz()).adcclk(6.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(36.mhz()).pclk1(12.mhz()).adcclk(6.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(32.mhz()).pclk1(16.mhz()).adcclk(8.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(40.mhz()).pclk1(20.mhz()).adcclk(10.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(48.mhz()).pclk1(24.mhz()).adcclk(12.mhz()).freeze(&mut flash.acr);
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(56.mhz())
        .pclk1(28.mhz())
        .adcclk(14.mhz())
        .freeze(&mut flash.acr);

    hprintln!("SYSCLK: {} Hz ...", clocks.sysclk().0).unwrap();
    hprintln!("ADCCLK: {} Hz ...", clocks.adcclk().0).unwrap();

    // ADC setup
    let mut adc = Adc::adc1(p.ADC1, &mut rcc.apb2, clocks);

    loop {
        // Ambient temperature
        let temp = adc.read_temp();

        hprintln!("Temp: {} C", temp).unwrap();

        delay(10000);
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
