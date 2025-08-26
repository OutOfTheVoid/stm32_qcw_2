use core::{mem::MaybeUninit, sync::atomic::{self, AtomicU32}};

use cortex_m_rt::exception;
use cortex_m::peripheral::scb::Exception;
use stm32g4::stm32g474::{interrupt, Interrupt, NVIC, TIM2};

static ROLLOVER_COUNT: AtomicU32 = AtomicU32::new(0);

static mut TIMER_2: MaybeUninit<TIM2> = MaybeUninit::uninit();

pub fn init(mut timer_2: TIM2) {
    unsafe {
        TIMER_2.write(timer_2);
        let timer_2 = TIMER_2.assume_init_mut();
        timer_2.arr().modify(|_, w| w.arr().set(160_000_000 - 1));
        timer_2.cnt().write(|w| w.cnt().set(0));
        timer_2.egr().write(|w| w.ug().set_bit());
        timer_2.dier().modify(|_, w| w.uie().set_bit());
        timer_2.psc().modify(|_, w| w.psc().set(0));
        timer_2.sr().modify(|_, w| w.uif().clear());
        timer_2.cr1().modify(|_, w| w.urs().any_event());
        NVIC::unmask(Interrupt::TIM2);
        timer_2.cr1().modify(|_, w| w.cen().set_bit());
    };
}



pub fn get_time_ns() -> u64 {
    loop {
        let seconds_a = ROLLOVER_COUNT.load(atomic::Ordering::Acquire);
        let subseconds = cortex_m::interrupt::free(|_| unsafe { TIMER_2.assume_init_ref() }.cnt().read().cnt().bits());
        let seconds_b = ROLLOVER_COUNT.load(atomic::Ordering::Acquire);
        if seconds_a == seconds_b {
            return (seconds_a as u64) * 1_000_000_000 + (subseconds as u64 * 1_000_000_000 / 160_000_000);
        }
    }
}

pub fn get_time_us() -> u64 {
    get_time_ns() / 1_000
}

pub fn get_time_ms() -> u64 {
    get_time_ns() / 1_000_000
}

pub fn get_time_seconds() -> f64 {
    get_time_ns() as f64 / 1_000_000_000.0
}


#[interrupt]
unsafe fn TIM2() {
    TIMER_2.assume_init_mut().sr().modify(|_, w| w.uif().clear());
    ROLLOVER_COUNT.fetch_add(1, atomic::Ordering::AcqRel);
}
