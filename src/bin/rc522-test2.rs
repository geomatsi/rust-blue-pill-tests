//#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m as cm;
use embedded_hal::digital::v1_compat::OldOutputPin;
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::*;
use hal::prelude::*;
use hal::spi::Spi;
use hal::stm32;
use hal::timer::Event;
use hal::timer::Timer;
use mfrc522::Mfrc522;
use panic_rtt_target as _;
use rtic::app;
use rtt_target::{rtt_init, UpChannel};
use stm32f1xx_hal as hal;
use stm32f1xx_hal::spi::Spi1Remap;

type SpiSckType = gpiob::PB3<Alternate<PushPull>>;
type SpiMisoType = gpiob::PB4<Input<Floating>>;
type SpiMosiType = gpiob::PB5<Alternate<PushPull>>;
type SpiNssType = gpioa::PA15<Output<PushPull>>;
type SpiType = Spi<stm32::SPI1, Spi1Remap, (SpiSckType, SpiMisoType, SpiMosiType), u8>;

#[app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // late resources
        stream1: UpChannel,
        stream2: UpChannel,
        tmr: hal::timer::CountDownTimer<stm32::TIM3>,
        led: gpioc::PC13<Output<PushPull>>,
        irq: gpiob::PB1<Input<Floating>>,
        nfc: Mfrc522<SpiType, OldOutputPin<SpiNssType>>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let channels = rtt_init! {
            up: {
                0: {
                    size: 512
                    name: "stream1"
                }
                1: {
                    size: 512
                    name: "stream2"
                }
            }
        };

        let stream1 = channels.up.0;
        let stream2 = channels.up.1;

        let mut rcc = cx.device.RCC.constrain();
        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);
        let mut flash = cx.device.FLASH.constrain();

        let clocks = rcc
            .cfgr
            .sysclk(8.mhz())
            .pclk1(8.mhz())
            .freeze(&mut flash.acr);

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);

        // configure PC13 pin to blink LED

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        led.set_high().unwrap();

        // configure and start TIM3 periodic timer

        let mut tmr = Timer::tim3(cx.device.TIM3, &clocks, &mut rcc.apb1).start_count_down(5.hz());
        tmr.listen(Event::Update);

        // configure  external irq line from NFC chip

        let mut irq = gpiob.pb1.into_floating_input(&mut gpiob.crl);
        irq.make_interrupt_source(&mut afio);
        irq.trigger_on_edge(&cx.device.EXTI, Edge::RISING_FALLING);
        irq.enable_interrupt(&cx.device.EXTI);

        unsafe {
            cm::peripheral::NVIC::unmask(stm32::Interrupt::EXTI1);
        }

        // configure SPI and connected NFC RC522 board

        let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let sck = pb3.into_alternate_push_pull(&mut gpiob.crl);
        let mosi = gpiob.pb5.into_alternate_push_pull(&mut gpiob.crl);
        let miso = pb4;
        let nss = pa15.into_push_pull_output(&mut gpioa.crh);

        let spi = Spi::spi1(
            cx.device.SPI1,
            (sck, miso, mosi),
            &mut afio.mapr,
            mfrc522::MODE,
            1.mhz(),
            clocks,
            &mut rcc.apb2,
        );

        let nfc = Mfrc522::new(spi, OldOutputPin::from(nss)).unwrap();

        init::LateResources {
            stream1,
            stream2,
            tmr,
            led,
            irq,
            nfc,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cm::asm::nop();
        }
    }

    #[task(binds = EXTI1, resources = [irq, nfc, stream1])]
    fn exti(cx: exti::Context) {
        if cx.resources.irq.check_interrupt() {
            if let Ok(atqa) = cx.resources.nfc.reqa() {
                if let Ok(uid) = cx.resources.nfc.select(&atqa) {
                    writeln!(cx.resources.stream1, "NFC: * {:?}", uid).ok();
                } else {
                    writeln!(cx.resources.stream1, "NFC: failed to read UID").ok();
                }
            } else {
                writeln!(cx.resources.stream1, "NFC: empty IRQ").ok();
            }
            cx.resources.irq.clear_interrupt_pending_bit();
        } else {
            writeln!(cx.resources.stream1, "NFC: unexpected IRQ").ok();
        }
    }

    #[task(binds = TIM3, resources = [led, tmr, stream2])]
    fn tim3(cx: tim3::Context) {
        writeln!(cx.resources.stream2, "TIM3 blink").ok();
        cx.resources.led.toggle().unwrap();
        cx.resources.tmr.clear_update_interrupt_flag();
    }
};
