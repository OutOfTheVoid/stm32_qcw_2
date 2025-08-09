use stm32g4::stm32g474::{Peripherals, GPIOC};

pub fn setup(gpio_c: &mut GPIOC) {
    gpio_c.moder().modify(|_, w| w.moder6().output());
    gpio_c.otyper().modify(|_, w| w.ot6().push_pull());
}

pub fn set(gpio_c: &mut GPIOC, state: bool) {
    gpio_c.odr().modify(|_, w| {
        if state {
            w.odr6().set_bit()
        } else {
            w.odr6().clear_bit()
        }
    });
}

