#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate cortex_m as cm;
use cm::iprintln;

extern crate cortex_m_rt as rt;

extern crate rtfm;
use rtfm::app;

extern crate panic_itm;

extern crate stm32f1xx_hal as hal;
use hal::prelude::*;
use hal::stm32;
use hal::timer::Event;
use hal::timer::Timer;

#[app(device = hal::stm32)]
const APP: () = {
    // resources
    static mut beat: u8 = 0;
    static mut stim: hal::stm32::ITM = ();
    static mut led1: hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>> = ();
    static mut tmr2: hal::timer::Timer<stm32::TIM2> = ();
    static mut tmr3: hal::timer::Timer<stm32::TIM3> = ();

    #[init]
    fn init() {
        let mut rcc = device.RCC.constrain();

        // configure clocks
        let mut flash = device.FLASH.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(8.mhz())
            .pclk1(8.mhz())
            .freeze(&mut flash.acr);

        // configure PC13 pin to blink LED
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);
        let l1 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        // configure and start TIM2 periodic timer
        let mut t2 = Timer::tim2(device.TIM2, 1.hz(), clocks, &mut rcc.apb1);
        t2.listen(Event::Update);

        // configure and start TIM3 periodic timer
        let mut t3 = Timer::tim3(device.TIM3, 5.hz(), clocks, &mut rcc.apb1);
        t3.listen(Event::Update);

        led1 = l1;
        tmr2 = t2;
        tmr3 = t3;
        stim = core.ITM;
    }

    #[idle]
    fn idle() -> ! {
        loop {
            cm::asm::wfi();
        }
    }

    #[interrupt(resources = [beat, tmr2, stim])]
    fn TIM2() {
        let dbg = &mut resources.stim.stim[0];
        iprintln!(dbg, "TIM2 beat = {}", *resources.beat);

        *resources.beat += 1;
        resources.tmr2.start(1.hz());
    }

    #[interrupt(resources = [led1, tmr3])]
    fn TIM3() {
        resources.led1.toggle();
        resources.tmr3.start(5.hz());
    }
};
