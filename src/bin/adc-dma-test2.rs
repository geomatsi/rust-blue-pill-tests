#![no_main]
#![no_std]

use cm::singleton;
use cortex_m as cm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use hal::adc;
use hal::gpio;
use hal::pac;
use hal::prelude::*;
use panic_semihosting as _;
use stm32f1xx_hal as hal;
use stm32f1xx_hal::adc::Continuous;

static mut G_ADC: Option<adc::AdcDma<gpio::gpioa::PA0<gpio::Analog>, Continuous>> = None;
static mut G_BUF: Option<&'static mut [u16; 8]> = None;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.adcclk(2.mhz()).freeze(&mut flash.acr);

    // dma channel #1
    let dma_ch1 = dp.DMA1.split(&mut rcc.ahb).1;

    // setup ADC
    let adc1 = adc::Adc::adc1(dp.ADC1, &mut rcc.apb2, clocks);

    // setup GPIOA
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // Configure pa0 as an analog input
    let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

    let mut adc_dma = adc1.with_dma(adc_ch0, dma_ch1);
    let mut buf = singleton!(: [u16; 8] = [0; 8]).unwrap();

    unsafe {
        G_ADC.replace(adc_dma);
        G_BUF.replace(buf);
    }

    loop {
        adc_dma = unsafe { G_ADC.take().unwrap() };
        buf = unsafe { G_BUF.take().unwrap() };

        let (buf1, adc_dma1) = adc_dma.read(buf).wait();
        hprintln!("{:#?} ", buf1).unwrap();

        unsafe {
            G_ADC.replace(adc_dma1);
            G_BUF.replace(buf1);
        }
    }
}
