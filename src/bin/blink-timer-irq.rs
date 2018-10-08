#![feature(extern_prelude)]
#![no_main]
#![no_std]

extern crate cortex_m_rt as rt;
use rt::entry;
use rt::exception;
use rt::ExceptionFrame;

extern crate cortex_m as cm;

extern crate cortex_m_semihosting as sh;
use sh::hio;
use sh::hio::HStdout;

extern crate panic_semihosting;

extern crate stm32f103xx_hal as hal;
use hal::prelude::*;
use hal::stm32f103xx;
use hal::timer::Event;
use hal::timer::Timer;

use core::fmt::Write;

type LedT = hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>;
type TimT = hal::timer::Timer<stm32f103xx::TIM3>;

static mut G_LED: Option<LedT> = None;
static mut G_TMR: Option<TimT> = None;

#[entry]
fn main() -> ! {
    let mut cp = cm::peripheral::Peripherals::take().unwrap();
    let dp = stm32f103xx::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    // configure NVIC interrupts
    setup_interrupts(&mut cp);

    // configure clocks
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.mhz())
        .pclk1(8.mhz())
        .freeze(&mut flash.acr);

    // configure PC13 pin to blink LED
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // configure and start TIM3 periodic timer
    let mut tmr = Timer::tim3(dp.TIM3, 1.hz(), clocks, &mut rcc.apb1);
    tmr.listen(Event::Update);

    unsafe {
        G_LED = Some(led);
        G_TMR = Some(tmr);
    }

    loop {
        cm::asm::nop();
    }
}

fn setup_interrupts(cp: &mut cm::peripheral::Peripherals) {
    let nvic = &mut cp.NVIC;

    // Enable TIM3 IRQ, set prio 1 and clear any pending IRQs
    nvic.enable(stm32f103xx::Interrupt::TIM3);
    nvic.clear_pending(stm32f103xx::Interrupt::TIM3);

    unsafe {
        nvic.set_priority(stm32f103xx::Interrupt::TIM3, 1);
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

stm32f103xx::interrupt!(TIM3, timer_tim3, state: Option<HStdout> = None);
fn timer_tim3(state: &mut Option<HStdout>) {
    if state.is_none() {
        *state = Some(hio::hstdout().unwrap());
    }

    if let Some(hstdout) = state.as_mut() {
        writeln!(hstdout, "BLINK").unwrap();
    }

    let led = unsafe { G_LED.as_mut().unwrap() };
    let tim = unsafe { G_TMR.as_mut().unwrap() };

    led.toggle();
    tim.start(1.hz());
}
