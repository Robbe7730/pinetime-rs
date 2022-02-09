use rtic::Mutex;

pub fn validate(mut ctx: crate::tasks::validate::Context) {
    ctx.shared.mcuboot.lock(|mcuboot| {
        if !mcuboot.footer.is_valid {
            crate::tasks::self_test::spawn().unwrap();
        }
    });

    crate::tasks::display_init::spawn().unwrap();
}
