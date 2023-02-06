use core::arch::asm;

use aarch64_cpu::Writeable;
use tracing::{info, info_span};

use crate::{arch::aarch64::regs::ExceptionClass, syscall};

use super::context::Registers;

// the first one in the table == base address of vector table
extern "C" {
    fn current_el_sp_el0_sync();
}

pub fn init_interrupts() {
    use aarch64_cpu::registers::VBAR_EL1;
    VBAR_EL1.set(current_el_sp_el0_sync as *const fn() as u64);
}

// allow dead code because rustc thinks we never construct these variants; we construct them in
// assembly in interrupts.S
// same for InterruptType and InterruptArgs
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug)]
enum InterruptSource {
    CurrentElSpEl0 = 0,
    CurrentElSpElx = 1,
    LowerElAa64 = 2,
    LowerElAa32 = 3,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Debug)]
enum InterruptType {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[no_mangle]
extern "C" fn demux_interrupt(regs: &Registers, source: InterruptSource, ty: InterruptType) {
    let link: u64;
    unsafe { asm!("mrs {0}, ELR_EL1", out(reg) link) };
    let syndrome = super::regs::ExceptionSyndrome::get();
    let span = info_span!("interrupt handler", src=?source, ?ty, ?syndrome, link);
    let _guard = span.enter();
    info!(target: "interrupt handler", "hello from interrupt handler");

    if syndrome.cause == ExceptionClass::SvcAa64 {
        syscall::dispatch(syndrome.iss as usize, &mut regs.x[0..8].try_into().unwrap());
    } else {
        let sp: u64;
        unsafe {
            match source {
                InterruptSource::CurrentElSpEl0 => unreachable!("we don't configure it like that"),
                InterruptSource::CurrentElSpElx => asm!("mov {0}, sp", out(reg) sp),
                InterruptSource::LowerElAa64 => asm!("mrs {0}, SP_EL0", out(reg) sp),
                InterruptSource::LowerElAa32 => unreachable!("no support for aa32"),
            }
        }
        tracing::error!(
            "unhandled exception at {link:#018x}!\n\n{} sp: {:#018x}",
            regs,
            sp,
        );
    }
}
