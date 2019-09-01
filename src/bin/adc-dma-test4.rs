#![no_main]
#![no_std]

use cm::interrupt::Mutex;
use cm::singleton;
use core::cell::RefCell;
use cortex_m as cm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use hal::adc;
use hal::adc::Adc;
use hal::adc::Scan;
use hal::adc::SetChannels;
use hal::delay::Delay;
use hal::dma;
use hal::dma::{Transfer, W};
use hal::gpio::gpioa::{PA0, PA1, PA2, PA3};
use hal::gpio::Analog;
use hal::pac;
use hal::prelude::*;
use hal::stm32;
use hal::stm32::interrupt;
use panic_semihosting as _;
use stm32f1xx_hal as hal;

type RdmaT = adc::AdcDma<AdcPins, Scan>;
type RbufT = &'static mut [u16; 4];

static G_XFR: Mutex<RefCell<Option<Transfer<W, RbufT, RdmaT>>>> = Mutex::new(RefCell::new(None));
static G_DMA: Mutex<RefCell<Option<RdmaT>>> = Mutex::new(RefCell::new(None));
static G_BUF: Mutex<RefCell<Option<RbufT>>> = Mutex::new(RefCell::new(None));

pub struct AdcPins(PA0<Analog>, PA1<Analog>, PA2<Analog>, PA3<Analog>);

impl SetChannels<AdcPins> for Adc<stm32::ADC1> {
    fn set_samples(&mut self) {
        self.set_channel_sample_time(0, adc::SampleTime::T_28);
        self.set_channel_sample_time(1, adc::SampleTime::T_28);
        self.set_channel_sample_time(2, adc::SampleTime::T_28);
        self.set_channel_sample_time(3, adc::SampleTime::T_28);
    }
    fn set_sequence(&mut self) {
        self.set_regular_sequence(&[0, 1, 2, 3]);
    }
}

#[entry]
fn main() -> ! {
    let cp = cm::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut nvic = cp.NVIC;

    let clocks = rcc.cfgr.adcclk(1.mhz()).freeze(&mut flash.acr);
    //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(32.mhz()).pclk1(16.mhz()).adcclk(8.mhz()).freeze(&mut flash.acr);

    // delay
    let mut delay = Delay::new(cp.SYST, clocks);

    // dma channel #1
    let mut dma_ch1 = dp.DMA1.split(&mut rcc.ahb).1;
    dma_ch1.listen(dma::Event::TransferComplete);

    // setup ADC
    let adc1 = adc::Adc::adc1(dp.ADC1, &mut rcc.apb2, clocks);

    // setup GPIOA
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // Configure analog inputs
    let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);
    let adc_ch1 = gpioa.pa1.into_analog(&mut gpioa.crl);
    let adc_ch2 = gpioa.pa2.into_analog(&mut gpioa.crl);
    let adc_ch3 = gpioa.pa3.into_analog(&mut gpioa.crl);

    let adc_pins = AdcPins(adc_ch0, adc_ch1, adc_ch2, adc_ch3);

    let buf = singleton!(: [u16; 4] = [0; 4]).unwrap();

    let adc_dma = adc1.with_scan_dma(adc_pins, dma_ch1);
    let xfer = adc_dma.read(buf);

    cm::interrupt::free(|cs| {
        G_XFR.borrow(cs).replace(Some(xfer));
    });

    nvic.enable(stm32::Interrupt::DMA1_CHANNEL1);

    cm::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_CHANNEL1);

    unsafe {
        nvic.set_priority(stm32::Interrupt::DMA1_CHANNEL1, 1);
    }

    loop {
        hprintln!("IDLE: wait 1 sec").unwrap();
        delay.delay_ms(1_000u16);

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
    }
}

#[interrupt]
fn DMA1_CHANNEL1() {
    cm::interrupt::free(|cs| {
        if let Some(xfer) = G_XFR.borrow(cs).replace(None) {
            let (buf, adc_dma) = xfer.wait();
            hprintln!("DMA1_CH1 IRQ: {:?}", buf).unwrap();
            G_DMA.borrow(cs).replace(Some(adc_dma));
            G_BUF.borrow(cs).replace(Some(buf));
        } else {
            hprintln!("DMA1_CH1 IRQ: ERR: no xfer").unwrap();
        }
    });
}
