pub fn idle(_ctx: crate::tasks::idle::Context) -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
