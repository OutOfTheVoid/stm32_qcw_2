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

use crate::{tesla_coil::HrtimPeripherals, time::{get_time_ms, get_time_us}};

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
        HRTIM_COMMON : mut hrtim_common,
        HRTIM_MASTER : mut hrtim_master,
        HRTIM_TIMA   : mut hrtim_tima,
        HRTIM_TIMB   : mut hrtim_timb,
        HRTIM_TIMC   : mut hrtim_timc,
        
        ..
    } = pac;

    let core_pac = cortex_m::Peripherals::take().unwrap();

    let cortex_m::Peripherals {
        SYST: mut syst,
        ..
    } = core_pac;

    let hrtim = HrtimPeripherals {
        common: hrtim_common,
        master: hrtim_master,
        timer_a: hrtim_tima,
        timer_b: hrtim_timb,
        timer_c: hrtim_timc
    };

    time::init(tim_2);
    debug_led::setup(&mut gpio_c);
    gate_driver_supply_inverter::init(&mut gpio_a, &mut tim_1);
    buck_converter::init(&mut gpio_b, &mut tim_8);
    tesla_coil::init(&mut gpio_b, &mut gpio_c, hrtim);

    unsafe { cortex_m::interrupt::enable() };

    let period = 160_000_000 / 400_000;

    loop {
        let mut start = time::get_time_us();

        tesla_coil::begin_open_loop(period as u16);
        debug_led::set(&mut gpio_c, true);

        while (get_time_us() - start) < 10 {}

        start = time::get_time_us();

        tesla_coil::stop();
        debug_led::set(&mut gpio_c, false);

        while (get_time_us() - start) < 9990 {}

    }
}