#![no_main]
#![no_std]

use embedded_hal::digital::v2::OutputPin;

use cortex_m_rt as rt;
use cortex_m_semihosting::hprintln;
use hal::prelude::*;
use hal::timer::Timer;
use nb::block;
use panic_semihosting as _;
use rt::entry;
use stm32f1xx_hal as hal;

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

    let mut tmr = Timer::tim3(dp.TIM3, &clocks, &mut rcc.apb1).start_count_down(1.hz());

    loop {
        c += 1;
        hprintln!("cycle {}", c).unwrap();

        led.set_high().unwrap();
        block!(tmr.wait()).ok();
        led.set_low().unwrap();
        block!(tmr.wait()).ok();
    }
}
