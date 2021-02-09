#![no_main]
#![no_std]

use cm::asm::wfi;
use cm::interrupt::Mutex;
use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m as cm;
use cortex_m_rt as rt;
use cortex_m_semihosting::hprintln;
use hal::gpio::gpioc::PC13;
use hal::gpio::{Output, PushPull};
use hal::prelude::*;
use hal::stm32;
use hal::stm32::{interrupt, TIM2};
use hal::timer::CountDownTimer;
use hal::timer::Event;
use hal::timer::Timer;
use panic_semihosting as _;
use rt::entry;
use stm32f1xx_hal as hal;

static G_LED: Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static G_TIM: Mutex<RefCell<Option<CountDownTimer<TIM2>>>> = Mutex::new(RefCell::new(None));

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
) -> (CountDownTimer<TIM2>, PC13<Output<PushPull>>) {
    let nvic = &mut cp.NVIC;

    unsafe {
        nvic.set_priority(stm32::Interrupt::TIM2, 1);
        cm::peripheral::NVIC::unmask(stm32::Interrupt::TIM2);
    }

    cm::peripheral::NVIC::unpend(stm32::Interrupt::TIM2);

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
    let mut tim = Timer::tim2(dp.TIM2, &clocks, &mut rcc.apb1).start_count_down(1.hz());

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
            tim.clear_update_interrupt_flag();
            led.toggle().unwrap();
        }
    });
}
