#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt as rt;
use cortex_m_semihosting::hprintln;
use hal::prelude::*;
use hal::stm32;
use hal::stm32::interrupt;
use hal::timer::Event;
use hal::timer::Timer;
use panic_semihosting as _;
use rt::entry;
use stm32f1xx_hal as hal;

type LedT = hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>>;
type TimT = hal::timer::CountDownTimer<stm32::TIM3>;

static mut G_LED: Option<LedT> = None;
static mut G_TMR: Option<TimT> = None;

#[entry]
fn main() -> ! {
    let mut cp = cm::peripheral::Peripherals::take().unwrap();
    let dp = hal::stm32::Peripherals::take().unwrap();
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
    let mut tmr = Timer::tim3(dp.TIM3, &clocks, &mut rcc.apb1).start_count_down(1.hz());
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
    nvic.enable(stm32::Interrupt::TIM3);
    cm::peripheral::NVIC::unpend(stm32::Interrupt::TIM3);

    unsafe {
        nvic.set_priority(stm32::Interrupt::TIM3, 1);
    }
}

#[interrupt]
fn TIM3() {
    hprintln!("BLINK").unwrap();

    let led = unsafe { G_LED.as_mut().unwrap() };
    let tim = unsafe { G_TMR.as_mut().unwrap() };

    led.toggle().unwrap();
    tim.start(1.hz());
}
