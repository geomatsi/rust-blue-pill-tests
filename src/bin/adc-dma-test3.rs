#![no_main]
#![no_std]

use cm::interrupt::Mutex;
use cm::singleton;
use core::cell::RefCell;
use cortex_m as cm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use hal::adc;
use hal::delay::Delay;
use hal::dma;
use hal::dma::{Transfer, W};
use hal::gpio;
use hal::pac;
use hal::prelude::*;
use hal::stm32;
use hal::stm32::interrupt;
use panic_semihosting as _;
use stm32f1xx_hal as hal;
use stm32f1xx_hal::adc::Continuous;

type RdmaT = adc::AdcDma<gpio::gpioa::PA0<gpio::Analog>, Continuous>;
type RbufT = &'static mut [u16; 4];

static G_XFR: Mutex<RefCell<Option<Transfer<W, RbufT, RdmaT>>>> = Mutex::new(RefCell::new(None));
static G_DMA: Mutex<RefCell<Option<RdmaT>>> = Mutex::new(RefCell::new(None));
static G_BUF: Mutex<RefCell<Option<RbufT>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let cp = cm::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut nvic = cp.NVIC;

    let clocks = rcc.cfgr.adcclk(2.mhz()).freeze(&mut flash.acr);

    // delay
    let mut delay = Delay::new(cp.SYST, clocks);

    // dma channel #1
    let mut dma_ch1 = dp.DMA1.split(&mut rcc.ahb).1;
    dma_ch1.listen(dma::Event::TransferComplete);

    // setup ADC
    let adc1 = adc::Adc::adc1(dp.ADC1, &mut rcc.apb2, clocks);

    // setup GPIOA
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // Configure pa0 as an analog input
    let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

    let buf = singleton!(: [u16; 4] = [0; 4]).unwrap();

    let adc_dma = adc1.with_dma(adc_ch0, dma_ch1);

    unsafe {
        nvic.set_priority(stm32::Interrupt::DMA1_CHANNEL1, 1);
        cm::peripheral::NVIC::unmask(stm32::Interrupt::DMA1_CHANNEL1);
    }

    cm::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_CHANNEL1);

    let xfer = adc_dma.read(buf);

    cm::interrupt::free(|cs| {
        G_XFR.borrow(cs).replace(Some(xfer));
    });

    loop {
        cm::interrupt::free(|cs| {
            if let (Some(adc_dma), Some(buf)) = (
                G_DMA.borrow(cs).replace(None),
                G_BUF.borrow(cs).replace(None),
            ) {
                hprintln!("IDLE: start next xfer").unwrap();
                let xfer = adc_dma.read(buf);
                G_XFR.borrow(cs).replace(Some(xfer));
            } else {
                hprintln!("IDLE: ERR: no rdma").unwrap();
            }
        });

        hprintln!("IDLE: wait 5 sec").unwrap();
        delay.delay_ms(5_000u16);
    }
}

#[interrupt]
fn DMA1_CHANNEL1() {
    cm::interrupt::free(|cs| {
        if let Some(xfer) = G_XFR.borrow(cs).replace(None) {
            let (buf, adc_dma) = xfer.wait();
            hprintln!("DMA1_CH1 IRQ: results: {:?}", buf).unwrap();
            G_DMA.borrow(cs).replace(Some(adc_dma));
            G_BUF.borrow(cs).replace(Some(buf));
        } else {
            hprintln!("DMA1_CH1 IRQ: ERR: no xfer").unwrap();
        }
    });
}
