#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m as cm;
use cm::iprintln;

extern crate rtfm;
use rtfm::app;

extern crate panic_itm;

extern crate stm32f1xx_hal as hal;
use hal::delay::Delay;
use hal::prelude::*;
use hal::stm32;
use hal::time::Instant;
use hal::time::MonoTimer;
use hal::timer::Event;
use hal::timer::Timer;

extern crate hc_sr04;
use hc_sr04::{Error, HcSr04};

#[app(device = hal::stm32)]
const APP: () = {
    static mut SENSOR: HcSr04<
        hal::gpio::gpioa::PA1<hal::gpio::Output<hal::gpio::PushPull>>,
        Delay,
    > = ();
    static mut TIMER: MonoTimer = ();
    static mut EXTI: hal::stm32::EXTI = ();
    static mut LED: hal::gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>> = ();
    static mut ITM: hal::stm32::ITM = ();
    static mut TMR: hal::timer::Timer<stm32::TIM3> = ();
    static mut TS: Option<Instant> = None;

    #[init]
    fn init() {
        // configure clocks
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(8.mhz())
            .pclk1(8.mhz())
            .freeze(&mut flash.acr);

        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

        let delay = Delay::new(core.SYST, clocks);
        let trace = hal::time::enable_trace(core.DCB);
        let monotimer = MonoTimer::new(core.DWT, trace, clocks);

        // configure PC13 pin to blink LED
        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        // configure PA0 pin to capture echo
        gpioa.pa0.into_floating_input(&mut gpioa.crl);

        // configure PA1 pin to trigger pulse
        let trig = gpioa.pa1.into_push_pull_output(&mut gpioa.crl);

        // configure and start TIM3 periodic timer
        let tmr = Timer::tim3(device.TIM3, 1.hz(), clocks, &mut rcc.apb1);

        // setup EXTI0 interrupt: pin PA0
        device.EXTI.imr.write(|w| w.mr0().set_bit());
        device.EXTI.ftsr.write(|w| w.tr0().set_bit());
        device.EXTI.rtsr.write(|w| w.tr0().set_bit());

        // create sensor
        let sensor = HcSr04::new(trig, delay, monotimer.frequency().0);

        SENSOR = sensor;
        LED = led;
        EXTI = device.EXTI;
        TIMER = monotimer;
        TMR = tmr;
        ITM = core.ITM;
    }

    #[idle(resources = [TMR, SENSOR, LED, ITM])]
    fn idle() -> ! {
        loop {
            resources.TMR.lock(|t| {
                t.start(1.hz());
                t.listen(Event::Update);
            });

            loop {
                let dist = resources.SENSOR.lock(|s| s.distance());
                match dist {
                    Ok(dist) => {
                        resources.TMR.lock(|t| {
                            t.unlisten(Event::Update);
                        });

                        match dist {
                            Some(dist) => {
                                let cm = dist.cm();
                                resources.ITM.lock(|t| {
                                    iprintln!(&mut t.stim[0], "{:?}", cm);
                                });
                                break;
                            }
                            None => {
                                resources.ITM.lock(|t| {
                                    iprintln!(&mut t.stim[0], "Err");
                                });
                                break;
                            }
                        }
                    }
                    Err(Error::WouldBlock) => {
                        cm::asm::wfi();
                    }
                    Err(_) => unreachable!(),
                }
            }

            for _ in 0..10000 {
                cm::asm::nop();
            }
        }
    }

    #[interrupt(resources = [TIMER, ITM, SENSOR, TS, EXTI])]
    fn EXTI0() {
        let dbg = &mut resources.ITM.stim[0];

        match *resources.TS {
            Some(ts) => {
                let delta = ts.elapsed();
                *resources.TS = None;
                iprintln!(dbg, "stop capture: {:?}", delta);

                resources
                    .SENSOR
                    .capture(delta)
                    .expect("echo handler: sensor in wrong state!");
            }
            None => {
                *resources.TS = Some(resources.TIMER.now());
                iprintln!(dbg, "start capture");

                resources
                    .SENSOR
                    .capture(0)
                    .expect("echo handler: sensor in wrong state!");
            }
        }

        resources.EXTI.pr.write(|w| w.pr0().set_bit());
    }

    #[interrupt(resources = [TMR, ITM, SENSOR, TS])]
    fn TIM3() {
        let dbg = &mut resources.ITM.stim[0];

        iprintln!(dbg, "timeout");
        resources
            .SENSOR
            .timedout()
            .expect("timeout handler: sensor in wrong state!");
        resources.TMR.unlisten(Event::Update);
        *resources.TS = None;
    }
};
