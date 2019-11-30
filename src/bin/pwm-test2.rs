#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use panic_semihosting as _;

use stm32f1xx_hal::delay::Delay;
use stm32f1xx_hal::timer::Timer;
use stm32f1xx_hal::{prelude::*, stm32};

/* make sure at least one PWM feature is selected */
#[cfg(not(any(
    feature = "tim2_remap_00",
    feature = "tim2_remap_01",
    feature = "tim2_remap_10",
    feature = "tim2_remap_11",
    feature = "tim3_remap_00",
    feature = "tim3_remap_10",
    feature = "tim4_remap_00"
)))]
compile_error!("PWM feature not selected");

#[cfg(feature = "tim2_remap_11")]
use stm32f1xx_hal::timer::Tim2FullRemap;
#[cfg(feature = "tim2_remap_00")]
use stm32f1xx_hal::timer::Tim2NoRemap;
#[cfg(feature = "tim2_remap_01")]
use stm32f1xx_hal::timer::Tim2PartialRemap1;
#[cfg(feature = "tim2_remap_10")]
use stm32f1xx_hal::timer::Tim2PartialRemap2;

#[cfg(feature = "tim3_remap_00")]
use stm32f1xx_hal::timer::Tim3NoRemap;
#[cfg(feature = "tim3_remap_10")]
use stm32f1xx_hal::timer::Tim3PartialRemap;

#[cfg(feature = "tim4_remap_00")]
use stm32f1xx_hal::timer::Tim4NoRemap;

/* main */

#[entry]
fn main() -> ! {
    let core = cm::Peripherals::take().unwrap();
    let p = stm32::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut delay = Delay::new(core.SYST, clocks);

    #[cfg(feature = "tim2_remap_00")]
    let mut chan = {
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);

        let p1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
        let p2 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
        let p3 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let p4 = gpioa.pa3.into_alternate_push_pull(&mut gpioa.crl);

        Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2NoRemap, _, _, _>(
            (p1, p2, p3, p4),
            &mut afio.mapr,
            10.khz(),
        )
    };

    #[cfg(feature = "tim2_remap_01")]
    let mut chan = {
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);

        // Use this to configure NJTRST as PB4
        let (pa15, pb3, _pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let p1 = pa15.into_alternate_push_pull(&mut gpioa.crh);
        let p2 = pb3.into_alternate_push_pull(&mut gpiob.crl);
        let p3 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let p4 = gpioa.pa3.into_alternate_push_pull(&mut gpioa.crl);

        Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2PartialRemap1, _, _, _>(
            (p1, p2, p3, p4),
            &mut afio.mapr,
            10.khz(),
        )
    };

    #[cfg(feature = "tim2_remap_10")]
    let mut chan = {
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);

        let p1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
        let p2 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
        let p3 = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
        let p4 = gpiob.pb11.into_alternate_push_pull(&mut gpiob.crh);

        Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2PartialRemap2, _, _, _>(
            (p1, p2, p3, p4),
            &mut afio.mapr,
            10.khz(),
        )
    };

    #[cfg(feature = "tim2_remap_11")]
    let mut chan = {
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);

        // Use this to configure NJTRST as PB4
        let (pa15, pb3, _pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let p1 = pa15.into_alternate_push_pull(&mut gpioa.crh);
        let p2 = pb3.into_alternate_push_pull(&mut gpiob.crl);
        let p3 = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
        let p4 = gpiob.pb11.into_alternate_push_pull(&mut gpiob.crh);

        Timer::tim2(p.TIM2, &clocks, &mut rcc.apb1).pwm::<Tim2FullRemap, _, _, _>(
            (p1, p2, p3, p4),
            &mut afio.mapr,
            10.khz(),
        )
    };

    #[cfg(feature = "tim3_remap_00")]
    let mut chan = {
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);

        let p1 = gpioa.pa6.into_alternate_push_pull(&mut gpioa.crl);
        let p2 = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
        let p3 = gpiob.pb0.into_alternate_push_pull(&mut gpiob.crl);
        let p4 = gpiob.pb1.into_alternate_push_pull(&mut gpiob.crl);

        Timer::tim3(p.TIM3, &clocks, &mut rcc.apb1).pwm::<Tim3NoRemap, _, _, _>(
            (p1, p2, p3, p4),
            &mut afio.mapr,
            10.khz(),
        )
    };

    #[cfg(feature = "tim3_remap_10")]
    let mut chan = {
        let gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);

        // Use this to configure NJTRST as PB4
        let (_pa15, _pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let p1 = pb4.into_alternate_push_pull(&mut gpiob.crl);
        let p2 = gpiob.pb5.into_alternate_push_pull(&mut gpiob.crl);
        let p3 = gpiob.pb0.into_alternate_push_pull(&mut gpiob.crl);
        let p4 = gpiob.pb1.into_alternate_push_pull(&mut gpiob.crl);

        Timer::tim3(p.TIM3, &clocks, &mut rcc.apb1).pwm::<Tim3PartialRemap, _, _, _>(
            (p1, p2, p3, p4),
            &mut afio.mapr,
            10.khz(),
        )
    };

    #[cfg(feature = "tim4_remap_00")]
    let mut chan = {
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);

        let p1 = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let p2 = gpiob.pb7.into_alternate_push_pull(&mut gpiob.crl);
        let p3 = gpiob.pb8.into_alternate_push_pull(&mut gpiob.crh);
        let p4 = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

        Timer::tim4(p.TIM4, &clocks, &mut rcc.apb1).pwm::<Tim4NoRemap, _, _, _>(
            (p1, p2, p3, p4),
            &mut afio.mapr,
            10.khz(),
        )
    };

    let max: u16 = chan.0.get_max_duty();
    let duty: u16 = max / 2;

    hprintln!("PWM max duty {}, duty {}", max, duty).unwrap();

    chan.0.enable();
    chan.1.enable();
    chan.2.enable();
    chan.3.enable();

    hprintln!("Lets rock !").unwrap();

    loop {
        hprintln!("ping...").unwrap();
        chan.0.set_duty(duty);
        chan.1.set_duty(duty);
        chan.2.set_duty(duty);
        chan.3.set_duty(duty);
        delay.delay_ms(500u16);

        hprintln!("pong...").unwrap();
        chan.0.set_duty(0);
        chan.1.set_duty(0);
        chan.2.set_duty(0);
        chan.3.set_duty(0);
        delay.delay_ms(500u16);
    }
}
