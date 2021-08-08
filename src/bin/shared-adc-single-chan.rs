#![no_std]
#![no_main]

use cortex_m_rt as rt;
use cortex_m_semihosting::hprintln;
use embedded_hal::adc::Channel;
use embedded_hal::adc::OneShot;
use panic_semihosting as _;
use rt::entry;
use stm32f1xx_hal::stm32::ADC1;
use stm32f1xx_hal::{adc::Adc, prelude::*, stm32};

pub struct Measurement<'a, ADC, PIN>
where
    PIN: Channel<ADC, ID = u8>,
{
    adc: &'a mut dyn OneShot<ADC, u16, PIN, Error = ()>,
    pin: PIN,
}

impl<'a, ADC, PIN> Measurement<'a, ADC, PIN>
where
    PIN: Channel<ADC, ID = u8>,
{
    pub fn init(adc: &'a mut dyn OneShot<ADC, u16, PIN, Error = ()>, pin: PIN) -> Self {
        Measurement { adc, pin }
    }

    pub fn test(&mut self) -> u16 {
        self.adc.read(&mut self.pin).unwrap()
    }
}

#[entry]
fn main() -> ! {
    let p = stm32::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.adcclk(2.mhz()).freeze(&mut flash.acr);

    // gpio input channels
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);
    let ch1 = gpioa.pa1.into_analog(&mut gpioa.crl);

    let adc = Adc::adc1(p.ADC1, &mut rcc.apb2, clocks);

    let adc_bus: &'static _ = shared_bus::new_cortexm!(Adc<ADC1> = adc).unwrap();

    let mut adc_mgr1 = adc_bus.acquire_adc();
    let mut adc_mgr2 = adc_bus.acquire_adc();

    let mut a = Measurement::init(&mut adc_mgr1, ch0);
    let mut b = Measurement::init(&mut adc_mgr2, ch1);

    loop {
        hprintln!("reading a: {}", a.test()).unwrap();
        hprintln!("reading a: {}", b.test()).unwrap();
    }
}
