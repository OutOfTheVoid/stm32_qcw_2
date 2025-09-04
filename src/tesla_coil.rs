/*
 * PB5 - output, push-pull : enable
 */

use stm32g4::stm32g474::GPIOC;

pub fn init(gpio_c: &mut GPIOC) {
    gpio_c.odr().modify(|_, w| w.odr11().low());
    gpio_c.otyper().modify(|_, w| w.ot11().push_pull());
    gpio_c.moder().modify(|_, w| w.moder11().output());
    gpio_c.ospeedr().modify(|_, w| w.ospeedr11().high_speed());
}

pub fn enable(gpio_c: &mut GPIOC) {
    gpio_c.odr().modify(|_, w| w.odr11().set_bit());
}

pub fn disable(gpio_c: &mut GPIOC) {
    gpio_c.odr().modify(|_, w| w.odr11().clear_bit());
}
