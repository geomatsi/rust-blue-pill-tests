#![no_main]
#![no_std]

extern crate cortex_m_rt as rt;
use rt::entry;

extern crate cortex_m as cm;
use cm::asm::wfi;
use cm::interrupt::Mutex;

extern crate cortex_m_semihosting as sh;
use sh::hprintln;

extern crate panic_semihosting;

extern crate stm32f1xx_hal as hal;
use core::cell::RefCell;
use core::ops::DerefMut;
use hal::gpio::gpioc::PC13;
use hal::gpio::{Output, PushPull};
use hal::prelude::*;
use hal::stm32;
use hal::stm32::{interrupt, TIM2};
use hal::timer::Event;
use hal::timer::Timer;

static G_LED: Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static G_TIM: Mutex<RefCell<Option<Timer<TIM2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut cp = cm::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let (tim, led) = init_periph(&mut cp, dp);

    cm::interrupt::free(|cs| {
        G_TIM.borrow(cs).replace(Some(tim));
        G_LED.borrow(cs).replace(Some(led));
    });

    loop {
        hprintln!("MAIN LOOP").unwrap();
        wfi();
    }
}

fn init_periph(
    cp: &mut cm::peripheral::Peripherals,
    dp: stm32::Peripherals,
) -> (Timer<TIM2>, PC13<Output<PushPull>>) {
    let nvic = &mut cp.NVIC;

    nvic.enable(stm32::Interrupt::TIM2);
    cm::peripheral::NVIC::unpend(stm32::Interrupt::TIM2);

    unsafe {
        nvic.set_priority(stm32::Interrupt::TIM2, 1);
    }

    // configure clocks
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.mhz())
        .pclk1(8.mhz())
        .freeze(&mut flash.acr);

    // configure PC13 pin to blink LED
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);
    let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // configure and start periodic GP timers
    let mut tim = Timer::tim2(dp.TIM2, 1.hz(), clocks, &mut rcc.apb1);

    tim.listen(Event::Update);

    (tim, led)
}

#[interrupt]
fn TIM2() {
    cm::interrupt::free(|cs| {
        if let (Some(ref mut tim), Some(ref mut led)) = (
            G_TIM.borrow(cs).borrow_mut().deref_mut(),
            G_LED.borrow(cs).borrow_mut().deref_mut(),
        ) {
            hprintln!("TIM2 IRQ").unwrap();
            led.toggle();
            tim.start(1.hz());
        }
    });
}