#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::digital::{v1_compat::OldOutputPin, v2::OutputPin};
use hal::prelude::*;
use hal::spi::Spi;
use mfrc522::Mfrc522;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal as hal;

#[entry]
fn main() -> ! {
    let dp = hal::stm32::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    rtt_init_print!();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(16.mhz())
        .pclk1(4.mhz())
        .adcclk(2.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_high().unwrap();

    let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    let sck = pb3.into_alternate_push_pull(&mut gpiob.crl);
    let miso = pb4;
    let mosi = gpiob.pb5.into_alternate_push_pull(&mut gpiob.crl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        mfrc522::MODE,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let nss = pa15.into_push_pull_output(&mut gpioa.crh);
    let mut mfrc522 = Mfrc522::new(spi, OldOutputPin::from(nss)).unwrap();

    loop {
        if let Ok(atqa) = mfrc522.reqa() {
            if let Ok(uid) = mfrc522.select(&atqa) {
                rprintln!("* {:?}", uid);
            }
        }
    }
}
