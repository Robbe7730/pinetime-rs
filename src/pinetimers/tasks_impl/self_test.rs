use rtic::Mutex;

pub fn self_test(mut ctx: crate::tasks::self_test::Context) {
    ctx.shared.flash.lock(|flash| {
        flash.self_test().unwrap();
    });
}
