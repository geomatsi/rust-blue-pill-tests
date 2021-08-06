#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m as cm;
use hal::prelude::*;
use hal::stm32;
use hal::timer::Event;
use hal::timer::Timer;
use panic_rtt_target as _;
use rtic::app;
use rtt_target::{rtt_init, UpChannel};
use stm32f1xx_hal as hal;

#[app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // resources
        #[init(0)]
        beat: u8,
        // late resources
        stream1: UpChannel,
        stream2: UpChannel,
        led1: hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>,
        tmr2: hal::timer::CountDownTimer<stm32::TIM2>,
        tmr3: hal::timer::CountDownTimer<stm32::TIM3>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let channels = rtt_init! {
            up: {
                0: {
                    size: 512
                    name: "stream1"
                }
                1: {
                    size: 512
                    name: "stream2"
                }
            }
        };

        let stream1 = channels.up.0;
        let stream2 = channels.up.1;

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
            stream1: stream1,
            stream2: stream2,
            led1: l1,
            tmr2: t2,
            tmr3: t3,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cm::asm::nop();
        }
    }

    #[task(binds = TIM2, resources = [beat, tmr2, stream1])]
    fn tim2(cx: tim2::Context) {
        writeln!(cx.resources.stream1, "TIM2 beat = {}", *cx.resources.beat).ok();

        *cx.resources.beat += 1;
        cx.resources.tmr2.clear_update_interrupt_flag();
    }

    #[task(binds = TIM3, resources = [led1, tmr3, stream2])]
    fn tim3(cx: tim3::Context) {
        writeln!(cx.resources.stream2, "TIM3 blink").ok();
        cx.resources.led1.toggle().unwrap();
        cx.resources.tmr3.clear_update_interrupt_flag();
    }
};
