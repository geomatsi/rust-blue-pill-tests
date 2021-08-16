#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt::entry;
use hal::delay::Delay;
use hal::prelude::*;
use hal::spi::Spi;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use smart_leds::{SmartLedsWrite, RGB8};
use stm32f1xx_hal as hal;

const NUM_LEDS: usize = 8;

#[entry]
fn main() -> ! {
    let dp = hal::stm32::Peripherals::take().unwrap();
    let cp = cm::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    rtt_init_print!();

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let pins = (
        gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh),
        gpiob.pb14.into_floating_input(&mut gpiob.crh),
        gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh),
    );

    let spi = Spi::spi2(
        dp.SPI2,
        pins,
        ws2812_spi::MODE,
        3.mhz(),
        clocks,
        &mut rcc.apb1,
    );

    rprintln!("ready to go...");

    let mut ws = ws2812_spi::Ws2812::new(spi);
    let mut pdata = [RGB8::default(); NUM_LEDS];
    let cdata: [RGB8; 3] = [
        RGB8 {
            r: 0x10,
            g: 0x0,
            b: 0x0,
        },
        RGB8 {
            r: 0x0,
            g: 0x10,
            b: 0x0,
        },
        RGB8 {
            r: 0x0,
            g: 0x0,
            b: 0x10,
        },
    ];
    let mut p: usize = 0;
    let mut c: usize = 0;

    loop {
        let pos = p.wrapping_rem(NUM_LEDS);
        let color = c.wrapping_rem(3);

        rprintln!("iteration: pos {} color {}...", pos, color);

        pdata[pos] = cdata[color];
        ws.write(pdata.iter().cloned()).unwrap();
        delay.delay_ms(100u16);

        if pos == 7 {
            c += 1;
        }

        p += 1;
    }
}
