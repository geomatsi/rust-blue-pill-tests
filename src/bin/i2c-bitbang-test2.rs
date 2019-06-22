//
// I2C bitbang for AT24 flash
//

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
use nb::block;

extern crate eeprom24x;
use eeprom24x::Eeprom24x;
use eeprom24x::SlaveAddr;

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

    let mut delay = Timer::tim2(dp.TIM2, 10.hz(), clocks, &mut rcc.apb1);
    let tmr = Timer::tim3(dp.TIM3, 200.khz(), clocks, &mut rcc.apb1);
    let scl = gpioa.pa1.into_open_drain_output(&mut gpioa.crl);
    let sda = gpioa.pa2.into_open_drain_output(&mut gpioa.crl);

    let i2c = bitbang_hal::i2c::I2cBB::new(scl, sda, tmr);
    let mut eeprom = Eeprom24x::new_24x04(i2c, SlaveAddr::default());

    // check high memory addresses: 1 bit passed as a part of i2c addr
    let addrs1: [u32; 4] = [0x100, 0x10F, 0x1F0, 0x1EE];
    let byte_w1 = 0xe5;
    let addrs2: [u32; 4] = [0x00, 0x0F, 0xF0, 0xEE];
    let byte_w2 = 0xaa;

    for addr in addrs1.iter() {
        eeprom.write_byte(*addr, byte_w1).unwrap();
        // need to wait before next write
        block!(delay.wait()).ok();
    }

    for addr in addrs2.iter() {
        eeprom.write_byte(*addr, byte_w2).unwrap();
        // need to wait before next write
        block!(delay.wait()).ok();
    }

    loop {
        for addr in addrs1.iter() {
            let byte_r = eeprom.read_byte(*addr).unwrap();
            hprintln!("w1[{}] r[{}]", byte_w1, byte_r).unwrap();
            block!(delay.wait()).ok();
        }

        for addr in addrs2.iter() {
            let byte_r = eeprom.read_byte(*addr).unwrap();
            hprintln!("w1[{}] r[{}]", byte_w2, byte_r).unwrap();
            block!(delay.wait()).ok();
        }
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
