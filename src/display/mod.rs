// Zero-sized types
pub struct PoweredOn;
pub struct PoweredOff;

pub struct Display<POWERED> {
    powered: POWERED,
}

impl Display<PoweredOn> {
    pub fn poweroff(self) -> Display<PoweredOff> {
        println!("POWER OFF");
        Display {
            powered: PoweredOff
        }
    }
}

impl Display<PoweredOff> {
    pub fn poweron(self) -> Display<PoweredOn> {
        println!("POWER ON");
        Display {
            powered: PoweredOn
        }
    }
}

pub fn get_display() -> Display<PoweredOff> {
    return Display {
        powered: PoweredOff
    }
}
