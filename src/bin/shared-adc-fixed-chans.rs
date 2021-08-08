#![no_main]
#![no_std]

use cortex_m_rt as rt;
use rt::entry;

use cortex_m_semihosting::hprintln;
use hal::adc::Adc;
use hal::prelude::*;
use hal::stm32;
use hal::stm32::ADC1;
use hal::timer::Timer;
use nb::block;
use panic_semihosting as _;
use shared_bus::AdcProxy;
use shared_bus::CortexMMutex;
use stm32f1xx_hal as hal;

/* */

type Chan0 = hal::gpio::gpioa::PA0<hal::gpio::Analog>;
type Chan1 = hal::gpio::gpioa::PA1<hal::gpio::Analog>;
type Chan2 = hal::gpio::gpioa::PA2<hal::gpio::Analog>;
type Chan3 = hal::gpio::gpioa::PA3<hal::gpio::Analog>;
type Chan4 = hal::gpio::gpioa::PA4<hal::gpio::Analog>;

pub struct M1<'a> {
    adc: AdcProxy<'a, CortexMMutex<Adc<ADC1>>>,
    ch0: Chan0,
    ch1: Chan1,
}

impl<'a> M1<'a> {
    pub fn init(adc: AdcProxy<'a, CortexMMutex<Adc<ADC1>>>, ch0: Chan0, ch1: Chan1) -> Self {
        M1 { adc, ch0, ch1 }
    }

    pub fn get_ch0(&mut self) -> u16 {
        self.adc.read(&mut self.ch0).unwrap()
    }

    pub fn get_ch1(&mut self) -> u16 {
        self.adc.read(&mut self.ch1).unwrap()
    }
}

pub struct M2<'a> {
    adc: AdcProxy<'a, CortexMMutex<Adc<ADC1>>>,
    ch2: Chan2,
    ch3: Chan3,
    ch4: Chan4,
}

impl<'a> M2<'a> {
    pub fn init(
        adc: AdcProxy<'a, CortexMMutex<Adc<ADC1>>>,
        ch2: Chan2,
        ch3: Chan3,
        ch4: Chan4,
    ) -> Self {
        M2 { adc, ch2, ch3, ch4 }
    }

    pub fn get_ch2(&mut self) -> u16 {
        self.adc.read(&mut self.ch2).unwrap()
    }

    pub fn get_ch3(&mut self) -> u16 {
        self.adc.read(&mut self.ch3).unwrap()
    }

    pub fn get_ch4(&mut self) -> u16 {
        self.adc.read(&mut self.ch4).unwrap()
    }
}

#[entry]
fn main() -> ! {
    let p = stm32::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.mhz())
        .pclk1(8.mhz())
        .adcclk(2.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);
    let ch1 = gpioa.pa1.into_analog(&mut gpioa.crl);
    let ch2 = gpioa.pa2.into_analog(&mut gpioa.crl);
    let ch3 = gpioa.pa3.into_analog(&mut gpioa.crl);
    let ch4 = gpioa.pa4.into_analog(&mut gpioa.crl);

    let mut tmr = Timer::tim3(p.TIM3, &clocks, &mut rcc.apb1).start_count_down(1.hz());

    let adc = Adc::adc1(p.ADC1, &mut rcc.apb2, clocks);

    let adc_bus: &'static _ = shared_bus::new_cortexm!(Adc<ADC1> = adc).unwrap();

    let adc_mgr1 = adc_bus.acquire_adc();
    let adc_mgr2 = adc_bus.acquire_adc();

    let mut a = M1::init(adc_mgr1, ch0, ch1);
    let mut b = M2::init(adc_mgr2, ch2, ch3, ch4);

    loop {
        let ch0 = a.get_ch0();
        let ch1 = a.get_ch1();
        hprintln!("readings: {} {}", ch0, ch1).unwrap();

        let ch2 = b.get_ch2();
        let ch3 = b.get_ch3();
        let ch4 = b.get_ch4();
        hprintln!("readings: {} {} {}", ch2, ch3, ch4).unwrap();

        block!(tmr.wait()).ok();
    }
}
