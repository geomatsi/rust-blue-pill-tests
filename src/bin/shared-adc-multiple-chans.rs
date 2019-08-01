#![allow(deprecated)]
#![no_main]
#![no_std]

use embedded_hal::adc::Channel;
use embedded_hal::adc::OneShot;

use cortex_m_rt as rt;
use rt::entry;

use cortex_m as cm;
use panic_itm as _;

use shared_bus;
use stm32f1xx_hal::gpio::{gpioa, Analog};
use stm32f1xx_hal::{adc::Adc, prelude::*, stm32};

pub struct Measurement<'a, ADC, PIN>
where
    PIN: Channel<ADC, ID = u8>,
{
    adc: &'a mut dyn OneShot<ADC, u16, PIN, Error = ()>,
    pin1: &'a mut dyn Channel<ADC, ID = u8>,
    pin2: &'a mut dyn Channel<ADC, ID = u8>,
}

impl<'a, ADC, PIN> Measurement<'a, ADC, PIN>
where
    PIN: Channel<ADC, ID = u8>,
{
    pub fn init(adc: &'a mut dyn OneShot<ADC, u16, PIN, Error = ()>, pin1: &'a mut dyn Channel<ADC, ID = u8>, pin2: &'a mut dyn Channel<ADC, ID = u8>) -> Self {
        Measurement { adc, pin1, pin2 }
    }

    pub fn test1(&mut self) -> u16 {
        self.adc.read(self.pin1).unwrap()
    }

    pub fn test2(&mut self) -> u16 {
        self.adc.read(self.pin2).unwrap()
    }
}

type AdcType = Adc<stm32::ADC1>;

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
    let ch2 = gpioa.pa2.into_analog(&mut gpioa.crl);
    let ch3 = gpioa.pa3.into_analog(&mut gpioa.crl);

    let adc = Adc::adc1(p.ADC1, &mut rcc.apb2, clocks);
    let adc_bus = shared_bus::BusManager::<
        cm::interrupt::Mutex<core::cell::RefCell<AdcType>>,
        AdcType,
    >::new(adc);

    let mut adc_mgr1 = adc_bus.acquire();
    let mut adc_mgr2 = adc_bus.acquire();

    let mut a = Measurement::init(&mut adc_mgr1, &mut ch0, &mut ch1);
    let mut b = Measurement::init(&mut adc_mgr2, &mut ch2, &mut ch3);

    loop {
        a.test1();
        b.test2();
    }
}
