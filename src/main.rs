#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(global_asm)]

mod hal;
mod misaligned;
mod trap;

#[cfg(not(test))]
use core::alloc::Layout;
#[cfg(not(test))]
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;

use rustsbi::{print, println};

use riscv::register::{medeleg, mhartid, mideleg, mie, mip};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

global_asm!(include_str!("entry.S"));

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let hart_id = mhartid::read();
    // 输出的信息大概是“[rustsbi-panic] hart 0 panicked at ...”
    println!("[rustsbi-panic] hart {} {}", hart_id, info);
    println!("[rustsbi-panic] system shutdown scheduled due to RustSBI panic");
    // use rustsbi::Reset;
    // hal::Reset.system_reset(
    //     rustsbi::reset::RESET_TYPE_SHUTDOWN,
    //     rustsbi::reset::RESET_REASON_SYSTEM_FAILURE,
    // );
    loop {}
}

#[cfg(not(test))]
#[alloc_error_handler]
fn oom(_layout: Layout) -> ! {
    loop {}
}

lazy_static::lazy_static! {
    // 最大的硬件线程编号；只在启动时写入，跨核软中断发生时读取
    pub static ref MAX_HART_ID: spin::Mutex<usize> =
        spin::Mutex::new(compiled_max_hartid());
}

// #[export_name = "_mp_hook"]
pub extern "C" fn mp_hook() -> bool {
    let hartid = mhartid::read();
    if hartid == 0 {
        true
    } else {
        use hal::Clint;
        use riscv::asm::wfi;
        unsafe {
            let mut clint = Clint::new(0x200_0000 as *mut u8);
            // Clear IPI
            clint.clear_soft(hartid);
            // Start listening for software interrupts
            mie::set_msoft();

            loop {
                wfi();
                if mip::read().msoft() {
                    break;
                }
            }

            // Stop listening for software interrupts
            mie::clear_msoft();
            // Clear IPI
            clint.clear_soft(hartid);
        }
        false
    }
}

#[export_name = "main"]
extern "C" fn main(_mhartid: usize) -> ! {
    // dtb_pa is put into a1 register on qemu boot
    // Ref: https://github.com/qemu/qemu/blob/aeb07b5f6e69ce93afea71027325e3e7a22d2149/hw/riscv/boot.c#L243

    if mp_hook() {
        // init
    }

    /* setup trap */

    trap::init_trap();
    /* main function start */

    extern "C" {
        static mut _sheap: u8;
        static mut _eheap: u8;
        static external_dtb: usize;
    }
    let dtb_pa = unsafe { &external_dtb } as *const _ as usize;
    if mhartid::read() == 0 {
        let sheap = unsafe { &mut _sheap } as *mut _ as usize;
        let eheap = unsafe { &mut _eheap } as *mut _ as usize;
        let heap_size = eheap - sheap;
        unsafe {
            ALLOCATOR.lock().init(sheap, heap_size);
        }

        // 其实这些参数不用提供，直接通过pac库生成
        let serial = hal::Uartlite::new(0x60000000, 0);
        // use through macro
        use rustsbi::legacy_stdio::init_legacy_stdio_embedded_hal;
        init_legacy_stdio_embedded_hal(serial);
        println!("[rustsbi] ----****----****----****----****----****----****----");
        // println!("[rustsbi] Serial initialized.");

        let clint = hal::Clint::new(0x2000000 as *mut u8);
        use rustsbi::init_ipi;
        init_ipi(clint);
        // println!("[rustsbi] IPI initialized.");

        // todo: do not create two instances
        let clint = hal::Clint::new(0x2000000 as *mut u8);
        use rustsbi::init_timer;
        init_timer(clint);
        let mut clint = hal::Clint::new(0x2000000 as *mut u8);
        clint.set_timer(0, u64::MAX);
        // println!("[rustsbi] Timer initialized.");

        use rustsbi::init_reset;
        init_reset(hal::Reset);
        // println!("[rustsbi] Reset initialized.");

        let hart_state_manager = hal::HartStateManager::new();
        use rustsbi::init_hsm;
        init_hsm(hart_state_manager);
    }

    trap::delegate_trap();
    if mhartid::read() == 0 {
        use riscv::register::misa::{self, MXL};
        println!("[rustsbi] RustSBI version {}", rustsbi::VERSION);
        println!("{}", rustsbi::LOGO);
        println!(
            "[rustsbi] Platform: ZCU102 (Version {})",
            env!("CARGO_PKG_VERSION")
        );
        let isa = misa::read();
        if let Some(isa) = isa {
            let mxl_str = match isa.mxl() {
                MXL::XLEN32 => "RV32",
                MXL::XLEN64 => "RV64",
                MXL::XLEN128 => "RV128",
            };
            print!("[rustsbi] misa: {}", mxl_str);
            for ext in 'A'..='Z' {
                if isa.has_extension(ext) {
                    print!("{}", ext);
                }
            }
            println!("");
        }
        println!("[rustsbi] mideleg: {:#x}", mideleg::read().bits());
        println!("[rustsbi] medeleg: {:#x}", medeleg::read().bits());
        let mut guard = MAX_HART_ID.lock();
        *guard = unsafe { count_harts(dtb_pa) };
        drop(guard);
        println!("[rustsbi] Kernel entry: 0x100200000");
    }

    init_pmp();
    unsafe {
        use riscv::register::{
            mcounteren, mepc,
            mstatus::{self, MPP},
            sstatus,
        };
        // mstatus::clear_mpie();
        mstatus::set_mpie();
        mstatus::set_sum();
        mcounteren::set_cy();
        mcounteren::set_tm();
        mcounteren::set_ir();
        sstatus::set_sum();
        mstatus::set_mpp(MPP::Supervisor);
        println!("[rustsbi] entering supervisor mode...");
        mepc::write(s_mode_start as usize);
        rustsbi::enter_privileged(mhartid::read(), dtb_pa)
    }
}

#[naked]
#[link_section = ".text"] // must add link section for all naked functions
unsafe extern "C" fn s_mode_start() -> ! {
    asm!(
        "
.align 2
1:  auipc ra, %pcrel_hi(1f)
    ld ra, %pcrel_lo(1b)(ra)
    jr ra
.align  3
1:  .dword 0x100200000
    ",
        options(noreturn)
    )
}

fn init_pmp() {
    use riscv::asm;
    use riscv::register::{pmpaddr0, pmpcfg0};
    pmpcfg0::write(0x1f);
    pmpaddr0::write(usize::MAX);
    unsafe {
        asm!("csrwi satp, 0x0");
        asm::sfence_vma_all();
    }
}

unsafe fn count_harts(dtb_pa: usize) -> usize {
    println!("[rustsbi-dtb] dtb_pa addr: {:#x}", dtb_pa);
    use device_tree::{DeviceTree, Node};
    const DEVICE_TREE_MAGIC: u32 = 0xD00DFEED;
    // 遍历“cpu_map”结构
    // 这个结构的子结构是“处理核簇”（cluster）
    // 每个“处理核簇”的子结构分别表示一个处理器核
    fn enumerate_cpu_map(cpu_map_node: &Node) -> usize {
        let mut tot = 0;
        for cluster_node in cpu_map_node.children.iter() {
            let name = &cluster_node.name;
            let count = cluster_node.children.iter().count();
            // 会输出：Hart count: cluster0 with 2 cores
            // 在justfile的“threads := "2"”处更改
            println!("[rustsbi-dtb] Hart count: {} with {} cores", name, count);
            tot += count;
        }
        tot
    }
    #[repr(C)]
    struct DtbHeader {
        magic: u32,
        size: u32,
    }
    let header = &*(dtb_pa as *const DtbHeader);
    // from_be 是大小端序的转换（from big endian）
    let magic = u32::from_be(header.magic);
    if magic == DEVICE_TREE_MAGIC {
        let size = u32::from_be(header.size);
        // 拷贝数据，加载并遍历
        let data = core::slice::from_raw_parts(dtb_pa as *const u8, size as usize);
        if let Ok(dt) = DeviceTree::load(data) {
            if let Some(cpu_map) = dt.find("/cpus/cpu-map") {
                return enumerate_cpu_map(cpu_map);
            }
        }
    }
    // 如果DTB的结构不对（读不到/cpus/cpu-map），返回默认的8个核
    let ans = compiled_max_hartid();
    println!("[rustsbi-dtb] Could not read '/cpus/cpu-map' from 'dtb_pa' device tree root; assuming {} cores", ans);
    ans
}

#[inline]
fn compiled_max_hartid() -> usize {
    let ans;
    unsafe {
        asm!("
        lui     {ans}, %hi(_max_hart_id)
        add     {ans}, {ans}, %lo(_max_hart_id)
    ", ans = out(reg) ans)
    };
    ans
}