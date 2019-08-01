#![allow(deprecated)]
#![no_std]
#![no_main]

use embedded_hal::adc::Channel;
use embedded_hal::adc::OneShot;

use cortex_m_rt as rt;
use rt::entry;

use cortex_m as cm;
use panic_itm as _;

use shared_bus;
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

    let adc = Adc::adc1(p.ADC1, &mut rcc.apb2, clocks);
    let adc_bus = shared_bus::BusManager::<
        cm::interrupt::Mutex<core::cell::RefCell<AdcType>>,
        AdcType,
    >::new(adc);

    let mut adc_mgr1 = adc_bus.acquire();
    let mut adc_mgr2 = adc_bus.acquire();

    let mut a = Measurement::init(&mut adc_mgr1, ch0);
    let mut b = Measurement::init(&mut adc_mgr2, ch1);

    loop {
        a.test();
        b.test();
    }
}
