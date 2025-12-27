/*
 * PA15 : AF2 - TIM8_CH1
 * PC13 : AF6 - TIM8_CH4N   
 */

use stm32g4::stm32g474::{Peripherals, GPIOA, GPIOC, TIM8};

const DEADTIME: u32 = 200;
const PERIOD: u32 = 160_000_000 / 30_000;
const PERIOD_MINUS_DEADTIME: u32 = PERIOD - DEADTIME;

pub fn init(gpio_a: &mut GPIOA, gpio_c: &mut GPIOC, tim_8: &mut TIM8) {
    // init gpio
    gpio_a.afrh().modify(|_, w| w.afrh15().af2());
    gpio_a.moder().modify(|_, w| w.moder15().alternate());
    gpio_a.otyper().modify(|_, w| w.ot15().push_pull());
    gpio_a.ospeedr().modify(|_, w| w.ospeedr15().high_speed());

    gpio_c.afrh().modify(|_, w| w.afrh13().af6());
    gpio_c.moder().modify(|_, w| w.moder13().alternate());
    gpio_c.otyper().modify(|_, w| w.ot13().push_pull());
    gpio_c.ospeedr().modify(|_, w| w.ospeedr13().high_speed());

    // setup TIM8 as the oscillator
    // free running, up counter at 30kHz
    // period = 170_000_000 / 200_000 = 850
    tim_8.psc().modify(|_, w| w.psc().set(0));
    tim_8.cr1().modify(|_, w| {
        w
            .cms().edge_aligned()
            .dir().up()
    });
    tim_8.cr2().modify(|_, w| {
        w
            .ois1().clear_bit()
            .ois4n().clear_bit()
    });
    tim_8.ccmr1_output().modify(|_, w| {
        w
            .oc1m().force_active()
            .oc1m_3().extended()
            .oc2m().force_inactive()
            .oc2m_3().extended()
    });
    tim_8.ccmr2_output().modify(|_, w| {
        w
            .oc4m().force_active()
            .oc4m_3().extended()
            .oc3m().force_inactive()
            .oc3m_3().extended()
    });

    tim_8.arr().modify(|_, w| w.arr().set(PERIOD));
    tim_8.cnt().modify(|_, w| w.cnt().set(0));

    set_level(0.0, tim_8);

    tim_8.ccer().modify(|_, w| {
        w
            .cc1e().enabled()
            //.cc3e().enabled()
            .cc4ne().enabled()
    });
    tim_8.bdtr().modify(|_, w| w.moe().enabled());
    tim_8.cr1().modify(|_, w| w.cen().set_bit());
}

pub fn set_level(level: f32, tim_8: &mut TIM8) {
    let on_time = (((PERIOD as f32) * level) as i32)
        .clamp(0, PERIOD_MINUS_DEADTIME as i32) as u32;

    /*
         .--------------------.               
         |                    |               
    .----.                    .--.--.        .
                                    |        |
                                    .--------.
    ^     ^                   ^     ^        
    |-----|                   |-----|        
    dt / 2                     dt / 2         

          |--------------------|     |--------|
              on-time                 off-time
     */

    tim_8.ccr1().modify(|_, w| w.ccr().set(DEADTIME / 2));
    tim_8.ccr2().modify(|_, w| w.ccr().set(on_time));

    tim_8.ccr4().modify(|_, w| w.ccr().set(DEADTIME / 2 + on_time));
    tim_8.ccr3().modify(|_, w| w.ccr().set(PERIOD));
}
