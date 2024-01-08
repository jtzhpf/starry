use super::context::TrapFrame;
use loongarch64::register::estat::{self, Exception, Trap};

#[cfg(feature = "monolithic")]
use super::enable_irqs;

#[cfg(feature = "monolithic")]
use super::disable_irqs;

#[cfg(feature = "monolithic")]
use crate::trap::handle_syscall;

#[cfg(feature = "monolithic")]
use page_table_entry::MappingFlags;

#[cfg(feature = "monolithic")]
use crate::trap::handle_page_fault;

#[cfg(feature = "signal")]
use crate::trap::handle_signal;

core::arch::global_asm!(
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
);

fn handle_breakpoint(era: &mut usize) {
    debug!("Exception(Breakpoint) @ {:#x} ", era);
    *era += 4;
}

#[no_mangle]
fn loongarch64_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    let estat = estat::read();
    let DEBUG:bool = false;
    if (estat.ecode() != 0) && (estat.ecode() != 0xb) && DEBUG {
        info!("Trap era : 0x{:x}",tf.era);
        info!("Trap badv: 0x{:x}",tf.badv);
        info!("Trap sp  : 0x{:x}",tf.regs[3]);
        info!("Trap ra  : 0x{:x}",tf.regs[1]);
    }
    
    match estat.cause() {
        Trap::Exception(Exception::Breakpoint) => handle_breakpoint(&mut tf.era),
        Trap::Interrupt(_) => {
            let irq_num: usize = estat.is().trailing_zeros() as usize;
            crate::trap::handle_irq_extern(irq_num)
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(Exception::Syscall) => {
            enable_irqs();
            // jump to next instruction anyway
            tf.era += 4;
            // get system call return value
            info!("Syscall num:{}", tf.regs[11]);
            let result = handle_syscall(
                tf.regs[11],
                [
                    tf.regs[4], tf.regs[5], tf.regs[6], tf.regs[7], tf.regs[8], tf.regs[9],
                ],
            );
            info!("Syscall Exit");
            // cx is changed during sys_exec, so we have to call it again
            tf.regs[4] = result as usize;
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(Exception::FetchPageFault) => {
            let addr = tf.badv;
            if !from_user {
                unimplemented!(
                    "FetchPageFault from kernel, addr: {:X}, era: {:X}",
                    addr,
                    tf.era
                );
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::EXECUTE, tf);
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(Exception::LoadPageFault) => {
            let addr = tf.badv;
            if !from_user {
                error!("LoadPageFault from kernel, addr: {:#x}", addr);
                unimplemented!("LoadPageFault fault from kernel");
            }
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::READ, tf);
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(Exception::StorePageFault) => {
            if !from_user {
                error!(
                    "StorePageFault from kernel, addr: {:#x} era:{:X}",
                    tf.badv,
                    tf.era
                );
                unimplemented!("StorePageFault from kernel");
            }
            let addr = tf.badv;
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::WRITE, tf);
        }


        #[cfg(feature = "monolithic")]
        Trap::Exception(Exception::PageModifyFault) => {
            let addr = tf.badv;
            handle_page_fault(addr.into(), MappingFlags::USER | MappingFlags::WRITE | MappingFlags::DIRTY, tf);
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(Exception::PagePrivilegeIllegal) => {
            let addr = tf.badv;
            if !from_user {
                error!(
                    "PagePrivilegeIllegal from kernel, addr: {:#x} era:{:X}",
                    tf.badv,
                    tf.era
                );
                unimplemented!("PagePrivilegeIllegal from kernel");
            };
            handle_page_fault(addr.into(), MappingFlags::USER, tf);
        }

        #[cfg(feature = "monolithic")]
        Trap::Exception(Exception::InstructionNotExist) => {
            let ip =  tf.era as u64;
            let inst = unsafe {*((ip) as *mut u32)};
            info!("Illegal Instruction: 0x{:x}, {:x}", ip, inst);
            panic!("Exit")
        }

        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
                estat.cause(),
                tf.era,
                tf
            );
        }
    }

    #[cfg(feature = "signal")]
    if from_user == true {
        handle_signal();
    }

    #[cfg(feature = "monolithic")]
    // 在保证将寄存器都存储好之后，再开启中断
    // 否则此时会因为写入csr寄存器过程中出现中断，导致出现异常
    disable_irqs();
}
