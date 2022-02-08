mod header;
mod footer;

use header::MCUBootHeader;
use footer::MCUBootFooter;

#[derive(Debug)]
pub struct MCUBoot {
    pub header: MCUBootHeader,
    pub footer: MCUBootFooter,
}

impl MCUBoot {
    pub fn get() -> Self {
        MCUBoot {
            header: MCUBootHeader::get(),
            footer: MCUBootFooter::get(),
        }
    }

    pub unsafe fn mark_valid(&mut self) {
        self.footer.mark_valid();
    }

    pub fn is_valid(&self) -> bool {
        self.footer.is_valid()
    }
}
