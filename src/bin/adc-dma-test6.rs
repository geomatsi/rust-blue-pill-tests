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
use hal::gpio::gpioa::{PA0, PA1, PA2, PA3, PA4};
use hal::gpio::Analog;
use hal::prelude::*;
use hal::stm32;
use panic_semihosting as _;
use rtfm;
use rtfm::app;
use rtfm::Instant;
use stm32f1xx_hal as hal;

type RdmaType1 = adc::AdcDma<AdcPinsOne, Scan>;
type RdmaType2 = adc::AdcDma<AdcPinsTwo, Scan>;

type RbufType1 = &'static mut [u16; 2];
type RbufType2 = &'static mut [u16; 3];

const PERIOD: u32 = 24_000_000;

pub struct AdcPinsOne(PA0<Analog>, PA1<Analog>);

impl SetChannels<AdcPinsOne> for Adc<stm32::ADC1> {
    fn set_samples(&mut self) {
        self.set_channel_sample_time(0, adc::SampleTime::T_28);
        self.set_channel_sample_time(1, adc::SampleTime::T_28);
    }

    fn set_sequence(&mut self) {
        self.set_regular_sequence(&[0, 1]);
    }
}

pub struct AdcPinsTwo(PA2<Analog>, PA3<Analog>, PA4<Analog>);

impl SetChannels<AdcPinsTwo> for Adc<stm32::ADC1> {
    fn set_samples(&mut self) {
        self.set_channel_sample_time(2, adc::SampleTime::T_28);
        self.set_channel_sample_time(3, adc::SampleTime::T_28);
        self.set_channel_sample_time(4, adc::SampleTime::T_28);
    }

    fn set_sequence(&mut self) {
        self.set_regular_sequence(&[2, 3, 4]);
    }
}

pub enum State {
    One,
    Two,
}

#[app(device = stm32f1xx_hal::stm32)]
const APP: () = {
    // resources
    static mut state: State = ();
    static mut transfer1: Option<Transfer<W, RbufType1, RdmaType1>> = ();
    static mut transfer2: Option<Transfer<W, RbufType2, RdmaType2>> = ();
    static mut adc_pins1: Option<AdcPinsOne> = ();
    static mut adc_pins2: Option<AdcPinsTwo> = ();
    static mut adc_dma1: Option<RdmaType1> = ();
    static mut adc_dma2: Option<RdmaType2> = ();
    static mut buffer1: Option<RbufType1> = ();
    static mut buffer2: Option<RbufType2> = ();

    #[init(schedule = [start_adc_dma])]
    fn init() {
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let clocks = rcc.cfgr.adcclk(1.mhz()).freeze(&mut flash.acr);
        //let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(32.mhz()).pclk1(16.mhz()).adcclk(8.mhz()).freeze(&mut flash.acr);

        // dma channel #1
        let mut dma_ch1 = device.DMA1.split(&mut rcc.ahb).1;
        dma_ch1.listen(dma::Event::TransferComplete);

        // setup ADC
        let adc1 = adc::Adc::adc1(device.ADC1, &mut rcc.apb2, clocks);

        // setup GPIOA
        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);

        // configure analog inputs
        let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);
        let adc_ch1 = gpioa.pa1.into_analog(&mut gpioa.crl);
        let adc_ch2 = gpioa.pa2.into_analog(&mut gpioa.crl);
        let adc_ch3 = gpioa.pa3.into_analog(&mut gpioa.crl);
        let adc_ch4 = gpioa.pa4.into_analog(&mut gpioa.crl);

        // configure ADC+DMA
        let adc_pins1 = AdcPinsOne(adc_ch0, adc_ch1);
        let adc_pins2 = AdcPinsTwo(adc_ch2, adc_ch3, adc_ch4);

        let buffer1 = singleton!(: [u16; 2] = [0; 2]).unwrap();
        let buffer2 = singleton!(: [u16; 3] = [0; 3]).unwrap();

        let adc_dma1 = adc1.with_scan_dma(adc_pins1, dma_ch1);

        schedule
            .start_adc_dma(Instant::now() + PERIOD.cycles())
            .unwrap();

        transfer1 = None;
        adc_pins1 = None;
        adc_dma1 = Some(adc_dma1);
        buffer1 = Some(buffer1);

        transfer2 = None;
        adc_pins2 = Some(adc_pins2);
        adc_dma2 = None;
        buffer2 = Some(buffer2);

        state = State::One;
    }

    #[idle]
    fn idle() -> ! {
        loop {
            cm::asm::wfi();
        }
    }

    #[task(resources = [state, transfer1, adc_pins1, adc_dma1, buffer1, transfer2, adc_pins2, adc_dma2, buffer2])]
    fn start_adc_dma() {
        match *resources.state {
            State::One => {
                if let (Some(adc_dma), Some(buffer)) =
                    (resources.adc_dma1.take(), resources.buffer1.take())
                {
                    hprintln!("TASK: start next xfer").unwrap();
                    let transfer = adc_dma.read(buffer);
                    *resources.transfer1 = Some(transfer);
                } else {
                    hprintln!("TASK: ERR: no ADC/DMA type One").unwrap();
                }
            }
            State::Two => {
                if let (Some(adc_dma), Some(buffer)) =
                    (resources.adc_dma2.take(), resources.buffer2.take())
                {
                    hprintln!("TASK: start next xfer").unwrap();
                    let transfer = adc_dma.read(buffer);
                    *resources.transfer2 = Some(transfer);
                } else {
                    hprintln!("TASK: ERR: no ADC/DMA type Two").unwrap();
                }
            }
        }
    }

    #[interrupt(schedule = [start_adc_dma], resources = [state, transfer1, adc_pins1, adc_dma1, buffer1, transfer2, adc_pins2, adc_dma2, buffer2])]
    fn DMA1_CHANNEL1() {
        match *resources.state {
            State::One => {
                if let (Some(transfer), Some(pins2)) =
                    (resources.transfer1.take(), resources.adc_pins2.take())
                {
                    let (buf1, adc_dma) = transfer.wait();
                    let (adc, pins1, chan) = adc_dma.split();

                    hprintln!("DMA1_CH1 IRQ: ONE: {:?}", buf1).unwrap();

                    *resources.adc_dma2 = Some(adc.with_scan_dma(pins2, chan));
                    *resources.adc_pins1 = Some(pins1);
                    *resources.buffer1 = Some(buf1);
                    *resources.state = State::Two;
                } else {
                    hprintln!("DMA1_CH1 IRQ: ERR: no transfer of type One").unwrap();
                }
            }
            State::Two => {
                if let (Some(transfer), Some(pins1)) =
                    (resources.transfer2.take(), resources.adc_pins1.take())
                {
                    let (buf2, adc_dma) = transfer.wait();
                    let (adc, pins2, chan) = adc_dma.split();

                    hprintln!("DMA1_CH1 IRQ: TWO: {:?}", buf2).unwrap();

                    *resources.adc_dma1 = Some(adc.with_scan_dma(pins1, chan));
                    *resources.adc_pins2 = Some(pins2);
                    *resources.buffer2 = Some(buf2);
                    *resources.state = State::One;
                } else {
                    hprintln!("DMA1_CH1 IRQ: ERR: no transfer of type One").unwrap();
                }
            }
        }

        schedule
            .start_adc_dma(Instant::now() + PERIOD.cycles())
            .unwrap();
    }

    // needed for RTFM timer queue and task management
    extern "C" {
        fn EXTI2();
    }
};
