#![no_main]
#![no_std]

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
use rt::ExceptionFrame;

extern crate cortex_m as cm;
use cm::interrupt::Mutex;

extern crate cortex_m_semihosting as sh;
use sh::hio;
use sh::hio::HStdout;

extern crate panic_semihosting;

extern crate stm32f103xx_hal as hal;
use hal::prelude::*;
use hal::gpio::*;
use hal::stm32f103xx::*;
use hal::timer::Timer;
use hal::timer::Event;

use core::cell::RefCell;
use core::ops::DerefMut;
use core::fmt::Write;

entry!(main);

fn main() -> ! {
    // configure TIM3 interrupt
    let cp = cm::peripheral::Peripherals::take().unwrap();
    let mut nvic = cp.NVIC;
    nvic.enable(Interrupt::TIM3);
    unsafe {
        nvic.set_priority(Interrupt::TIM3, 1);
    }
    nvic.clear_pending(Interrupt::TIM3);

    // configure peripherals
    let dp = Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr
        .sysclk(8.mhz())
        .pclk1(8.mhz())
        .freeze(&mut flash.acr);

    let mut tmr = Timer::tim3(dp.TIM3, 1.hz(), clocks, &mut rcc.apb1);
    tmr.listen(Event::Update);

    loop {
        cm::asm::nop();
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

interrupt!(TIM3, timer_tim3, state: Option<HStdout> = None);

fn timer_tim3(state: &mut Option<HStdout>) {
    if state.is_none() {
        *state = Some(hio::hstdout().unwrap());
    }

    if let Some(hstdout) = state.as_mut() {
        writeln!(hstdout, "TIM3 FIRE").unwrap();
    }

    unsafe {
        (*GPIOC::ptr())
            .odr
            .modify(|r, w| w.odr13().bit(!r.odr13().bit()));

        (*TIM3::ptr())
            .sr
            .modify(|_, w| w.uif().clear_bit());

    }
}
