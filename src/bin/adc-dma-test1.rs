#![no_main]
#![no_std]

use cortex_m::singleton;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use hal::adc;
use hal::gpio;
use hal::pac;
use hal::prelude::*;
use panic_semihosting as _;
use stm32f1xx_hal as hal;
use stm32f1xx_hal::adc::Continuous;

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

    let adc_dma = adc1.with_dma(adc_ch0, dma_ch1);
    let buf = singleton!(: [u16; 8] = [0; 8]).unwrap();

    /*
     * Consider the following simple loop:
     * loop {
     *     let (buf, adc_dma) = adc_dma.read(buf).wait();
     *     hprintln!("{:#?}", buf).unwrap();
     * }
     *
     * This approach does not work due to the following error:
     *     ^^^^^^^ value moved here, in previous iteration of loop
     *
     * Here is a simple workaround for the problem: recursive read_loop function.
     * BTW, it might have some problems with stack overflow due to recursion.
     *
     */

    read_loop(buf, adc_dma);
}

#[allow(unconditional_recursion)]
fn read_loop(
    buf: &'static mut [u16; 8],
    adc_dma: adc::AdcDma<gpio::gpioa::PA0<gpio::Analog>, Continuous>,
) -> ! {
    let (buf, adc_dma) = adc_dma.read(buf).wait();
    hprintln!("{:#?} ", buf).unwrap();
    read_loop(buf, adc_dma);
}
