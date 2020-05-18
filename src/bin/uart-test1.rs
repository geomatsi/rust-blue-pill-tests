#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use hal::delay::Delay;
use hal::prelude::*;
use hal::serial::{Config, Serial};
use nb::block;
use panic_semihosting as _;
use stm32f1xx_hal as hal;

#[entry]
fn main() -> ! {
    let dp = hal::stm32::Peripherals::take().unwrap();
    let cp = cm::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(32.mhz())
        .pclk1(16.mhz())
        .freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut delay = Delay::new(cp.SYST, clocks);

    let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    let rx = gpiob.pb11.into_floating_input(&mut gpiob.crh);

    let mut serial = Serial::usart3(
        dp.USART3,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115_200.bps()),
        clocks,
        &mut rcc.apb1,
    );

    loop {
        hprintln!("Hello World!").unwrap();
        for byte in b"Hello, World!\r\n" {
            block!(serial.write(*byte)).unwrap();
        }
        delay.delay_ms(1_000u16);
    }
}
