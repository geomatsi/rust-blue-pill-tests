#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m as cm;
use hal::adc;
use hal::adc::Adc;
use hal::gpio::gpioa::{PA0, PA1};
use hal::gpio::Analog;
use hal::prelude::*;
use hal::stm32::ADC1;
use panic_rtt_target as _;
use rtic::app;
use rtic::cyccnt::Instant;
use rtic::cyccnt::U32Ext;
use rtt_target::rprintln;
use rtt_target::rtt_init_print;
use shared_bus::AdcProxy;
use shared_bus::CortexMMutex;
use stm32f1xx_hal as hal;

const PERIOD1: u32 = 24_000_000;
const PERIOD2: u32 = 12_000_000;

#[app(device = stm32f1xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        // late resources
        adc_proxy1: AdcProxy<'static, CortexMMutex<Adc<ADC1>>>,
        adc_proxy2: AdcProxy<'static, CortexMMutex<Adc<ADC1>>>,
        adc_ch1: PA0<Analog>,
        adc_ch2: PA1<Analog>,
    }

    #[init(schedule = [task1, task2])]
    fn init(mut cx: init::Context) -> init::LateResources {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(32.mhz())
            .pclk1(16.mhz())
            .adcclk(8.mhz())
            .freeze(&mut flash.acr);

        // init logging
        rtt_init_print!();

        // setup ADC
        let adc = adc::Adc::adc1(cx.device.ADC1, &mut rcc.apb2, clocks);
        let adc_bus: &'static _ = shared_bus::new_cortexm!(Adc<ADC1> = adc).unwrap();
        let adc_proxy1 = adc_bus.acquire_adc();
        let adc_proxy2 = adc_bus.acquire_adc();

        // setup GPIOA
        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);
        let adc_ch1 = gpioa.pa0.into_analog(&mut gpioa.crl);
        let adc_ch2 = gpioa.pa1.into_analog(&mut gpioa.crl);

        /* Enable the monotonic timer based on CYCCNT */
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();

        cx.schedule
            .task1(Instant::now() + PERIOD1.cycles())
            .unwrap();

        cx.schedule
            .task2(Instant::now() + PERIOD2.cycles())
            .unwrap();

        init::LateResources {
            adc_ch1,
            adc_ch2,
            adc_proxy1,
            adc_proxy2,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cm::asm::nop();
        }
    }

    #[task(schedule = [task1], resources = [adc_proxy1, adc_ch1])]
    fn task1(cx: task1::Context) {
        let val: u16 = cx.resources.adc_proxy1.read(cx.resources.adc_ch1).unwrap();
        rprintln!("reading1: {}", val);
        cx.schedule
            .task1(Instant::now() + PERIOD1.cycles())
            .unwrap();
    }

    #[task(schedule = [task2], resources = [adc_proxy2, adc_ch2])]
    fn task2(cx: task2::Context) {
        let val: u16 = cx.resources.adc_proxy2.read(cx.resources.adc_ch2).unwrap();
        rprintln!("reading2: {}", val);
        cx.schedule
            .task2(Instant::now() + PERIOD2.cycles())
            .unwrap();
    }

    // needed for RTFM timer queue and task management
    extern "C" {
        fn EXTI2();
    }
};
