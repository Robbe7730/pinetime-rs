use rtic::mutex_prelude::TupleExt02;

pub fn self_test(ctx: crate::tasks::self_test::Context) {
    (
        ctx.shared.flash,
        ctx.shared.mcuboot,
    ).lock(|flash, mcuboot| {
        flash.self_test().unwrap();

        rtt_target::rprintln!("Selftest succeeded, marking image as valid");
        unsafe { mcuboot.mark_valid() }
    });
}
