use rtt_target::rprintln;

pub struct Display {
    sleep: bool,
}

#[repr(u8)]
enum DisplayCommand {
    SleepIn = 0x10,
    SleepOut = 0x11,
}

impl Display {
    fn send_command(&mut self, command: DisplayCommand) {
        let value: u8 = command as u8;
        rprintln!("DISPLAY SEND {}", value);
    }

    pub fn set_sleep(&mut self, value: bool) {
        self.send_command(if value { DisplayCommand::SleepIn } else { DisplayCommand::SleepOut });

        self.sleep = value;
    }

    pub fn default() -> Display {
        Display {
            sleep: true,
        }
    }
}
