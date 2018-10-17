#![feature(extern_prelude)]
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m;

extern crate cortex_m as cm;

extern crate cortex_m_rtfm as rtfm;
use rtfm::{app, Threshold};

extern crate panic_itm;

extern crate stm32f103xx_hal as hal;
use hal::gpio;
use hal::prelude::*;
use hal::spi::Spi;
use hal::timer::Event;
use hal::timer::Timer;

extern crate nrf24l01;
use nrf24l01::NRF24L01;

app! {
    device: hal::stm32f103xx,

    resources: {
        static NRF: NRF24L01<
            Spi<
                hal::stm32f103xx::SPI1,
                (
                    gpio::gpioa::PA5<gpio::Alternate<gpio::PushPull>>,
                    gpio::gpioa::PA6<gpio::Input<gpio::Floating>>,
                    gpio::gpioa::PA7<gpio::Alternate<gpio::PushPull>>
                )
            >,
            gpio::gpioa::PA4<gpio::Output<gpio::PushPull>>,
            gpio::gpiob::PB0<gpio::Output<gpio::PushPull>>
        >;
        static LED: gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>;
        static ITM: hal::stm32f103xx::ITM;
        static TMR: hal::timer::Timer<stm32f103xx::TIM3>;
    },

    idle: {
        resources: [ITM],
    },

    tasks: {
        TIM3: {
            path: timer_handler,
            resources: [TMR, LED, ITM, NRF],
        }
    },
}

fn init(p: init::Peripherals) -> init::LateResources {
    // configure clocks
    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.mhz())
        .pclk1(8.mhz())
        .freeze(&mut flash.acr);

    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.device.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);

    // configure PC13 pin as LED
    let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // configure PB0 pin as NRF24 CE
    let ce = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);

    // configure PA4 pin as NRF24 NCS
    let cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);

    // configure PA5/PA6/PA7 as SPI1 pins
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    // configure SPI1 for NRF24
    let spi = Spi::spi1(
        p.device.SPI1,
        (sck, miso, mosi),
        &mut afio.mapr,
        nrf24l01::MODE,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    // nRF24L01 setup, by default rate=2Mbps, CRC=1
    let mut nrf = NRF24L01::new(spi, cs, ce, 10 /* chan */, 4 /* payload */).unwrap();
    let addr = [0xe0, 0xe1, 0xe2, 0xe3, 0xe4];
    nrf.set_taddr(&addr).unwrap();
    nrf.config().unwrap();

    // configure and start TIM3 periodic timer
    let mut tmr = Timer::tim3(p.device.TIM3, 1.hz(), clocks, &mut rcc.apb1);
    tmr.listen(Event::Update);

    // init late resources
    init::LateResources {
        NRF: nrf,
        LED: led,
        TMR: tmr,
        ITM: p.core.ITM,
    }
}

fn idle(_t: &mut Threshold, mut _r: idle::Resources) -> ! {
    loop {
        rtfm::wfi();
    }
}

fn timer_handler(_t: &mut Threshold, mut r: TIM3::Resources) {
    let dbg = &mut r.ITM.stim[0];
    let buffer = [0x31, 0x32, 0x33, 0x34];

    iprintln!(dbg, "TX...");

    r.NRF.send(&buffer).unwrap();
    r.LED.toggle();
    r.TMR.start(1.hz());
}
