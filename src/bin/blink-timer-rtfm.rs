#![feature(extern_prelude)]
#![no_main]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
use rtfm::{app, Threshold};

extern crate cortex_m_semihosting as sh;
use sh::hio;

extern crate panic_semihosting;

extern crate stm32f103xx_hal as hal;
use hal::prelude::*;
use hal::timer::Event;
use hal::timer::Timer;

use core::fmt::Write;

app! {
    device: hal::stm32f103xx,

    resources: {
        static skip: u8 = 0;
        static led1: hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>;
        static tmr2: hal::timer::Timer<stm32f103xx::TIM2>;
        static tmr3: hal::timer::Timer<stm32f103xx::TIM3>;
    },

    tasks: {
        TIM2: {
            path: tim2_handler,
            resources: [tmr2, skip],
        },
        TIM3: {
            path: tim3_handler,
            resources: [led1, tmr3],
        },
    }
}

fn init(p: init::Peripherals, _r: init::Resources) -> init::LateResources {
    let mut rcc = p.device.RCC.constrain();

    // configure clocks
    let mut flash = p.device.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.mhz())
        .pclk1(8.mhz())
        .freeze(&mut flash.acr);

    // configure PC13 pin to blink LED
    let mut gpioc = p.device.GPIOC.split(&mut rcc.apb2);
    let l1 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // configure and start TIM2 periodic timer
    let mut t2 = Timer::tim2(p.device.TIM2, 1.hz(), clocks, &mut rcc.apb1);
    t2.listen(Event::Update);

    // configure and start TIM3 periodic timer
    let mut t3 = Timer::tim3(p.device.TIM3, 5.hz(), clocks, &mut rcc.apb1);
    t3.listen(Event::Update);

    // init late resources
    init::LateResources {
        led1: l1,
        tmr2: t2,
        tmr3: t3,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn tim2_handler(_t: &mut Threshold, mut r: TIM2::Resources) {
    let mut dbg = hio::hstdout().unwrap();
    *r.skip += 1;

    if *r.skip == 5 {
        writeln!(dbg, "TIM2").unwrap();
        *r.skip = 0;
    }

    r.tmr2.start(1.hz());
}

fn tim3_handler(_t: &mut Threshold, mut r: TIM3::Resources) {
    r.led1.toggle();
    r.tmr3.start(5.hz());
}
