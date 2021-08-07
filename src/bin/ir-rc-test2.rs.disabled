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

/* IR receiver for something like 1-wire protocol */

#[derive(Debug, Clone, Copy)]
pub enum ReceiverState {
    Waiting,
    Recv,
    Recv0,
    Recv1,
    Done,
    Error,
}

#[derive(Debug, Clone, Copy)]
pub enum ReceiverResult {
    Proc,
    Done(u8),
    Fail(u8),
}

struct TestReceiver {
    state: ReceiverState,
    curr_stamp: u32,
    prev_stamp: u32,
    rate: u32,
    pin: bool,
    res: u8,
    num: u8,
}

impl TestReceiver {
    pub fn new(sample_rate: u32) -> Self {
        TestReceiver {
            state: ReceiverState::Waiting,
            rate: sample_rate,
            curr_stamp: 0,
            prev_stamp: 0,
            pin: true,
            res: 0u8,
            num: 0u8,
        }
    }

    pub fn sample(&mut self, new_pin: bool, stamp: u32) -> ReceiverResult {
        if new_pin == self.pin {
            return ReceiverResult::Proc;
        }

        // FIXME: (curr - prev) in usec
        let mut delta = stamp.wrapping_sub(self.prev_stamp);
        delta = delta * (1_000_000 / self.rate);

        // new_val = 1 => val = 0 = > rising
        let rising = new_pin;

        let prev_stamp = self.prev_stamp;
        self.prev_stamp = stamp;
        self.pin = new_pin;

        match self.state {
            ReceiverState::Waiting => {
                if rising {
                    self.state = ReceiverState::Error;
                    return ReceiverResult::Fail(1);
                }

                if prev_stamp != 0 && delta < 4000 {
                    self.state = ReceiverState::Error;
                    return ReceiverResult::Fail(2);
                }

                self.state = ReceiverState::Recv;
                ReceiverResult::Proc
            }
            ReceiverState::Recv => {
                if !rising {
                    self.state = ReceiverState::Error;
                    return ReceiverResult::Fail(3);
                }

                // FIXME
                if delta > (800 - 400) && delta < (800 + 400) {
                    if self.num == 7 {
                        self.state = ReceiverState::Done;
                        return ReceiverResult::Done(self.res);
                    }
                    self.state = ReceiverState::Recv1;
                    return ReceiverResult::Proc;
                }

                // FIXME
                if delta > (2500 - 1000) && delta < (2500 + 1000) {
                    if self.num == 7 {
                        self.state = ReceiverState::Done;
                        return ReceiverResult::Done(self.res);
                    }
                    self.state = ReceiverState::Recv0;
                    return ReceiverResult::Proc;
                }

                hprintln!("num={} delta={}", self.num, delta).unwrap();
                self.state = ReceiverState::Error;
                ReceiverResult::Fail(4)
            }
            ReceiverState::Recv0 => {
                if rising {
                    self.state = ReceiverState::Error;
                    return ReceiverResult::Fail(5);
                }

                // FIXME
                if delta > (800 - 400) && delta < (800 + 400) {
                    self.res |= 0 << self.num;
                    self.num += 1;

                    self.state = ReceiverState::Recv;
                    return ReceiverResult::Proc;
                }

                hprintln!("num={} delta={}", self.num, delta).unwrap();
                self.state = ReceiverState::Error;
                ReceiverResult::Fail(6)
            }
            ReceiverState::Recv1 => {
                if rising {
                    self.state = ReceiverState::Error;
                    return ReceiverResult::Fail(7);
                }

                if delta > (2500 - 1000) && delta < (2500 + 1000) {
                    self.res |= 1 << self.num;
                    self.num += 1;

                    self.state = ReceiverState::Recv;
                    return ReceiverResult::Proc;
                }

                hprintln!("num={} delta={}", self.num, delta).unwrap();
                self.state = ReceiverState::Error;
                ReceiverResult::Fail(8)
            }
            ReceiverState::Error => {
                hprintln!("num={} delta={}", self.num, delta).unwrap();
                ReceiverResult::Fail(9)
            }
            _ => unreachable!(),
        }
    }

    pub fn reset(&mut self) {
        self.state = ReceiverState::Waiting;
        self.curr_stamp = 0;
        self.prev_stamp = 0;
        self.pin = true;
        self.res = 0u8;
        self.num = 0u8;
    }
}

/* */

const FREQ: u32 = 50000;

static G_TIM: Mutex<RefCell<Option<CountDownTimer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));
static G_DEC: Mutex<RefCell<Option<TestReceiver>>> = Mutex::new(RefCell::new(None));
static G_IRR: Mutex<RefCell<Option<PB0<Input<Floating>>>>> = Mutex::new(RefCell::new(None));
static G_CNT: Mutex<RefCell<Option<u32>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(mut cp), Some(dp)) = (cm::Peripherals::take(), stm32::Peripherals::take()) {
        cm::interrupt::free(|cs| {
            let mut rcc = dp.RCC.constrain();
            let mut flash = dp.FLASH.constrain();

            //let clocks = rcc.cfgr.sysclk(8.mhz()).pclk1(8.mhz()).freeze(&mut flash.acr);

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

            let dec = TestReceiver::new(FREQ);

            G_DEC.borrow(cs).replace(Some(dec));
            G_TIM.borrow(cs).replace(Some(tmr));
            G_IRR.borrow(cs).replace(Some(irr));
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
        if let (Some(ref mut tim), Some(ref mut pin), Some(ref mut dec), Some(ref mut cnt)) = (
            G_TIM.borrow(cs).borrow_mut().deref_mut(),
            G_IRR.borrow(cs).borrow_mut().deref_mut(),
            G_DEC.borrow(cs).borrow_mut().deref_mut(),
            G_CNT.borrow(cs).borrow_mut().deref_mut(),
        ) {
            let val = pin.is_high().unwrap();

            tim.clear_update_interrupt_flag();

            match dec.sample(val, *cnt) {
                ReceiverResult::Done(v) => {
                    hprintln!("result 0x{:x}", v).unwrap();
                    dec.reset();
                }
                ReceiverResult::Fail(e) => {
                    hprintln!("error {}", e).unwrap();
                    dec.reset();
                }
                ReceiverResult::Proc => {}
            }

            *cnt = cnt.wrapping_add(1);
        }
    });
}
