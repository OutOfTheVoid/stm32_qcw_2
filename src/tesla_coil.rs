/*
 * PB5 - output, push-pull : enable
 */

use stm32g4::stm32g474::GPIOB;

pub fn init(gpio_b: &mut GPIOB) {
    gpio_b.odr().modify(|_, w| w.odr5().low());
    gpio_b.otyper().modify(|_, w| w.ot5().push_pull());
    gpio_b.moder().modify(|_, w| w.moder5().output());
    gpio_b.ospeedr().modify(|_, w| w.ospeedr5().high_speed());
}

pub fn enable(gpio_b: &mut GPIOB) {
    gpio_b.odr().modify(|_, w| w.odr5().set_bit());
}

pub fn disable(gpio_b: &mut GPIOB) {
    gpio_b.odr().modify(|_, w| w.odr5().clear_bit());
}
