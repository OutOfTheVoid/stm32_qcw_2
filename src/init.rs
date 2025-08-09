use stm32g4::stm32g474::Peripherals;
use cortex_m::asm;

pub fn system_init(pac: &mut Peripherals) {
    pac.PWR.cr1().modify(|_, w| unsafe { w.vos().bits(0b01) });
    loop {
        if pac.PWR.sr2().read().vosf().bit_is_clear() {
            break;
        }
    }
    // PWR
    pac.PWR.cr5().modify(|_, w| {
        // enable boost mode
        w.r1mode().clear_bit()
    });
    // FLASH
    pac.FLASH.acr().modify(|_, w| {
        w
            // set flash to 4 wait states
            .latency().wait4()
            // enable prefetch
            .prften().set_bit()
    });
    // CLOCKS
    pac.RCC.cr().modify(|_, w| {
        // turn off the pll, and turn on the hse
        w
            .pllon().clear_bit()
            .hseon().set_bit()
    });
    loop {
        // wait for the hse to be ready and the pll to stop
        let cr = pac.RCC.cr().read();
        if cr.hserdy().is_ready() && cr.pllrdy().is_not_ready() {
            break;
        }
    }
    // set HPRE divider to 2 for an intermediate frequency transisiton
    pac.RCC.cfgr().modify(|_, w| {
        w.hpre().div2()
    });
    // HSE clock is 8 MHz
    pac.RCC.pllcfgr().modify(|_, w| {
        // configure the pll for 320 MHz
        // with pllr div 2 for 160 MHz for SYSCLK
        // with pllp div 6 for 53.33 MHz for ADCCLK
        w
            .pllsrc().hse()
            .plln().div40()
            .pllm().div1()
            .pllr().div2()
            .pllren().clear_bit()
            .pllpdiv().div6()
            .pllpen().clear_bit()
    });
    // enable the pll and wait for it to be ready
    pac.RCC.cr().modify(|_, w| w.pllon().set_bit());
    loop {
        if pac.RCC.cr().read().pllrdy().is_ready() {
            break;
        }
    }
    pac.RCC.pllcfgr().modify(|_, w| {
        w
            .pllren().set_bit()
            .pllpen().set_bit()
    });
    // switch SYSCLK to pllr
    pac.RCC.cfgr().modify(|_, w| { w.sw().pll() });
    // wait at least one µs
    for _ in 0..160 {
        asm::nop();
    }
    // set HPRE to 1 to enable 160 MHz operation
    pac.RCC.cfgr().modify(|_, w| {
        w.hpre().div1()
    });

    // GPIOA - enable and reset
    pac.RCC.ahb2enr().modify(|_, w| w.gpioaen().set_bit());
    pac.RCC.ahb2rstr().modify(|_, w| w.gpioarst().set_bit());
    pac.RCC.ahb2rstr().modify(|_, w| w.gpioarst().clear_bit());

    // GPIOB - enable and reset
    pac.RCC.ahb2enr().modify(|_, w| w.gpioben().set_bit());
    pac.RCC.ahb2rstr().modify(|_, w| w.gpiobrst().set_bit());
    pac.RCC.ahb2rstr().modify(|_, w| w.gpiobrst().clear_bit());

    // GPIOC - enable and reset
    pac.RCC.ahb2enr().modify(|_, w| w.gpiocen().set_bit());
    pac.RCC.ahb2rstr().modify(|_, w| w.gpiocrst().set_bit());
    pac.RCC.ahb2rstr().modify(|_, w| w.gpiocrst().clear_bit());

    // HRTIM - enable and reset
    pac.RCC.apb2enr().modify(|_, w| w.hrtim1en().set_bit());
    pac.RCC.apb2rstr().modify(|_, w| w.hrtim1rst().set_bit());
    pac.RCC.apb2rstr().modify(|_, w| w.hrtim1rst().clear_bit());

    // TIM1 - enable and reset
    pac.RCC.apb2enr().modify(|_, w| w.tim1en().set_bit());
    pac.RCC.apb2rstr().modify(|_, w| w.tim1rst().set_bit());
    pac.RCC.apb2rstr().modify(|_, w| w.tim1rst().clear_bit());

     // TIM8 - enable and reset
    pac.RCC.apb2enr().modify(|_, w| w.tim8en().set_bit());
    pac.RCC.apb2rstr().modify(|_, w| w.tim8rst().set_bit());
    pac.RCC.apb2rstr().modify(|_, w| w.tim8rst().clear_bit());

    // TIM20 - enable and reset
    pac.RCC.apb2enr().modify(|_, w| w.tim20en().set_bit());
    pac.RCC.apb2rstr().modify(|_, w| w.tim20rst().set_bit());
    pac.RCC.apb2rstr().modify(|_, w| w.tim20rst().clear_bit());

}
