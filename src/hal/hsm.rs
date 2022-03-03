use super::clint::Clint;
use rustsbi::SbiRet;

#[allow(dead_code)]
pub struct HartStateManager {
    hart_state: HartState,
}

impl HartStateManager {
    pub fn new() -> HartStateManager {
        HartStateManager {
            hart_state: HartState::Started,
        }
    }
}

#[allow(unused_variables)]
impl rustsbi::Hsm for HartStateManager {
    fn hart_start(&self, hartid: usize, start_addr: usize, opaque: usize) -> SbiRet {
        let clint = Clint::new(0x2000000 as *mut u8);
        clint.send_soft(hartid);
        SbiRet::ok(0)
    }
    fn hart_stop(&self, hartid: usize) -> SbiRet {
        SbiRet {
            error: sbi_ret_value::SBI_ERR_NOT_SUPPORTED,
            value: 0,
        }
    }
    fn hart_get_status(&self, hartid: usize) -> SbiRet {
        SbiRet {
            error: sbi_ret_value::SBI_ERR_NOT_SUPPORTED,
            value: 0,
        }
    }
}

#[allow(dead_code)]
mod hart_state_id {
    pub const STARTED: u8 = 0;
    pub const STOPPED: u8 = 1;
    pub const START_PENDING: u8 = 2;
    pub const STOP_PENDING: u8 = 3;
    pub const SUSPENDED: u8 = 4;
    pub const SUSPEND_PENDING: u8 = 5;
    pub const RESUME_PENDING: u8 = 6;
}

#[allow(dead_code)]
pub enum HartState {
    Started,
    Stopped,
    StartPending,
    StopPending,
    Suspended,
    SuspendedPending,
    ResumePending,
}

#[allow(dead_code)]
mod sbi_ret_value {
    pub const SBI_SUCCESS: usize = 0;
    pub const SBI_ERR_FAILED: usize = usize::from_ne_bytes(isize::to_ne_bytes(-1));
    pub const SBI_ERR_NOT_SUPPORTED: usize = usize::from_ne_bytes(isize::to_ne_bytes(-2));
    pub const SBI_ERR_INVALID_PARAM: usize = usize::from_ne_bytes(isize::to_ne_bytes(-3));
    pub const SBI_ERR_DENIED: usize = usize::from_ne_bytes(isize::to_ne_bytes(-4));
    pub const SBI_ERR_INVALID_ADDRESS: usize = usize::from_ne_bytes(isize::to_ne_bytes(-5));
    pub const SBI_ERR_ALREADY_AVAILABLE: usize = usize::from_ne_bytes(isize::to_ne_bytes(-6));
}
