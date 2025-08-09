#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unused)]

use cortex_m::asm::{self, wfi};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32g4::stm32g474::{self as pac, hrtim_timc};
use pac::Peripherals;

mod init;
mod debug_led;
mod gate_driver_supply_inverter;
mod buck_converter;
mod tesla_coil;

use init::*;

use crate::tesla_coil::HrtimPeripherals;

#[entry]
unsafe fn main() -> ! {
    let mut pac = pac::Peripherals::steal();
    system_init(&mut pac);

    let Peripherals {
        GPIOA        : mut gpio_a,
        GPIOB        : mut gpio_b,
        GPIOC        : mut gpio_c,
        TIM1         : mut tim_1,
        TIM8         : mut tim_8,
        HRTIM_COMMON : mut hrtim_common,
        HRTIM_MASTER : mut hrtim_master,
        HRTIM_TIMA   : mut hrtim_tima,
        HRTIM_TIMB   : mut hrtim_timb,
        HRTIM_TIMC   : mut hrtim_timc,
        ..
    } = pac;

    let hrtim = HrtimPeripherals {
        common: hrtim_common,
        master: hrtim_master,
        timer_a: hrtim_tima,
        timer_b: hrtim_timb,
        timer_c: hrtim_timc
    };

    debug_led::setup(&mut gpio_c);
    //gate_driver_supply_inverter::init(&mut gpio_a, &mut tim_1);
    //buck_converter::init(&mut gpio_b, &mut tim_8);
    tesla_coil::init(&mut gpio_b, &mut gpio_c, hrtim);

    //gpio_c.moder().modify(|_, w| w.moder11().input());
    //gpio_c.pupdr().modify(|_, w| w.pupdr11().pull_up());

    //tesla_coil::read_feedback_period_raw();

    unsafe { cortex_m::interrupt::enable() };

    const CLOCK_SPEED: i32 = 160_000_000;
    const PERIOD_MIN: i32 = CLOCK_SPEED / 500_000;
    const PERIOD_MAX: i32 = CLOCK_SPEED / 300_000;
    let mut period = PERIOD_MIN;
    let mut period_inc = 1;

    loop {
        tesla_coil::begin_open_loop(period as u16);
        for _ in 0..100_000 {
            asm::nop();
        }
        tesla_coil::stop();
        if period == PERIOD_MIN {
            period_inc = 1;
            debug_led::set(&mut gpio_c, true);
        }
        if period == PERIOD_MAX {
            period_inc = -1;
            debug_led::set(&mut gpio_c, true);
        }
        period += period_inc;
    }
}