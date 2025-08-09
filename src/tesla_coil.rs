/*
 * PB12 : AF13 - HRTIM1_CHC1 - out +
 * PB13 : AF13 - HRTIM1_CHC2 - out -
 * PB3  : AF13 - HRTIM1_EEV9 - feedback
 */

use core::{cell::RefCell, mem::MaybeUninit, sync::atomic::{AtomicU32, Ordering}};

use cortex_m::interrupt::{self as cm_interrupt};
use stm32g4::stm32g474::{self, interrupt, GPIOB, GPIOC, HRTIM_COMMON, HRTIM_MASTER, HRTIM_TIMA, HRTIM_TIMB, HRTIM_TIMC, NVIC};

// 320 cycles at 160 MHz gives us a max frequency of 500 kHz
const FEEDBACK_BLANKING_INTERVAL: u16 = 320;

// 533 cycles at 160 mhz gives us a min frequency of ~300 kHz
const FEEDBACK_VALID_INTERVAL: u16 = 533;

const CKPSC_1: u8 = 0b101;

pub struct HrtimPeripherals {
    pub common: HRTIM_COMMON,
    pub master: HRTIM_MASTER,
    pub timer_a: HRTIM_TIMA,
    pub timer_b: HRTIM_TIMB,
    pub timer_c: HRTIM_TIMC,
}

static mut HRTIM_PERIPHERALS: MaybeUninit<HrtimPeripherals> = MaybeUninit::uninit();

pub fn init(gpio_b: &mut GPIOB, gpio_c: &mut GPIOC, mut hrtim: HrtimPeripherals) {
    init_capture_timer(&mut hrtim, gpio_b);
    init_phase_timer(&mut hrtim);
    init_output_timer(&mut hrtim, gpio_b);

    cm_interrupt::free(|_| unsafe {
        HRTIM_PERIPHERALS.write(hrtim);
    });

    unsafe { cortex_m::peripheral::NVIC::unmask(stm32g474::Interrupt::HRTIM_TIMA_IRQN) };
}

fn init_capture_timer(hrtim: &mut HrtimPeripherals, gpio_b: &mut GPIOB) {
    hrtim.master.cr().modify(|_, w| w.ckpsc().set(0b101));
    // Setup hrtim eev9 input for falling edge on PB3
    hrtim.common.eecr2().modify(|_, w| {
        w
            .ee9sns().rising()
            .ee9src().src1()
    });
    hrtim.timer_a.cr().modify(|_, w| {
        w.ckpsc().set(CKPSC_1)
    });
    hrtim.timer_a.cr().modify(|_, w| {
        w
            .retrig().set_bit()
            .trstu().set_bit()
            .cont().set_bit()
    });
    hrtim.timer_a.cpt1cr().modify(|_, w| w.exev9cpt().set_bit());
    hrtim.timer_a.rstr().modify(|_, w| {
        w
            .extevnt9().set_bit()
            .cmp2().set_bit()
    });
    hrtim.timer_a.eefr2().modify(|_, w| w.ee9fltr().blank_reset_to_compare1());
    hrtim.timer_a.cmp1r().modify(|_, w| w.cmp().set(FEEDBACK_BLANKING_INTERVAL));
    hrtim.timer_a.cmp2r().modify(|_, w| w.cmp().set(FEEDBACK_VALID_INTERVAL));
    hrtim.timer_a.perr().modify(|_, w| w.per().set(0xF000));
    hrtim.timer_a.cntr().modify(|_, w| w.cnt().set(0));
    hrtim.timer_a.icr().write(|w| w.cpt1c().bit(true));
    hrtim.timer_a.dier().modify(|_, w| w.cpt1ie().set_bit());
    hrtim.master.cr().modify(|_, w| w.tacen().set_bit());

    gpio_b.pupdr().modify(|_, w| w.pupdr3().pull_up());
    gpio_b.afrl().modify(|_, w| w.afrl3().af13());
    gpio_b.moder().modify(|_, w| w.moder3().alternate());
}

fn init_phase_timer(hrtim: &mut HrtimPeripherals) {
    hrtim.timer_b.cr().modify(|_, w| {
        w
            .ckpsc().set(CKPSC_1)
            .trstu().set_bit()
    });
    hrtim.timer_b.rstr().modify(|_, w| { w.extevnt9().set_bit()});
    hrtim.timer_b.perr().modify(|_, w| w.per().set(100));
}

fn init_output_timer(hrtim: &mut HrtimPeripherals, gpio_b: &mut GPIOB) {
    hrtim.timer_c.cr().modify(|_, w| {
        w
            .ckpsc().set(CKPSC_1)
            .trstu().set_bit()
            .cont().set_bit()
            .preen().set_bit()
    });
    hrtim.timer_c.outr().modify(|_, w| {
        w
            .fault1().disabled()
            .fault2().disabled()
            .idlem1().clear_bit()
            .idlem2().clear_bit()
            .pol1().active_high()
            .pol2().active_high()
    });
    hrtim.timer_c.set1r().modify(|_, w| w.update().set_active());
    hrtim.timer_c.rst1r().modify(|_, w| w.cmp1().set_inactive());
    hrtim.timer_c.rst2r().modify(|_, w| w.update().set_inactive());
    hrtim.timer_c.set2r().modify(|_, w| w.cmp1().set_active());
    hrtim.timer_c.perr().modify(|_, w| w.per().set(0xE000));
    hrtim.timer_c.cmp1r().modify(|_, w| w.cmp().set(0x7000));


    hrtim.common.bmcr().modify(|_, w| {
        w
            .tcbm().stopped()
            .bmclk().timer_c()
            .bmom().single_shot()
    });
    hrtim.common.bmcmpr().modify(|_, w| w.bmcmp().set(2));
    hrtim.common.bmper().modify(|_, w| w.bmper().set(0xFFFF));

    hrtim.common.oenr().modify(|_, w| {
        w
            .tc1oen().set_bit()
            .tc2oen().set_bit()
    });
    hrtim.master.cr().modify(|_, w| w.tccen().set_bit());

    gpio_b.afrh().modify(|_, w| {
        w
            .afrh12().af13()
            .afrh13().af13()
    });
    gpio_b.moder().modify(|_, w| {
        w
            .moder12().alternate()
            .moder13().alternate()
    });
    gpio_b.otyper().modify(|_, w| {
        w  
            .ot12().push_pull()
            .ot13().push_pull()
    });
    gpio_b.ospeedr().modify(|_, w| {
        w
            .ospeedr12().high_speed()
            .ospeedr13().high_speed()
    });
}

pub fn begin_open_loop(period: u16) {
    cm_interrupt::free(|_| unsafe {
        let hrtim = HRTIM_PERIPHERALS.assume_init_ref();
        hrtim.timer_c.perr().modify(|_, w| w.per().set(period));
        hrtim.timer_c.cmp1r().modify(|_, w| w.cmp().set(period / 2));
        hrtim.common.bmcr().modify(|_, w| w.bme().set_bit());
        hrtim.timer_c.rstr().modify(|_, w| w.timbcmp1().clear_bit());
        hrtim.timer_c.cr().modify(|_, w| w.cont().set_bit());
        hrtim.common.oenr().modify(|_, w| {
            w
                .tc1oen().set_bit()
                .tc2oen().set_bit()
        });
        hrtim.master.cr().modify(|_, w| w.tccen().set_bit());
    });
}

pub fn stop() {
    cm_interrupt::free(|_| unsafe {
        let hrtim = HRTIM_PERIPHERALS.assume_init_ref();
        hrtim.common.bmtrgr().modify(|_, w| w.tcrst().trigger());
        while !hrtim.common.bmcr().read().bmstat().is_burst() {}
        hrtim.common.odisr().write(|w| {
            w
                .tc1odis().set_bit()
                .tc2odis().set_bit()
        });
        hrtim.timer_c.cr().modify(|_, w| w.cont().clear_bit());
        hrtim.timer_c.rstr().modify(|_, w| w.timbcmp1().clear_bit());
        hrtim.master.cr().modify(|_, w| w.tccen().clear_bit());
        hrtim.common.bmcr().modify(|_, w| w.bme().clear_bit());
        hrtim.common.bmtrgr().modify(|_, w| w.tcrst().no_effect());
    });
}

const FREQ_SAMPLE_NONE: u32 = 0x8000_0000;
static FREQ_SAMPLE: AtomicU32 = AtomicU32::new(FREQ_SAMPLE_NONE);

pub fn poll_feedback_period() -> Option<u16> {
    let value = FREQ_SAMPLE.swap(FREQ_SAMPLE_NONE, Ordering::SeqCst);
    if value == FREQ_SAMPLE_NONE {
        None
    } else {
        Some(value as u16)
    }
}

pub fn read_feedback_period_raw() -> u16 {
    cm_interrupt::free(|_| unsafe {
        let hrtim = HRTIM_PERIPHERALS.assume_init_ref();
        hrtim.timer_a.cpt1r().read().cpt().bits()
    })
}

#[interrupt]
unsafe fn HRTIM_TIMA_IRQN() {
    let hrtim = unsafe { HRTIM_PERIPHERALS.assume_init_mut() };
    if hrtim.timer_a.isr().read().cpt1().bit_is_set() {
        hrtim.timer_a.icr().write(|w| w.cpt1c().bit(true));
        let value = hrtim.timer_a.cpt1r().read().cpt().bits();
        FREQ_SAMPLE.store(value as u32, Ordering::SeqCst);
    }
}

