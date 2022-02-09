use rtic::mutex_prelude::TupleExt03;

pub fn self_test(ctx: crate::tasks::self_test::Context) {
    (
        ctx.shared.internal_flash,
        ctx.shared.external_flash,
        ctx.shared.mcuboot,
    ).lock(|internal_flash, external_flash, mcuboot| {
        external_flash.self_test().unwrap();

        rtt_target::rprintln!("Selftest succeeded, marking image as valid");
        mcuboot.mark_valid(internal_flash)
    });
}
