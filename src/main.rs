#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unused)]

use core::f32;

use cortex_m::asm::{self, wfi};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32g4::stm32g474::{self as pac, hrtim_timc, GPIOB, GPIOC, TIM8};
use pac::Peripherals;
use libm::cosf;

mod init;
mod debug_led;
mod gate_driver_supply_inverter;
mod buck_converter;
mod tesla_coil;
mod time;

use init::*;

use crate::{time::{get_time_ms, get_time_us}};

const RAMP_START_POWER : f32 =      0.1;
const RAMP_DEPTH       : f32 =      0.9;
const MODULATION_DEPTH : f32 =      0.0;
const RAMP_TIME_US     : u64 =    8_000;
const COOLDOWN_TIME_US : u64 =    1_000;
const OFF_TIME_US      : u64 =  400_000 - RAMP_TIME_US - COOLDOWN_TIME_US;
const PAUSE_TIME_US    : u64 = 2000_000;

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
    buck_converter::init(&mut gpio_a, &mut gpio_c, &mut tim_8);
    tesla_coil::init(&mut gpio_c);

    loop {
        debug_led::set(&mut gpio_c, true);
        fire(
            10000, 
            |t, _| {
                if t < 7000 {
                    t as f32 * 0.5 / 7000f32 + 0.1
                } else if t < 8000 {
                    (t - 7000) as f32 * 0.4 / 1000f32 + 0.6
                } else {
                    (10000 - t) as f32 / 2000f32
                }
            },
            &mut tim_8,
            &mut gpio_c
        );
        debug_led::set(&mut gpio_c, false);
        let start = time::get_time_ms();
        while time::get_time_ms() - start < 490 {}
    }
}

fn fire<F: Fn(u32, f32) -> f32>(ontime_us: u32, ramp_fn: F, tim_8: &mut TIM8, gpio_c: &mut GPIOC) {
    buck_converter::set_level(ramp_fn(0, 0.0), tim_8);
    let mut start = time::get_time_us();
    loop {
        if (time::get_time_us() - start) >= 300 {
            break;
        }
    }
    tesla_coil::enable(gpio_c);
    start = time::get_time_us();
    loop {
        let t = time::get_time_us() - start;
        let level = ramp_fn(t as u32, t as f32 / ontime_us as f32);
        buck_converter::set_level(level, tim_8);
        if t >= ontime_us as u64 {
            break;
        }
    }
    tesla_coil::disable(gpio_c);
    buck_converter::set_level(0.0, tim_8);

}
