use crate::error::Error;
use crate::vcpu::ExitReason;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use super::bindings::*;

pub struct Vcpu {
    pub(crate) cpuid: i32,
    pub(crate) file: File,
    pub(crate) rip: u64,
}

impl Vcpu {
    fn vm_get_register(
        &self,
        regnum: vm_reg_name,
    ) -> Result<u64, Error> {
        let mut args = vm_register {
            cpuid: self.cpuid,
            regnum,
            value: 0,
        };

        unsafe {
            vm_get_register(self.file.as_raw_fd(), &mut args)
        }?;
        
        Ok(args.value)
    }

    fn vm_set_register(
        &self,
        regnum: vm_reg_name,
        value: u64,
    ) -> Result<(), Error> {
         let args = vm_register {
            cpuid: self.cpuid,
            regnum,
            value,
        };

        unsafe {
            vm_set_register(self.file.as_raw_fd(), &args)
        }?;
        
        Ok(())
    }

    fn vm_get_segment_descriptor(
        &self,
        regnum: vm_reg_name,
    ) -> Result<seg_desc, Error> {
        let mut args = vm_seg_desc {
            cpuid: self.cpuid,
            regnum,
            desc: unsafe { std::mem::zeroed() },
        };

        unsafe {
            vm_get_segment_descriptor(self.file.as_raw_fd(), &mut args)
        }?;

        Ok(args.desc)
    }

    fn vm_set_segment_descriptor(
        &self,
        regnum: vm_reg_name,
        desc: seg_desc,
    ) -> Result<(), Error> {
        let args = vm_seg_desc {
            cpuid: self.cpuid,
            regnum,
            desc,
        };

        unsafe {
            vm_set_segment_descriptor(self.file.as_raw_fd(), &args)
        }?;

        Ok(())
    }


    pub fn run(&self) -> Result<ExitReason, Error> {
        let mut args: vm_run = unsafe { std::mem::zeroed() };

        args.cpuid = self.cpuid;
        args.rip   = self.rip;

        unsafe {
            vm_run(self.file.as_raw_fd(), &mut args)
        }?;

        let exit_reason = match args.vm_exit.exitcode {
            vm_exitcode::VM_EXITCODE_HLT => ExitReason::Halted,
            _ => ExitReason::Unknown,
        };

        Ok(exit_reason)
    }
}

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
impl CpuRegs for Vcpu {
    fn get_registers(
        &self,
        registers: &[Register],
    ) -> Result<Vec<u64>, Error> {
        let mut values = vec![];

        for register in registers {
            let regnum = match *register {
                Register::Rax =>    vm_reg_name::VM_REG_GUEST_RAX,
                Register::Rcx =>    vm_reg_name::VM_REG_GUEST_RCX,
                Register::Rdx =>    vm_reg_name::VM_REG_GUEST_RDX,
                Register::Rbx =>    vm_reg_name::VM_REG_GUEST_RBX,
                Register::Rsp =>    vm_reg_name::VM_REG_GUEST_RSP,
                Register::Rbp =>    vm_reg_name::VM_REG_GUEST_RBP,
                Register::Rsi =>    vm_reg_name::VM_REG_GUEST_RSI,
                Register::Rdi =>    vm_reg_name::VM_REG_GUEST_RDI,
                Register::R8  =>    vm_reg_name::VM_REG_GUEST_R8,
                Register::R9  =>    vm_reg_name::VM_REG_GUEST_R9,
                Register::R10 =>    vm_reg_name::VM_REG_GUEST_R10,
                Register::R11 =>    vm_reg_name::VM_REG_GUEST_R11,
                Register::R12 =>    vm_reg_name::VM_REG_GUEST_R12,
                Register::R13 =>    vm_reg_name::VM_REG_GUEST_R13,
                Register::R14 =>    vm_reg_name::VM_REG_GUEST_R14,
                Register::R15 =>    vm_reg_name::VM_REG_GUEST_R15,
                Register::Rip =>    vm_reg_name::VM_REG_GUEST_RIP,
                Register::Rflags => vm_reg_name::VM_REG_GUEST_RFLAGS,
            };

            let value = self.vm_get_register(regnum)?;

            values.push(value);
        }

        Ok(values)
    }

    fn set_registers(
        &mut self,
        registers: &[Register],
        values: &[u64],
    ) -> Result<(), Error> {
        for (register, value) in registers.iter().zip(values.iter()) {
            let regnum = match *register {
                Register::Rax =>    vm_reg_name::VM_REG_GUEST_RAX,
                Register::Rcx =>    vm_reg_name::VM_REG_GUEST_RCX,
                Register::Rdx =>    vm_reg_name::VM_REG_GUEST_RDX,
                Register::Rbx =>    vm_reg_name::VM_REG_GUEST_RBX,
                Register::Rsp =>    vm_reg_name::VM_REG_GUEST_RSP,
                Register::Rbp =>    vm_reg_name::VM_REG_GUEST_RBP,
                Register::Rsi =>    vm_reg_name::VM_REG_GUEST_RSI,
                Register::Rdi =>    vm_reg_name::VM_REG_GUEST_RDI,
                Register::R8  =>    vm_reg_name::VM_REG_GUEST_R8,
                Register::R9  =>    vm_reg_name::VM_REG_GUEST_R9,
                Register::R10 =>    vm_reg_name::VM_REG_GUEST_R10,
                Register::R11 =>    vm_reg_name::VM_REG_GUEST_R11,
                Register::R12 =>    vm_reg_name::VM_REG_GUEST_R12,
                Register::R13 =>    vm_reg_name::VM_REG_GUEST_R13,
                Register::R14 =>    vm_reg_name::VM_REG_GUEST_R14,
                Register::R15 =>    vm_reg_name::VM_REG_GUEST_R15,
                Register::Rip =>    vm_reg_name::VM_REG_GUEST_RIP,
                Register::Rflags => vm_reg_name::VM_REG_GUEST_RFLAGS,
            };

            match *register {
                Register::Rip => self.rip = *value,
                _ => (),
            };

            self.vm_set_register(regnum, *value)?;
        }

        Ok(())
    }

    fn get_control_registers(
        &self,
        registers: &[ControlRegister],
    ) -> Result<Vec<u64>, Error> {
        let mut values = vec![];

        for register in registers {
            let regnum = match *register {
                ControlRegister::Cr0 => Some(vm_reg_name::VM_REG_GUEST_CR0),
                ControlRegister::Cr1 => None,
                ControlRegister::Cr2 => None,
                ControlRegister::Cr3 => Some(vm_reg_name::VM_REG_GUEST_CR3),
                ControlRegister::Cr4 => Some(vm_reg_name::VM_REG_GUEST_CR4),
                ControlRegister::Cr8 => None,
            };

            let value = if let Some(regnum) = regnum {
                self.vm_get_register(regnum)?
            } else {
                0
            };

            values.push(value);
        }

        Ok(values)
    }

    fn set_control_registers(
        &mut self,
        registers: &[ControlRegister],
        values: &[u64],
    ) -> Result<(), Error> {
        for (register, value) in registers.iter().zip(values.iter()) {
            let regnum = match *register {
                ControlRegister::Cr0 => vm_reg_name::VM_REG_GUEST_CR0,
                ControlRegister::Cr1 => continue,
                ControlRegister::Cr2 => continue,
                ControlRegister::Cr3 => vm_reg_name::VM_REG_GUEST_CR3,
                ControlRegister::Cr4 => vm_reg_name::VM_REG_GUEST_CR4,
                ControlRegister::Cr8 => continue,
            };
            
            self.vm_set_register(regnum, *value)?;
        }

        Ok(())
    }

    fn get_msrs(
        &self,
        registers: &[u32],
    ) -> Result<Vec<u64>, Error> {
        let mut values = vec![];

        for register in registers {
            let value = match *register {
                MSR_IA32_EFER =>
                    self.vm_get_register(vm_reg_name::VM_REG_GUEST_EFER)?,
                _ => 0,
            };

            values.push(value);
        }

        Ok(values)
    }

    fn set_msrs(
        &mut self,
        registers: &[u32],
        values: &[u64],
    ) -> Result<(), Error> {
        for (register, value) in registers.iter().zip(values.iter()) {
            match *register {
                MSR_IA32_EFER =>
                    self.vm_set_register(vm_reg_name::VM_REG_GUEST_EFER, *value)?,
                _ => (),
            }
        }

        Ok(())
    }

    fn get_segment_registers(
        &self,
        registers: &[SegmentRegister],
    ) -> Result<Vec<Segment>, Error> {
        let mut segments = vec![];

        for register in registers {
            let regnum = match *register {
                SegmentRegister::Cs  => vm_reg_name::VM_REG_GUEST_CS,
                SegmentRegister::Ds  => vm_reg_name::VM_REG_GUEST_DS,
                SegmentRegister::Es  => vm_reg_name::VM_REG_GUEST_ES,
                SegmentRegister::Fs  => vm_reg_name::VM_REG_GUEST_FS,
                SegmentRegister::Gs  => vm_reg_name::VM_REG_GUEST_GS,
                SegmentRegister::Ss  => vm_reg_name::VM_REG_GUEST_SS,
                SegmentRegister::Tr  => vm_reg_name::VM_REG_GUEST_TR,
                SegmentRegister::Ldt => vm_reg_name::VM_REG_GUEST_LDTR,
            };

            let selector = self.vm_get_register(regnum)?;
            let descriptor = self.vm_get_segment_descriptor(regnum)?;

            segments.push(Segment {
                base: descriptor.base,
                limit: descriptor.limit,
                selector: selector as u16,
                segment_type: (descriptor.access & 0xf) as u8,
                non_system_segment: (descriptor.access >> 4) & 0x1 == 0x1,
                dpl: ((descriptor.access >> 5) & 0x3) as u8,
                present: (descriptor.access >> 7) & 0x1 == 0x1,
                available: (descriptor.access >> 12) & 0x1 == 0x1,
                long: (descriptor.access >> 13) & 0x1 == 0x1,
                default: (descriptor.access >> 14) & 0x1 == 0x1,
                granularity: (descriptor.access >> 15) & 0x1 == 0x1,
            });
        }

        Ok(vec![])
    }

    fn set_segment_registers(
        &mut self,
        registers: &[SegmentRegister],
        values: &[Segment],
    ) -> Result<(), Error> {
        for (register, segment) in registers.iter().zip(values.iter()) {
            let regnum = match *register {
                SegmentRegister::Cs  => vm_reg_name::VM_REG_GUEST_CS,
                SegmentRegister::Ds  => vm_reg_name::VM_REG_GUEST_DS,
                SegmentRegister::Es  => vm_reg_name::VM_REG_GUEST_ES,
                SegmentRegister::Fs  => vm_reg_name::VM_REG_GUEST_FS,
                SegmentRegister::Gs  => vm_reg_name::VM_REG_GUEST_GS,
                SegmentRegister::Ss  => vm_reg_name::VM_REG_GUEST_SS,
                SegmentRegister::Tr  => vm_reg_name::VM_REG_GUEST_TR,
                SegmentRegister::Ldt => vm_reg_name::VM_REG_GUEST_LDTR,
            };

            let access =
                (segment.segment_type as u32) & 0xf |
                (segment.non_system_segment as u32) << 4 |
                ((segment.dpl as u32) & 0x3) << 5 |
                (segment.present as u32) << 7 |
                (segment.available as u32) << 12 |
                (segment.long as u32) << 13 |
                (segment.default as u32) << 14 |
                (segment.granularity as u32) << 15;

            self.vm_set_segment_descriptor(regnum, seg_desc {
                base: segment.base,
                limit: segment.limit,
                access,
            })?;
            self.vm_set_register(regnum, segment.selector as u64)?;
        }

        Ok(())
    }

    fn get_descriptor_tables(
        &self,
        registers: &[DescriptorTableRegister],
    ) -> Result<Vec<DescriptorTable>, Error> {
        let mut tables = vec![];

        for register in registers {
            let regnum = match *register {
                DescriptorTableRegister::Gdt =>
                    vm_reg_name::VM_REG_GUEST_GDTR,
                DescriptorTableRegister::Idt =>
                    vm_reg_name::VM_REG_GUEST_IDTR,
            };

            let descriptor = self.vm_get_segment_descriptor(regnum)?;

            tables.push(DescriptorTable {
                base: descriptor.base,
                limit: descriptor.limit as u16,
            });
        }


        Ok(tables)
    }

    fn set_descriptor_tables(
        &mut self,
        registers: &[DescriptorTableRegister],
        values: &[DescriptorTable],
    ) -> Result<(), Error> {
        for (register, table) in registers.iter().zip(values.iter()) {
            let regnum = match *register {
                DescriptorTableRegister::Gdt =>
                    vm_reg_name::VM_REG_GUEST_GDTR,
                DescriptorTableRegister::Idt =>
                    vm_reg_name::VM_REG_GUEST_IDTR,
            };

            self.vm_set_segment_descriptor(regnum, seg_desc {
                base: table.base,
                limit: table.limit as u32,
                access: 0,
            })?;
        }

        Ok(())
    }
}
