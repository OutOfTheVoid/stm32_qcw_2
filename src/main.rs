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
mod time;

use init::*;

use crate::{time::{get_time_ms, get_time_us}};

const RAMP_START_POWER : f32 =     0.1;
const RAMP_DEPTH       : f32 =     0.9;
const RAMP_TIME_US     : u64 =   10_000;
const COOLDOWN_TIME_US : u64 =     1000;
const OFF_TIME_US      : u64 = 800_000 - RAMP_TIME_US - COOLDOWN_TIME_US;

#[entry]
unsafe fn main() -> ! {
    let mut pac = pac::Peripherals::steal();
    system_init(&mut pac);

    let Peripherals {
        GPIOA        : mut gpio_a,
        GPIOB        : mut gpio_b,
        GPIOC        : mut gpio_c,
        TIM1         : mut tim_1,
        TIM2         : mut tim_2,
        TIM8         : mut tim_8,
        ..
    } = pac;

    let core_pac = cortex_m::Peripherals::take().unwrap();

    let cortex_m::Peripherals {
        SYST: mut syst,
        ..
    } = core_pac;

    time::init(tim_2);

    let mut start = time::get_time_ms();
    while (get_time_ms() - start) < 500 {}

    debug_led::setup(&mut gpio_c);
    gate_driver_supply_inverter::init(&mut gpio_a, &mut tim_1);
    buck_converter::init(&mut gpio_b, &mut tim_8);
    tesla_coil::init(&mut gpio_b);

    unsafe { cortex_m::interrupt::enable() };

    let mut start = time::get_time_us();

    loop {
        buck_converter::set_level(RAMP_START_POWER, &mut tim_8);

        start = time::get_time_us();

        tesla_coil::enable(&mut gpio_b);
        debug_led::set(&mut gpio_c, true);

        loop {
            let t = get_time_us() - start;
            buck_converter::set_level(RAMP_START_POWER + RAMP_DEPTH * (t as f32 / RAMP_TIME_US as f32), &mut tim_8);
            if t >= RAMP_TIME_US {
                break;
            }
        }

        buck_converter::set_level(0.0, &mut tim_8);

        start = time::get_time_us();
        loop {
            let t = get_time_us() - start;
            if t >= COOLDOWN_TIME_US {
                break;
            }
        }

        tesla_coil::disable(&mut gpio_b);
        debug_led::set(&mut gpio_c, false);

        start = time::get_time_us();
        while (get_time_us() - start) < OFF_TIME_US {}

    }
}