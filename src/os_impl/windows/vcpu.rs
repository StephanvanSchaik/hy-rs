use crate::error::Error;
use crate::vcpu::ExitReason;
use std::ops::Deref;
use std::sync::Arc;
use super::bindings::*;
use super::vm::PartitionHandle;

pub struct Vcpu {
    pub(crate) handle: Arc<PartitionHandle>,
    pub(crate) id: u32,
}

impl Vcpu {
    pub fn run(&mut self) -> Result<ExitReason, Error> {
        let mut context = WHV_RUN_VP_EXIT_CONTEXT::default();

        unsafe {
            WHvRunVirtualProcessor(
                self.handle.deref().0,
                self.id,
                &mut context as *mut WHV_RUN_VP_EXIT_CONTEXT as *mut std::ffi::c_void,
                std::mem::size_of::<WHV_RUN_VP_EXIT_CONTEXT>() as u32,
            )
        }?;

        let exit_reason = match context.ExitReason {
            super::bindings::WHvRunVpExitReasonMemoryAccess => {
                let info = unsafe { context.Anonymous.MemoryAccess };

                ExitReason::InvalidMemoryAccess {
                    gpa: info.Gpa,
                    gva: info.Gva as usize,
                }
            }
            super::bindings::WHvRunVpExitReasonUnrecoverableException =>
                ExitReason::UnhandledException,
            super::bindings::WHvRunVpExitReasonX64Halt =>
                ExitReason::Halted,
            exit_reason => {
                println!("{:?}", exit_reason);
                ExitReason::Unknown
            }
        };

        Ok(exit_reason)
    }
}

impl Drop for Vcpu {
    fn drop(&mut self) {
        let _ = unsafe {
            WHvDeleteVirtualProcessor(
                self.handle.deref().0,
                self.id,
            )
        };
    }
}

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::{
    ControlRegister, CpuRegs, DescriptorTable, DescriptorTableRegister, Segment, SegmentRegister,
    Register,
};

#[cfg(target_arch = "x86_64")]
impl CpuRegs for Vcpu {
    fn get_registers(
        &self,
        registers: &[Register],
    ) -> Result<Vec<u64>, Error> {
        let registers: Vec<WHV_REGISTER_NAME> = registers
            .into_iter()
            .map(|register| match register {
                Register::Rax => WHvX64RegisterRax,
                Register::Rcx => WHvX64RegisterRcx,
                Register::Rdx => WHvX64RegisterRdx,
                Register::Rbx => WHvX64RegisterRbx,
                Register::Rsp => WHvX64RegisterRsp,
                Register::Rbp => WHvX64RegisterRbp,
                Register::Rsi => WHvX64RegisterRsi,
                Register::Rdi => WHvX64RegisterRdi,
                Register::R8 => WHvX64RegisterR8,
                Register::R9 => WHvX64RegisterR9,
                Register::R10 => WHvX64RegisterR10,
                Register::R11 => WHvX64RegisterR11,
                Register::R12 => WHvX64RegisterR12,
                Register::R13 => WHvX64RegisterR13,
                Register::R14 => WHvX64RegisterR14,
                Register::R15 => WHvX64RegisterR15,
                Register::Rip => WHvX64RegisterRip,
                Register::Rflags => WHvX64RegisterRflags,
            })
            .collect();

        let mut values = vec![WHV_REGISTER_VALUE::default(); registers.len()];

        unsafe {
            WHvGetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                registers.as_ptr(),
                registers.len() as u32,
                values.as_mut_ptr(),
            )
        }?;

        Ok(values
            .into_iter()
            .map(|value| unsafe { value.Reg64 })
            .collect())
    }
    
    fn set_registers(
        &mut self,
        registers: &[Register],
        values: &[u64],
    ) -> Result<(), Error> {
        let registers: Vec<WHV_REGISTER_NAME> = registers
            .into_iter()
            .map(|register| match register {
                Register::Rax => WHvX64RegisterRax,
                Register::Rcx => WHvX64RegisterRcx,
                Register::Rdx => WHvX64RegisterRdx,
                Register::Rbx => WHvX64RegisterRbx,
                Register::Rsp => WHvX64RegisterRsp,
                Register::Rbp => WHvX64RegisterRbp,
                Register::Rsi => WHvX64RegisterRsi,
                Register::Rdi => WHvX64RegisterRdi,
                Register::R8 => WHvX64RegisterR8,
                Register::R9 => WHvX64RegisterR9,
                Register::R10 => WHvX64RegisterR10,
                Register::R11 => WHvX64RegisterR11,
                Register::R12 => WHvX64RegisterR12,
                Register::R13 => WHvX64RegisterR13,
                Register::R14 => WHvX64RegisterR14,
                Register::R15 => WHvX64RegisterR15,
                Register::Rip => WHvX64RegisterRip,
                Register::Rflags => WHvX64RegisterRflags,
            })
            .collect();

        let values: Vec<WHV_REGISTER_VALUE> = values
            .into_iter()
            .map(|value| WHV_REGISTER_VALUE {
                Reg64: *value,
            })
            .collect();

        unsafe {
            WHvSetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                registers.as_ptr(),
                registers.len() as u32,
                values.as_ptr(),
            )
        }?;

        Ok(())
    }

    fn get_control_registers(
        &self,
        registers: &[ControlRegister],
    ) -> Result<Vec<u64>, Error> {
        let mut regs: Vec<WHV_REGISTER_NAME> = vec![];
        let mut indices = vec![];
        
        for (index, register) in registers.iter().enumerate() {
            let register = match *register {
                ControlRegister::Cr0 => WHvX64RegisterCr0,
                ControlRegister::Cr2 => WHvX64RegisterCr2,
                ControlRegister::Cr3 => WHvX64RegisterCr3,
                ControlRegister::Cr4 => WHvX64RegisterCr4,
                ControlRegister::Cr8 => WHvX64RegisterCr8,
                _ => {
                    indices.push(index);
                    continue;
                }
            };

            regs.push(register);
        }

        let mut values = vec![WHV_REGISTER_VALUE::default(); regs.len()];

        unsafe {
            WHvGetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                regs.as_ptr(),
                regs.len() as u32,
                values.as_mut_ptr(),
            )
        }?;

        let mut values: Vec<u64> = values
            .into_iter()
            .map(|value| unsafe { value.Reg64 })
            .collect();

        for index in indices {
            values.insert(index, 0);
        }

        Ok(values)
    }

    fn set_control_registers(
        &mut self,
        registers: &[ControlRegister],
        values: &[u64],
    ) -> Result<(), Error> {
        let mut regs: Vec<WHV_REGISTER_NAME> = vec![];
        let mut vals = vec![];

        for (register, value) in registers.iter().zip(values.iter()) {
            let register = match register {
                ControlRegister::Cr0 => WHvX64RegisterCr0,
                ControlRegister::Cr2 => WHvX64RegisterCr2,
                ControlRegister::Cr3 => WHvX64RegisterCr3,
                ControlRegister::Cr4 => WHvX64RegisterCr4,
                ControlRegister::Cr8 => WHvX64RegisterCr8,
                _ => continue,
            };

            regs.push(register);
            vals.push(WHV_REGISTER_VALUE {
                Reg64: *value,
            });
        }

        unsafe {
            WHvSetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                regs.as_ptr(),
                regs.len() as u32,
                vals.as_ptr(),
            )
        }?;

        Ok(())
    }

    fn get_msrs(
        &self,
        registers: &[u32],
    ) -> Result<Vec<u64>, Error> {
        let mut regs: Vec<WHV_REGISTER_NAME> = vec![];
        let mut indices = vec![];

        for (index, register) in registers.iter().enumerate() {
            let register = match *register {
                crate::arch::x86_64::MSR_IA32_EFER =>
                    WHvX64RegisterEfer,
                crate::arch::x86_64::MSR_IA32_KERNEL_GS_BASE =>
                    WHvX64RegisterKernelGsBase,
                crate::arch::x86_64::MSR_IA32_SYSENTER_CS =>
                    WHvX64RegisterSysenterCs,
                crate::arch::x86_64::MSR_IA32_SYSENTER_EIP =>
                    WHvX64RegisterSysenterEip,
                crate::arch::x86_64::MSR_IA32_SYSENTER_ESP =>
                    WHvX64RegisterSysenterEsp,
                crate::arch::x86_64::MSR_IA32_STAR =>
                    WHvX64RegisterStar,
                crate::arch::x86_64::MSR_IA32_LSTAR =>
                    WHvX64RegisterLstar,
                crate::arch::x86_64::MSR_IA32_CSTAR =>
                    WHvX64RegisterCstar,
                crate::arch::x86_64::MSR_IA32_SYSCALL_MASK =>
                    WHvX64RegisterSfmask,
                _ => {
                    indices.push(index);
                    continue;
                }
            };

            regs.push(register);
        }

        let mut values = vec![WHV_REGISTER_VALUE::default(); regs.len()];

        unsafe {
            WHvGetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                regs.as_ptr(),
                regs.len() as u32,
                values.as_mut_ptr(),
            )
        }?;

        let mut values: Vec<u64> = values
            .into_iter()
            .map(|value| unsafe { value.Reg64 })
            .collect();

        for index in indices {
            values.insert(index, 0);
        }

        Ok(values)
    }

    fn set_msrs(
        &mut self,
        registers: &[u32],
        values: &[u64],
    ) -> Result<(), Error> {
        let mut regs: Vec<WHV_REGISTER_NAME> = vec![];
        let mut vals: Vec<WHV_REGISTER_VALUE> = vec![];

        for (register, value) in registers.iter().zip(values.iter()) {
            let register = match *register {
                crate::arch::x86_64::MSR_IA32_EFER =>
                    WHvX64RegisterEfer,
                crate::arch::x86_64::MSR_IA32_KERNEL_GS_BASE =>
                    WHvX64RegisterKernelGsBase,
                crate::arch::x86_64::MSR_IA32_SYSENTER_CS =>
                    WHvX64RegisterSysenterCs,
                crate::arch::x86_64::MSR_IA32_SYSENTER_EIP =>
                    WHvX64RegisterSysenterEip,
                crate::arch::x86_64::MSR_IA32_SYSENTER_ESP =>
                    WHvX64RegisterSysenterEsp,
                crate::arch::x86_64::MSR_IA32_STAR =>
                    WHvX64RegisterStar,
                crate::arch::x86_64::MSR_IA32_LSTAR =>
                    WHvX64RegisterLstar,
                crate::arch::x86_64::MSR_IA32_CSTAR =>
                    WHvX64RegisterCstar,
                crate::arch::x86_64::MSR_IA32_SYSCALL_MASK =>
                    WHvX64RegisterSfmask,
                _ => continue,
            };

            regs.push(register);
            vals.push(WHV_REGISTER_VALUE {
                Reg64: *value
            });
        }

        unsafe {
            WHvSetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                regs.as_ptr(),
                regs.len() as u32,
                vals.as_ptr(),
            )
        }?;

        Ok(())
    }

    fn get_segment_registers(
        &self,
        registers: &[SegmentRegister],
    ) -> Result<Vec<Segment>, Error> {
        let registers: Vec<WHV_REGISTER_NAME> = registers
            .into_iter()
            .map(|register| match register {
                SegmentRegister::Cs  => WHvX64RegisterCs,
                SegmentRegister::Ds  => WHvX64RegisterDs,
                SegmentRegister::Es  => WHvX64RegisterEs,
                SegmentRegister::Fs  => WHvX64RegisterFs,
                SegmentRegister::Gs  => WHvX64RegisterGs,
                SegmentRegister::Ss  => WHvX64RegisterSs,
                SegmentRegister::Tr  => WHvX64RegisterTr,
                SegmentRegister::Ldt => WHvX64RegisterLdtr,
            })
            .collect();

        let mut values = vec![WHV_REGISTER_VALUE::default(); registers.len()];

        unsafe {
            WHvGetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                registers.as_ptr(),
                registers.len() as u32,
                values.as_mut_ptr(),
            )
        }?;

        Ok(values
            .into_iter()
            .map(|value| {
                let segment = unsafe { value.Segment };
                let attributes = unsafe { segment.Anonymous.Attributes };

                Segment {
                    base: segment.Base,
                    limit: segment.Limit,
                    selector: segment.Selector,
                    segment_type: (attributes & 0xf) as u8,
                    non_system_segment: (attributes >> 4) & 0x1 == 0x1,
                    dpl: ((attributes >> 5) & 0x3) as u8,
                    present: (attributes >> 7) & 0x1 == 0x1,
                    available: (attributes >> 12) & 0x1 == 0x1,
                    long: (attributes >> 13) & 0x1 == 0x1,
                    default: (attributes >> 14) & 0x1 == 0x1,
                    granularity: (attributes >> 15) & 0x1 == 0x1,
                }
            })
            .collect())
    }

    fn set_segment_registers(
        &mut self,
        registers: &[SegmentRegister],
        values: &[Segment],
    ) -> Result<(), Error> {
        let registers: Vec<WHV_REGISTER_NAME> = registers
            .into_iter()
            .map(|register| match register {
                SegmentRegister::Cs  => WHvX64RegisterCs,
                SegmentRegister::Ds  => WHvX64RegisterDs,
                SegmentRegister::Es  => WHvX64RegisterEs,
                SegmentRegister::Fs  => WHvX64RegisterFs,
                SegmentRegister::Gs  => WHvX64RegisterGs,
                SegmentRegister::Ss  => WHvX64RegisterSs,
                SegmentRegister::Tr  => WHvX64RegisterTr,
                SegmentRegister::Ldt => WHvX64RegisterLdtr,
            })
            .collect();

        let values: Vec<WHV_REGISTER_VALUE> = values
            .into_iter()
            .map(|value| {
                let mut new_value = WHV_REGISTER_VALUE::default();
                let segment = unsafe { &mut new_value.Segment };

                segment.Base = value.base;
                segment.Limit = value.limit;
                segment.Selector = value.selector;

                let attributes =
                    (value.segment_type as u16) & 0xf |
                    (value.non_system_segment as u16) << 4 |
                    ((value.dpl as u16) & 0x3) << 5 |
                    (value.present as u16) << 7 |
                    (value.available as u16) << 12 |
                    (value.long as u16) << 13 |
                    (value.default as u16) << 14 |
                    (value.granularity as u16) << 15;

                segment.Anonymous.Attributes = attributes;

                new_value
            })
            .collect();

        unsafe {
            WHvSetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                registers.as_ptr(),
                registers.len() as u32,
                values.as_ptr(),
            )
        }?;

        Ok(())
    }

    fn get_descriptor_tables(
        &self,
        registers: &[DescriptorTableRegister],
    ) -> Result<Vec<DescriptorTable>, Error> {
        let registers: Vec<WHV_REGISTER_NAME> = registers
            .into_iter()
            .map(|register| match register {
                DescriptorTableRegister::Gdt => WHvX64RegisterGdtr,
                DescriptorTableRegister::Idt => WHvX64RegisterIdtr,
            })
            .collect();

        let mut values = vec![WHV_REGISTER_VALUE::default(); registers.len()];

        unsafe {
            WHvGetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                registers.as_ptr(),
                registers.len() as u32,
                values.as_mut_ptr(),
            )
        }?;

        Ok(values
            .into_iter()
            .map(|value| {
                let table = unsafe { value.Table };

                DescriptorTable {
                    base: table.Base,
                    limit: table.Limit,
                }
            })
            .collect())
    }

    fn set_descriptor_tables(
        &mut self,
        registers: &[DescriptorTableRegister],
        values: &[DescriptorTable],
    ) -> Result<(), Error> {
        let registers: Vec<WHV_REGISTER_NAME> = registers
            .into_iter()
            .map(|register| match register {
                DescriptorTableRegister::Gdt => WHvX64RegisterGdtr,
                DescriptorTableRegister::Idt => WHvX64RegisterIdtr,
            })
            .collect();

        let values: Vec<WHV_REGISTER_VALUE> = values
            .into_iter()
            .map(|value| {
                let mut new_value = WHV_REGISTER_VALUE::default();
                let table = unsafe { &mut new_value.Table };

                table.Base = value.base;
                table.Limit = value.limit;

                new_value
            })
            .collect();

        unsafe {
            WHvSetVirtualProcessorRegisters(
                self.handle.deref().0,
                self.id,
                registers.as_ptr(),
                registers.len() as u32,
                values.as_ptr(),
            )
        }?;

        Ok(())
    }
}
