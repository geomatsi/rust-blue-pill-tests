#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cm::singleton;
use cortex_m as cm;
use cortex_m_semihosting::hprintln;
use hal::adc;
use hal::adc::Adc;
use hal::adc::Scan;
use hal::adc::SetChannels;
use hal::dma;
use hal::dma::{Transfer, W};
use hal::gpio::gpioa::{PA0, PA1, PA2, PA3};
use hal::gpio::Analog;
use hal::prelude::*;
use hal::stm32;
use panic_semihosting as _;
use rtic::app;
use rtic::cyccnt::Instant;
use rtic::cyccnt::U32Ext;
use stm32f1xx_hal as hal;

type RdmaT = adc::AdcDma<AdcPins, Scan>;
type RbufT = &'static mut [u16; 4];

const PERIOD: u32 = 24_000_000;

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
#[app(device = stm32f1xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        // late resources
        xfr: Option<Transfer<W, RbufT, RdmaT>>,
        dma: Option<RdmaT>,
        buf: Option<RbufT>,
    }

    #[init(schedule = [start_adc_dma])]
    fn init(mut cx: init::Context) -> init::LateResources {
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        let clocks = rcc.cfgr.adcclk(1.mhz()).freeze(&mut flash.acr);
        //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(32.mhz()).pclk1(16.mhz()).adcclk(8.mhz()).freeze(&mut flash.acr);

        // dma channel #1
        let mut dma_ch1 = cx.device.DMA1.split(&mut rcc.ahb).1;
        dma_ch1.listen(dma::Event::TransferComplete);

        // setup ADC
        let adc1 = adc::Adc::adc1(cx.device.ADC1, &mut rcc.apb2, clocks);

        // setup GPIOA
        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);

        // configure analog inputs
        let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);
        let adc_ch1 = gpioa.pa1.into_analog(&mut gpioa.crl);
        let adc_ch2 = gpioa.pa2.into_analog(&mut gpioa.crl);
        let adc_ch3 = gpioa.pa3.into_analog(&mut gpioa.crl);

        // configure ADC+DMA
        let adc_pins = AdcPins(adc_ch0, adc_ch1, adc_ch2, adc_ch3);
        let buffer = singleton!(: [u16; 4] = [0; 4]).unwrap();
        let adc_dma = adc1.with_scan_dma(adc_pins, dma_ch1);

        /* Enable the monotonic timer based on CYCCNT */
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();

        cx.schedule
            .start_adc_dma(Instant::now() + PERIOD.cycles())
            .unwrap();

        init::LateResources {
            xfr: None,
            dma: Some(adc_dma),
            buf: Some(buffer),
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cm::asm::wfi();
        }
    }

    #[task(resources = [xfr, dma, buf])]
    fn start_adc_dma(cx: start_adc_dma::Context) {
        if let (Some(adc_dma), Some(buffer)) = (cx.resources.dma.take(), cx.resources.buf.take()) {
            hprintln!("IDLE: start next xfer").unwrap();
            let transfer = adc_dma.read(buffer);
            *cx.resources.xfr = Some(transfer);
        } else {
            hprintln!("IDLE: ERR: no rdma").unwrap();
        }
    }

    #[task(binds = DMA1_CHANNEL1, schedule = [start_adc_dma], resources = [xfr, dma, buf])]
    fn dma1_channel1(cx: dma1_channel1::Context) {
        if let Some(xfr) = cx.resources.xfr.take() {
            let (buf, dma) = xfr.wait();
            hprintln!("DMA1_CH1 IRQ: {:?}", buf).unwrap();
            *cx.resources.dma = Some(dma);
            *cx.resources.buf = Some(buf);
        } else {
            hprintln!("DMA1_CH1 IRQ: ERR: no xfer").unwrap();
        }

        cx.schedule
            .start_adc_dma(Instant::now() + PERIOD.cycles())
            .unwrap();
    }

    // needed for RTFM timer queue and task management
    extern "C" {
        fn EXTI2();
    }
};
