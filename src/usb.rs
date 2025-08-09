use cortex_m::asm;
use stm32g4::stm32g474::Peripherals;

pub fn setup(pac: &mut Peripherals) {
    pac.USB.cntr().modify(|_, w| w.pdwn().disabled());
    // wait 10ms for the phy to come online
    for _ in 0..170_000_0 {
        asm::nop();
    }
    pac.CRS.cfgr().modify(|_, w| {
        let f_target = 48_000_000u32;
        let f_sync = 1_000u32;
        let sync_ratio = (f_target / f_sync) as u16;
        let step_numerator = 13u32;
        let step_denominator = 10000u32;
        w
            .reload().set(sync_ratio - 1)
            .felim().set(((((sync_ratio as u32) * step_numerator) / step_denominator) / 2) as u8)
            .syncsrc().usb_sof()
    });
    pac.CRS.cr().modify(|_, w| {
        w
            .autotrimen().set_bit()
            .cen().set_bit()
    });
}