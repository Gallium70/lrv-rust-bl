use crate::hal;
use crate::misaligned;

global_asm!(include_str!("rv64.S"));

global_asm!(include_str!("trap.S"));

#[allow(unused)]
#[derive(Debug)]
struct TrapFrame {
    ra: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    satp: usize,
}

impl TrapFrame {
    #[inline]
    fn set_register_xi(&mut self, i: u8, data: usize) {
        match i {
            1 => self.ra = data,
            10 => self.a0 = data,
            11 => self.a1 = data,
            12 => self.a2 = data,
            13 => self.a3 = data,
            14 => self.a4 = data,
            15 => self.a5 = data,
            16 => self.a6 = data,
            17 => self.a7 = data,
            5 => self.t0 = data,
            6 => self.t1 = data,
            7 => self.t2 = data,
            28 => self.t3 = data,
            29 => self.t4 = data,
            30 => self.t5 = data,
            31 => self.t6 = data,
            8 => self.s0 = data,
            9 => self.s1 = data,
            18 => self.s2 = data,
            19 => self.s3 = data,
            20 => self.s4 = data,
            21 => self.s5 = data,
            22 => self.s6 = data,
            23 => self.s7 = data,
            24 => self.s8 = data,
            25 => self.s9 = data,
            26 => self.s10 = data,
            27 => self.s11 = data,
            _ => panic!("invalid set target {}", i),
        }
    }

    #[inline]
    fn set_register_xic(&mut self, i: u8, data: usize) {
        match i {
            0b010 => self.a0 = data,
            0b011 => self.a1 = data,
            0b100 => self.a2 = data,
            0b101 => self.a3 = data,
            0b110 => self.a4 = data,
            0b111 => self.a5 = data,
            0b000 => self.s0 = data,
            0b001 => self.s1 = data,
            _ => panic!("invalid set compressed target {}", i),
        }
    }

    #[inline]
    fn get_register_xi(&self, i: u8) -> usize {
        match i {
            0 => 0,
            1 => self.ra,
            10 => self.a0,
            11 => self.a1,
            12 => self.a2,
            13 => self.a3,
            14 => self.a4,
            15 => self.a5,
            16 => self.a6,
            17 => self.a7,
            5 => self.t0,
            6 => self.t1,
            7 => self.t2,
            28 => self.t3,
            29 => self.t4,
            30 => self.t5,
            31 => self.t6,
            8 => self.s0,
            9 => self.s1,
            18 => self.s2,
            19 => self.s3,
            20 => self.s4,
            21 => self.s5,
            22 => self.s6,
            23 => self.s7,
            24 => self.s8,
            25 => self.s9,
            26 => self.s10,
            27 => self.s11,
            _ => panic!("invalid get target {}", i),
        }
    }

    #[inline]
    fn get_register_xic(&self, i: u8) -> usize {
        match i {
            0b010 => self.a0,
            0b011 => self.a1,
            0b100 => self.a2,
            0b101 => self.a3,
            0b110 => self.a4,
            0b111 => self.a5,
            0b000 => self.s0,
            0b001 => self.s1,
            _ => panic!("invalid get compressed target {}", i),
        }
    }
}

pub fn init_trap() {
    use riscv::register::mtvec::{self, TrapMode};
    extern "C" {
        fn _start_trap();
    }
    unsafe {
        mtvec::write(_start_trap as usize, TrapMode::Direct);
    }
}

pub fn delegate_trap() {
    use riscv::register::{medeleg, mideleg, mie};
    // 把S的中断全部委托给S层
    unsafe {
        mideleg::set_sext();
        mideleg::set_stimer();
        mideleg::set_ssoft();
        // medeleg::set_instruction_misaligned();
        // medeleg::set_breakpoint();
        medeleg::clear_breakpoint();
        medeleg::set_user_env_call();
        // medeleg::set_instruction_page_fault();
        medeleg::set_load_page_fault();
        medeleg::set_store_page_fault();
        // medeleg::set_instruction_fault();
        // medeleg::clear_load_page_fault();
        // medeleg::clear_store_page_fault();
        medeleg::clear_instruction_page_fault();
        medeleg::clear_instruction_fault();
        // medeleg::set_load_fault();
        // medeleg::set_store_fault();
        mie::set_mext();
        // 不打开mie::set_mtimer
        mie::set_msoft();
    }
}

#[export_name = "_start_trap_rust"]
extern "C" fn start_trap_rust(trap_frame: &mut TrapFrame) {
    use misaligned::MemoryUnit;
    use riscv::register::{
        mcause::{self, Exception, Interrupt, Trap},
        mepc, mhartid, mie, mip,
        mstatus::{self, MPP, SPP},
        mtval, scause, sepc, stval, stvec,
    };
    use rustsbi::println;
    let cause = mcause::read().cause();
    match cause {
        Trap::Exception(Exception::SupervisorEnvCall) => {
            let params = [trap_frame.a0, trap_frame.a1, trap_frame.a2, trap_frame.a3, trap_frame.a4];
            // Call RustSBI procedure
            let ans = rustsbi::ecall(trap_frame.a7, trap_frame.a6, params);
            // Return the return value to TrapFrame
            trap_frame.a0 = ans.error;
            trap_frame.a1 = ans.value;
            // Skip ecall instruction
            mepc::write(mepc::read().wrapping_add(4));
        }
        Trap::Interrupt(Interrupt::MachineSoft) => {
            println!("[rustsbi trap handler] Machine Software Interrupt! mhartid: {:016x?}", mhartid::read());
            // 机器软件中断返回给S层
            unsafe {
                mip::set_ssoft();
                mie::clear_msoft();
            }
        }
        Trap::Interrupt(Interrupt::MachineTimer) => {
            // 机器时间中断返回给S层
            unsafe {
                mip::set_stimer();
                mie::clear_mtimer();
            }
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            // println!("[rustsbi trap handler]Illegal instruction!");
            let vaddr = mepc::read();
            // let ins = vaddr;
            let ins = unsafe { misaligned::load_vaddr(vaddr, MemoryUnit::Word, false) };
            if ins & 0xFFFFF07F == 0xC0102073 {
                // rdtime
                let rd = ((ins >> 7) & 0b1_1111) as u8;
                // todo: one instance only
                let clint = hal::Clint::new(0x2000000 as *mut u8);
                let time_usize = clint.get_mtime() as usize;
                trap_frame.set_register_xi(rd, time_usize);
                mepc::write(mepc::read().wrapping_add(4)); // 跳过指令
            } else
            if mstatus::read().mpp() != MPP::Machine {
                // 出现非法指令异常，转发到S特权层
                // invalid instruction, can't emulate, raise to supervisor
                unsafe {
                    // 设置S层异常原因为：非法指令
                    scause::set(scause::Trap::Exception(scause::Exception::IllegalInstruction));
                    // 填写异常指令的指令内容
                    stval::write(mtval::read());
                    // 填写S层需要返回到的地址，这里的mepc会被随后的代码覆盖掉
                    sepc::write(mepc::read());
                    // 设置中断位
                    mstatus::set_mpp(MPP::Supervisor);
                    mstatus::set_spp(SPP::Supervisor);
                    if mstatus::read().sie() {
                        mstatus::set_spie()
                    }
                    mstatus::clear_sie();
                    // 设置返回地址，返回到S层
                    // 注意，无论是Direct还是Vectored模式，所有异常的向量偏移都是0，不需要处理中断向量，跳转到入口地址即可
                    mepc::write(stvec::read().address());
                };
            } else {
                // 真·非法指令异常，是M层出现的
                #[cfg(target_pointer_width = "64")]
                panic!("invalid instruction, mepc: {:016x?}, instruction: {:016x?}", mepc::read(), ins);
                #[cfg(target_pointer_width = "32")]
                panic!("invalid instruction, mepc: {:08x?}, instruction: {:08x?}", mepc::read(), ins);
            }
        }
        Trap::Exception(Exception::LoadMisaligned) => {
            let ins_vaddr = mepc::read();
            let ins = unsafe {
                misaligned::load_vaddr(ins_vaddr, MemoryUnit::HalfWord, false)
            };
            let op = ins & 0b11;
            match op {
                3 => {
                    // not compressed load
                    let ins = unsafe {
                        misaligned::load_vaddr(ins_vaddr, MemoryUnit::Word, false)
                    };
                    let rd = ((ins >> 7) & 0b1_1111) as u8;
                    let mem_unit = MemoryUnit::from((ins >> 12) & 0b11);
                    let load_vaddr = mtval::read();
                    let signed = ((ins >> 14) & 1) == 0;
                    let load_value = unsafe { misaligned::load_vaddr(load_vaddr, mem_unit, signed)};
                    trap_frame.set_register_xi(rd, load_value);
                    // println!("[rustsbi trap handler] Load misaligned! epc: {:016x?}, ins: {:016x}, addr: {:016x}", ins_vaddr , ins, load_vaddr);
                    // println!("[rustsbi trap handler] rd: {:?} value: {:016x}", rd,load_value);
                    mepc::write(mepc::read().wrapping_add(4)); // 跳过指令
                }
                2 => {
                    // compressed, sp based
                    let rd = ((ins >> 7) & 0b1_1111) as u8;
                    let mem_unit = MemoryUnit::from((ins >> 13) & 0b11); // 只考虑 RV64IC
                    let load_vaddr = mtval::read();
                    let signed = true;
                    let load_value = unsafe { misaligned::load_vaddr(load_vaddr, mem_unit, signed)};
                    trap_frame.set_register_xi(rd, load_value);
                    // println!("[rustsbi trap handler] Load misaligned! epc: {:016x?}, ins: {:016x}, addr: {:016x}", ins_vaddr , ins, load_vaddr);
                    // println!("[rustsbi trap handler] rd: {:?} value: {:016x}", rd,load_value);
                    mepc::write(mepc::read().wrapping_add(2)); // 跳过指令
                }
                0 => {
                    // compressed
                    // 只考虑 RV64IC，不考虑浮点
                    let rd = ((ins >> 2) & 0b111) as u8;
                    let mem_unit = MemoryUnit::from((ins >> 13) & 0b11);
                    let load_vaddr = mtval::read();
                    let signed = true;
                    let load_value = unsafe { misaligned::load_vaddr(load_vaddr, mem_unit, signed)};
                    trap_frame.set_register_xic(rd, load_value);
                    // println!("[rustsbi trap handler] Load misaligned! epc: {:016x?}, ins: {:016x}, addr: {:016x}", ins_vaddr , ins, load_vaddr);
                    // println!("[rustsbi trap handler] rd: {:?} value: {:016x}", rd,load_value);
                    mepc::write(mepc::read().wrapping_add(2)); // 跳过指令
                }
                _ => {
                    panic!("[rustsbi trap handler] Invalid load misaligned! epc: {:016x?}, ins: {:016x}", ins_vaddr , ins);
                }
            }
        }
        Trap::Exception(Exception::StoreMisaligned) => {
            let ins_vaddr = mepc::read();
            let ins = unsafe {
                misaligned::load_vaddr(ins_vaddr, MemoryUnit::HalfWord, false)
            };
            let op = ins & 0b11;
            match op {
                3 => {
                    // not compressed load
                    let ins = unsafe {
                        misaligned::load_vaddr(ins_vaddr, MemoryUnit::Word, false)
                    };
                    let rs = ((ins >> 20) & 0b1_1111) as u8;
                    let store_value = trap_frame.get_register_xi(rs);
                    let mem_unit = MemoryUnit::from((ins >> 12) & 0b11);
                    let store_vaddr = mtval::read();
                    unsafe { misaligned::store_vaddr(store_vaddr, mem_unit, store_value)};
                    // println!("[rustsbi trap handler] Store misaligned! epc: {:016x?}, ins: {:016x}, addr: {:016x}", ins_vaddr , ins, store_vaddr);
                    // println!("[rustsbi trap handler] rs: {:?} value: {:016x}", rs,store_value);
                    mepc::write(mepc::read().wrapping_add(4)); // 跳过指令
                }
                2 => {
                    // compressed, sp based
                    let rs = ((ins >> 2) & 0b1_1111) as u8;
                    let store_value = trap_frame.get_register_xi(rs);
                    let mem_unit = MemoryUnit::from((ins >> 13) & 0b11); // 只考虑 RV64IC
                    let store_vaddr = mtval::read();
                    unsafe { misaligned::store_vaddr(store_vaddr, mem_unit, store_value)};
                    // println!("[rustsbi trap handler] Store misaligned! epc: {:016x?}, ins: {:016x}, addr: {:016x}", ins_vaddr , ins, store_vaddr);
                    // println!("[rustsbi trap handler] rs: {:?} value: {:016x}", rs,store_value);
                    mepc::write(mepc::read().wrapping_add(2)); // 跳过指令
                }
                0 => {
                    // compressed
                    // 只考虑 RV64IC，不考虑浮点
                    let rs = ((ins >> 2) & 0b111) as u8;
                    let store_value = trap_frame.get_register_xic(rs);
                    let mem_unit = MemoryUnit::from((ins >> 13) & 0b11);
                    let store_vaddr = mtval::read();
                    unsafe { misaligned::store_vaddr(store_vaddr, mem_unit, store_value)};
                    // println!("[rustsbi trap handler] Store misaligned! epc: {:016x?}, ins: {:016x}, addr: {:016x}", ins_vaddr , ins, store_vaddr);
                    // println!("[rustsbi trap handler] rs: {:?} value: {:016x}", rs,store_value);
                    mepc::write(mepc::read().wrapping_add(2)); // 跳过指令
                }
                _ => {
                    panic!("[rustsbi trap handler] Invalid Store misaligned! epc: {:016x?}, ins: {:016x}", ins_vaddr , ins);
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        cause => panic!(
            "Unhandled exception! mcause: {:?}, mepc: {:016x?}, mtval: {:016x?}, mstatus: {:016x?}, trap frame: {:p}, {:x?}",
            cause,
            mepc::read(),
            mtval::read(),
            mstatus::read(),
            &trap_frame as *const _,
            trap_frame
        ),
        #[cfg(target_pointer_width = "32")]
        cause => panic!(
            "Unhandled exception! mcause: {:?}, mepc: {:08x?}, mtval: {:08x?}, mstatus: {:08x?}, trap frame: {:x?}",
            cause,
            mepc::read(),
            mtval::read(),
            mstatus::read(),
            trap_frame
        ),
    }
}
