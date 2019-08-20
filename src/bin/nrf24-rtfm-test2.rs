#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cm::iprintln;
use cortex_m as cm;
use embedded_hal::spi::MODE_0;
use hal::gpio;
use hal::prelude::*;
use hal::spi::Spi;
use panic_itm as _;
use rtfm;
use rtfm::app;
use stm32f1xx_hal as hal;

use embedded_nrf24l01;
use embedded_nrf24l01::Configuration;
use embedded_nrf24l01::CrcMode;
use embedded_nrf24l01::DataRate;
use embedded_nrf24l01::StandbyMode;
use embedded_nrf24l01::NRF24L01;

type Standby = StandbyMode<
    NRF24L01<
        gpio::gpiob::PB0<gpio::Output<gpio::PushPull>>,
        gpio::gpioa::PA4<gpio::Output<gpio::PushPull>>,
        Spi<
            hal::stm32::SPI1,
            (
                gpio::gpioa::PA5<gpio::Alternate<gpio::PushPull>>,
                gpio::gpioa::PA6<gpio::Input<gpio::Floating>>,
                gpio::gpioa::PA7<gpio::Alternate<gpio::PushPull>>,
            ),
        >,
    >,
>;

// Simple Rx test for embedded-nrf24l01 crate

#[app(device = stm32f1xx_hal::stm32)]
const APP: () = {
    static mut NRF: Option<Standby> = ();
    static mut LED: gpio::gpioc::PC13<hal::gpio::Output<hal::gpio::PushPull>> = ();
    static mut ITM: hal::stm32::ITM = ();

    #[init]
    fn init() {
        // configure clocks
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(8.mhz())
            .pclk1(8.mhz())
            .freeze(&mut flash.acr);

        let mut afio = device.AFIO.constrain(&mut rcc.apb2);

        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

        // configure PC13 pin as LED
        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

        // configure PB0 pin as NRF24 CE
        let ce = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);

        // configure PA4 pin as NRF24 NCS
        let cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);

        // configure PA5/PA6/PA7 as SPI1 pins
        let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        // configure SPI1 for NRF24
        let spi = Spi::spi1(
            device.SPI1,
            (sck, miso, mosi),
            &mut afio.mapr,
            MODE_0,
            2.mhz(),
            clocks,
            &mut rcc.apb2,
        );

        // nRF24L01 general setup
        let mut nrf = NRF24L01::new(ce, cs, spi).unwrap();
        nrf.set_frequency(120).unwrap();
        nrf.set_rf(DataRate::R250Kbps, 3 /* 0 dBm */).unwrap();
        nrf.set_crc(Some(CrcMode::OneByte)).unwrap();
        nrf.set_auto_retransmit(0b0100, 0b1111).unwrap();
        nrf.set_auto_ack(&[true; 6]).unwrap();

        // nRF24L01 Rx setup
        let addr: [u8; 5] = [0xe5, 0xe4, 0xe3, 0xe2, 0xe1];
        nrf.set_rx_addr(0, &addr).unwrap();
        nrf.set_pipes_rx_lengths(&[None; 6]).unwrap();
        nrf.set_pipes_rx_enable(&[true, false, false, false, false, false])
            .unwrap();
        nrf.flush_tx().unwrap();
        nrf.flush_rx().unwrap();

        // init late resources
        NRF = Some(nrf);
        LED = led;
        ITM = core.ITM;
    }

    #[idle(resources = [LED, ITM, NRF])]
    fn idle() -> ! {
        let dbg = &mut resources.ITM.stim[0];
        let nrf = resources.NRF.take().expect("NRF is not available");
        let rx = &mut nrf.rx().unwrap();

        iprintln!(dbg, "NRF Rx is ready to go...");

        loop {
            let pipe = rx.can_read().unwrap();

            if pipe.is_some() {
                let data = rx.read().unwrap();
                iprintln!(dbg, "{:?}", data.as_ref());
                resources.LED.toggle().unwrap();
            }
        }
    }
};
