#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_semihosting::hprintln;
use hal::prelude::*;
use hal::stm32;
use hal::timer::Event;
use hal::timer::Timer;
use panic_semihosting as _;
use rtfm::app;

use stm32f1xx_hal as hal;

#[app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // resources
        #[init(0)]
        beat: u8,
        // late resources
        led1: hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>,
        tmr2: hal::timer::CountDownTimer<stm32::TIM2>,
        tmr3: hal::timer::CountDownTimer<stm32::TIM3>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let mut rcc = cx.device.RCC.constrain();

        // configure clocks
        let mut flash = cx.device.FLASH.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(8.mhz())
            .pclk1(8.mhz())
            .freeze(&mut flash.acr);

        // configure PC13 pin to blink LED
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);
        let l1 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        // configure and start TIM2 periodic timer
        let mut t2 = Timer::tim2(cx.device.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz());
        t2.listen(Event::Update);

        // configure and start TIM3 periodic timer
        let mut t3 = Timer::tim3(cx.device.TIM3, &clocks, &mut rcc.apb1).start_count_down(5.hz());
        t3.listen(Event::Update);

        init::LateResources {
            led1: l1,
            tmr2: t2,
            tmr3: t3,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cm::asm::wfi();
        }
    }

    #[task(binds = TIM2, resources = [beat, tmr2])]
    fn tim2(cx: tim2::Context) {
        hprintln!("TIM2 beat = {}", *cx.resources.beat).unwrap();

        *cx.resources.beat += 1;
        cx.resources.tmr2.start(1.hz());
    }

    #[task(binds = TIM3, resources = [led1, tmr3])]
    fn tim3(cx: tim3::Context) {
        cx.resources.led1.toggle().unwrap();
        cx.resources.tmr3.start(5.hz());
    }
};
