#![no_main]
#![no_std]

use embedded_hal::digital::v2::InputPin;

use cm::interrupt::Mutex;
use cortex_m as cm;

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m_rt::entry;

use hal::gpio::gpiob::PB0;
use hal::gpio::Floating;
use hal::gpio::Input;

use hal::prelude::*;
use hal::stm32;
use hal::stm32::interrupt;
use hal::timer::CountDownTimer;
use hal::timer::Event;
use hal::timer::Timer;
use stm32f1xx_hal as hal;

use infrared::nec::*;
use infrared::rc5::*;
use infrared::rc6::*;
use infrared::Receiver;
use infrared::ReceiverState;

const FREQ: u32 = 40000;

static G_TIM: Mutex<RefCell<Option<CountDownTimer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));

static G_NES: Mutex<RefCell<Option<NecSamsungReceiver>>> = Mutex::new(RefCell::new(None));
static G_NEC: Mutex<RefCell<Option<NecReceiver>>> = Mutex::new(RefCell::new(None));
static G_RC5: Mutex<RefCell<Option<Rc5Receiver>>> = Mutex::new(RefCell::new(None));
static G_RC6: Mutex<RefCell<Option<Rc6Receiver>>> = Mutex::new(RefCell::new(None));

static G_IRR: Mutex<RefCell<Option<PB0<Input<Floating>>>>> = Mutex::new(RefCell::new(None));

static G_PIN: Mutex<RefCell<Option<bool>>> = Mutex::new(RefCell::new(None));
static G_CNT: Mutex<RefCell<Option<u32>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(mut cp), Some(dp)) = (cm::Peripherals::take(), stm32::Peripherals::take()) {
        cm::interrupt::free(|cs| {
            let mut rcc = dp.RCC.constrain();
            let mut flash = dp.FLASH.constrain();

            let clocks = rcc
                .cfgr
                .use_hse(8.mhz())
                .sysclk(48.mhz())
                .pclk1(24.mhz())
                .freeze(&mut flash.acr);

            hprintln!("SYSCLK: {} Hz ...", clocks.sysclk().0).unwrap();
            hprintln!("PCLK: {} Hz ...", clocks.pclk1().0).unwrap();

            // IR diode: signal connected to PB0
            let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
            let irr = gpiob.pb0.into_floating_input(&mut gpiob.crl);

            setup_interrupts(&mut cp);
            let mut tmr = Timer::tim2(dp.TIM2, &clocks, &mut rcc.apb1).start_count_down(FREQ.hz());
            tmr.listen(Event::Update);

            let nes = NecSamsungReceiver::new(FREQ);
            let nec = NecReceiver::new(FREQ);
            let rc5 = Rc5Receiver::new(FREQ);
            let rc6 = Rc6Receiver::new(FREQ);

            G_NES.borrow(cs).replace(Some(nes));
            G_NEC.borrow(cs).replace(Some(nec));
            G_RC5.borrow(cs).replace(Some(rc5));
            G_RC6.borrow(cs).replace(Some(rc6));
            G_TIM.borrow(cs).replace(Some(tmr));
            G_IRR.borrow(cs).replace(Some(irr));
            G_PIN.borrow(cs).replace(Some(false));
            G_CNT.borrow(cs).replace(Some(0));
        });
    }

    loop {
        cm::asm::nop();
    }
}

fn setup_interrupts(cp: &mut cm::peripheral::Peripherals) {
    let nvic = &mut cp.NVIC;

    unsafe {
        nvic.set_priority(stm32::Interrupt::TIM2, 1);
        cm::peripheral::NVIC::unmask(stm32::Interrupt::TIM2);
    }

    cm::peripheral::NVIC::unpend(stm32::Interrupt::TIM2);
}

#[interrupt]
fn TIM2() {
    cm::interrupt::free(|cs| {
        if let (
            Some(ref mut tim),
            Some(ref mut pin),
            Some(ref mut nes),
            Some(ref mut nec),
            Some(ref mut rc5),
            Some(ref mut rc6),
            Some(ref mut val),
            Some(ref mut cnt),
        ) = (
            G_TIM.borrow(cs).borrow_mut().deref_mut(),
            G_IRR.borrow(cs).borrow_mut().deref_mut(),
            G_NES.borrow(cs).borrow_mut().deref_mut(),
            G_NEC.borrow(cs).borrow_mut().deref_mut(),
            G_RC5.borrow(cs).borrow_mut().deref_mut(),
            G_RC6.borrow(cs).borrow_mut().deref_mut(),
            G_PIN.borrow(cs).borrow_mut().deref_mut(),
            G_CNT.borrow(cs).borrow_mut().deref_mut(),
        ) {
            let new_val = pin.is_low().unwrap();

            tim.clear_update_interrupt_flag();

            if *val != new_val {
                let rising = new_val;

                if let Some(cmd) = sample_on_edge(nec, rising, *cnt) {
                    hprintln!("{:?}", cmd).unwrap();
                    nec.reset();
                }

                if let Some(cmd) = sample_on_edge(nes, rising, *cnt) {
                    hprintln!("{:?}", cmd).unwrap();
                    nes.reset();
                }

                if let Some(cmd) = sample_on_edge(rc5, rising, *cnt) {
                    hprintln!("{:?}", cmd).unwrap();
                    rc5.reset();
                }

                if let Some(cmd) = sample_on_edge(rc6, rising, *cnt) {
                    hprintln!("{:?}", cmd).unwrap();
                    rc6.reset();
                }
            }

            *cnt = cnt.wrapping_add(1);
            *val = new_val;
        }
    });
}

fn sample_on_edge<CMD, ERR>(
    recv: &mut dyn Receiver<Cmd = CMD, Err = ERR>,
    edge: bool,
    t: u32,
) -> Option<CMD> {
    match recv.sample_edge(edge, t) {
        ReceiverState::Idle => {
            return None;
        }
        ReceiverState::Receiving => {
            return None;
        }
        ReceiverState::Disabled => {
            return None;
        }
        ReceiverState::Done(c) => {
            return Some(c);
        }
        ReceiverState::Error(_err) => {
            hprintln!("ERR").unwrap();
            recv.reset();
            return None;
        }
    }
}
