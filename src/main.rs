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

    let mut start = time::get_time_ms();
    while (get_time_ms() - start) < 500 {}

    debug_led::setup(&mut gpio_c);
    gate_driver_supply_inverter::init(&mut gpio_a, &mut tim_1);
    buck_converter::init(&mut gpio_b, &mut tim_8);
    tesla_coil::init(&mut gpio_b, &mut gpio_c, hrtim);

    unsafe { cortex_m::interrupt::enable() };

    let mut period = (160_000_000 / 440_000) as u16;

    buck_converter::set_level(1.0, &mut tim_8);

    loop {
        //if let Some(new_period) = tesla_coil::poll_feedback_period() {
        //    period = new_period;
        //}

        let mut start = time::get_time_us();

        tesla_coil::start_open_loop(period as u16);

        while (get_time_us() - start) < 50 {}
        start = time::get_time_us();

        tesla_coil::continue_closed_loop();

        while (get_time_us() - start) < 3950 {
            if tesla_coil::check_closed_loop_operational() {
                debug_led::set(&mut gpio_c, true);
            }
        }
        start = time::get_time_us();

        tesla_coil::stop();
        debug_led::set(&mut gpio_c, false);

        while (get_time_us() - start) < 96_000 {}

    }
}