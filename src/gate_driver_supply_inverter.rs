// High side  - PA8 / TIM1_CH1  (AF6)
// Low side   - PA10 / TIM1_CH3  (AF6)

use stm32g4::stm32g474::{Peripherals, GPIOA, TIM1};

pub fn init(gpio_a: &mut GPIOA, tim1: &mut TIM1) {
    // init gpio
    gpio_a.afrh().modify(|_, w| {
        w
            .afrh8().af6()
            .afrh10().af6()
    });
    gpio_a.moder().modify(|_, w| {
        w
            .moder8().alternate()
            .moder10().alternate()
    });
    gpio_a.otyper().modify(|_, w| {
        w
            .ot8().push_pull()
            .ot10().push_pull()
    });
    gpio_a.ospeedr().modify(|_, w| {
        w 
            .ospeedr8().low_speed()
            .ospeedr10().low_speed()
    });

    // setup TIM1 as the oscillator
    // free running, up counter at 200kHz
    // period = 170_000_000 / 200_000 = 850
    tim1.psc().modify(|_, w| w.psc().set(0));
    tim1.cr1().modify(|_, w| {
        w
            .cms().edge_aligned()
            .dir().up()
    });
    tim1.cr2().modify(|_, w| {
        w
            .ois1().clear_bit()
            .ois3().clear_bit()
    });
    tim1.ccmr1_output().modify(|_, w| {
        w
            .oc1m().force_active()
            .oc1m_3().extended()
            .oc2m().force_inactive()
            .oc2m_3().extended()
    });
    tim1.ccmr2_output().modify(|_, w| {
        w
            .oc3m().force_active()
            .oc3m_3().extended()
            .oc4m().force_inactive()
            .oc4m_3().extended()
    });

    const PERIOD: u32 = 850;

    tim1.arr().modify(|_, w| w.arr().set(PERIOD));
    tim1.cnt().modify(|_, w| w.cnt().set(0));

    let half_deadtime = PERIOD / 8;
    let half_period = PERIOD / 2;

    tim1.ccr1().modify(|_, w| w.ccr().set(half_deadtime * 1));
    tim1.ccr2().modify(|_, w| w.ccr().set(half_period - half_deadtime * 1));

    tim1.ccr3().modify(|_, w| w.ccr().set(half_period + half_deadtime));
    tim1.ccr4().modify(|_, w| w.ccr().set(PERIOD - half_deadtime * 1));

    tim1.ccer().modify(|_, w| {
        w
            .cc1e().enabled()
            .cc3e().enabled()
    });
    tim1.bdtr().modify(|_, w| w.moe().enabled());

    tim1.cr1().modify(|_, w| w.cen().set_bit());
}