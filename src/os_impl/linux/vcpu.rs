use crate::error::Error;
use crate::vcpu::ExitReason;
use kvm_bindings::{kvm_msr_entry, Msrs};
use kvm_ioctls::{VcpuExit, VcpuFd};

pub struct Vcpu {
    pub(crate) vcpu: VcpuFd,
}

impl Vcpu {
    pub fn run(&self) -> Result<ExitReason, Error> {
        let exit_reason = self.vcpu.run()?;

        let exit_reason = match exit_reason {
            VcpuExit::IoOut(port, data) =>
                ExitReason::IoOut { port, data },
            VcpuExit::IoIn(port, data) =>
                ExitReason::IoIn { port, data },
            VcpuExit::MmioRead(address, data) =>
                ExitReason::MmioRead { address, data },
            VcpuExit::MmioWrite(address, data) =>
                ExitReason::MmioWrite { address, data },
            VcpuExit::Hlt =>
                ExitReason::Halted,
            VcpuExit::Shutdown =>
                ExitReason::UnhandledException,
            _ =>
                ExitReason::Unknown,
        };

        Ok(exit_reason)
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
        let regs = self.vcpu.get_regs()?;

        let values = registers
            .into_iter()
            .map(|register| match register {
                Register::Rax    => regs.rax,
                Register::Rcx    => regs.rcx,
                Register::Rdx    => regs.rdx,
                Register::Rbx    => regs.rbx,
                Register::Rsp    => regs.rsp,
                Register::Rbp    => regs.rbp,
                Register::Rsi    => regs.rsi,
                Register::Rdi    => regs.rdi,
                Register::R8     => regs.r8,
                Register::R9     => regs.r9,
                Register::R10    => regs.r10,
                Register::R11    => regs.r11,
                Register::R12    => regs.r12,
                Register::R13    => regs.r13,
                Register::R14    => regs.r14,
                Register::R15    => regs.r15,
                Register::Rip    => regs.rip,
                Register::Rflags => regs.rflags,
            })
            .collect();

        Ok(values)
    }

    fn set_registers(
        &mut self,
        registers: &[Register],
        values: &[u64],
    ) -> Result<(), Error> {
        let mut regs = self.vcpu.get_regs()?;

        for (register, value) in registers.iter().zip(values.iter()) {
            let register = match register {
                Register::Rax    => &mut regs.rax,
                Register::Rcx    => &mut regs.rcx,
                Register::Rdx    => &mut regs.rdx,
                Register::Rbx    => &mut regs.rbx,
                Register::Rsp    => &mut regs.rsp,
                Register::Rbp    => &mut regs.rbp,
                Register::Rsi    => &mut regs.rsi,
                Register::Rdi    => &mut regs.rdi,
                Register::R8     => &mut regs.r8,
                Register::R9     => &mut regs.r9,
                Register::R10    => &mut regs.r10,
                Register::R11    => &mut regs.r11,
                Register::R12    => &mut regs.r12,
                Register::R13    => &mut regs.r13,
                Register::R14    => &mut regs.r14,
                Register::R15    => &mut regs.r15,
                Register::Rip    => &mut regs.rip,
                Register::Rflags => &mut regs.rflags,
            };

            *register = *value;
        }

        self.vcpu.set_regs(&regs)?;

        Ok(())
    }

    fn get_control_registers(
        &self,
        registers: &[ControlRegister],
    ) -> Result<Vec<u64>, Error> {
        let regs = self.vcpu.get_sregs()?;

        let values = registers
            .into_iter()
            .map(|register| match register {
                ControlRegister::Cr0 => regs.cr0,
                ControlRegister::Cr1 => 0,
                ControlRegister::Cr2 => regs.cr2,
                ControlRegister::Cr3 => regs.cr3,
                ControlRegister::Cr4 => regs.cr4,
                ControlRegister::Cr8 => regs.cr8,
            })
            .collect();

        Ok(values)
    }

    fn set_control_registers(
        &mut self,
        registers: &[ControlRegister],
        values: &[u64],
    ) -> Result<(), Error> {
        let mut regs = self.vcpu.get_sregs()?;

        for (register, value) in registers.iter().zip(values.iter()) {
            let register = match register {
                ControlRegister::Cr0 => &mut regs.cr0,
                ControlRegister::Cr1 => continue,
                ControlRegister::Cr2 => &mut regs.cr2,
                ControlRegister::Cr3 => &mut regs.cr3,
                ControlRegister::Cr4 => &mut regs.cr4,
                ControlRegister::Cr8 => &mut regs.cr8,
            };

            *register = *value;
        }

        self.vcpu.set_sregs(&regs)?;

        Ok(())
    }

    fn get_msrs(
        &self,
        registers: &[u32],
    ) -> Result<Vec<u64>, Error> {
        let mut entries = vec![];
        let mut indices = vec![];

        for (index, register) in registers.iter().enumerate() {
            if *register == crate::arch::x86_64::MSR_IA32_EFER {
                indices.push(index);
            } else {
                entries.push(kvm_msr_entry {
                    index: *register,
                    ..Default::default()
                });
            }
        }

        let mut values: Vec<u64> = if entries.len() > 0 {
            let mut msrs = Msrs::from_entries(&entries).unwrap();

            self.vcpu.get_msrs(&mut msrs)?;

            msrs
                .as_slice()
                .into_iter()
                .map(|msr| msr.data)
                .collect()
        } else {
            vec![]
        };

        if indices.len() > 0 {
            let regs = self.vcpu.get_sregs()?;

            for index in indices {
                values.insert(index, regs.efer);
            }
        }

        Ok(values)
    }

    fn set_msrs(
        &mut self,
        registers: &[u32],
        values: &[u64],
    ) -> Result<(), Error> {
        let mut entries = vec![];
        let mut efer = None;

        for (register, value) in registers.iter().zip(values.iter()) {
            if *register == crate::arch::x86_64::MSR_IA32_EFER {
                efer = Some(*value);
            } else {
                entries.push(kvm_msr_entry {
                    index: *register,
                    data: *value,
                    ..Default::default()
                });
            }
        }

        if entries.len() > 0 {
            let msrs = Msrs::from_entries(&entries).unwrap();

            self.vcpu.set_msrs(&msrs)?;
        }

        if let Some(value) = efer {
            let mut regs = self.vcpu.get_sregs()?;

            regs.efer = value;

            self.vcpu.set_sregs(&regs)?;
        }

        Ok(())
    }

    fn get_segment_registers(
        &self,
        registers: &[SegmentRegister],
    ) -> Result<Vec<Segment>, Error> {
        let regs = self.vcpu.get_sregs()?;

        let values = registers
            .into_iter()
            .map(|register| {
                let segment = match register {
                    SegmentRegister::Cs => regs.cs,
                    SegmentRegister::Ds => regs.ds,
                    SegmentRegister::Es => regs.es,
                    SegmentRegister::Fs => regs.fs,
                    SegmentRegister::Gs => regs.gs,
                    SegmentRegister::Ss => regs.ss,
                    SegmentRegister::Tr => regs.tr,
                    SegmentRegister::Ldt => regs.ldt,
                };

                Segment {
                    base: segment.base,
                    limit: segment.limit,
                    selector: segment.selector,
                    segment_type: segment.type_,
                    non_system_segment: segment.s != 0,
                    dpl: segment.dpl,
                    present: segment.present != 0,
                    available: segment.avl != 0,
                    long: segment.l != 0,
                    default: segment.db != 0,
                    granularity: segment.g != 0,
                }
            })
            .collect();

        Ok(values)
    }

    fn set_segment_registers(
        &mut self,
        registers: &[SegmentRegister],
        values: &[Segment],
    ) -> Result<(), Error> {
        let mut regs = self.vcpu.get_sregs()?;

        for (register, value) in registers.iter().zip(values.iter()) {
            let register = match register {
                SegmentRegister::Cs => &mut regs.cs,
                SegmentRegister::Ds => &mut regs.ds,
                SegmentRegister::Es => &mut regs.es,
                SegmentRegister::Fs => &mut regs.fs,
                SegmentRegister::Gs => &mut regs.gs,
                SegmentRegister::Ss => &mut regs.ss,
                SegmentRegister::Tr => &mut regs.tr,
                SegmentRegister::Ldt => &mut regs.ldt,
            };

            register.base     = value.base;
            register.limit    = value.limit;
            register.selector = value.selector;
            register.type_    = value.segment_type;
            register.s        = value.non_system_segment as u8;
            register.dpl      = value.dpl;
            register.present  = value.present as u8;
            register.avl      = value.available as u8;
            register.l        = value.long as u8;
            register.db       = value.default as u8;
            register.g        = value.granularity as u8;
        }

        self.vcpu.set_sregs(&regs)?;

        Ok(())
    }

    fn get_descriptor_tables(
        &self,
        registers: &[DescriptorTableRegister],
    ) -> Result<Vec<DescriptorTable>, Error> {
        let regs = self.vcpu.get_sregs()?;
        let mut values = vec![];

        for register in registers {
            let register = match register {
                DescriptorTableRegister::Gdt => &regs.gdt,
                DescriptorTableRegister::Idt => &regs.idt,
            };

            values.push(DescriptorTable {
                base: register.base,
                limit: register.limit,
            });
        }

        Ok(values)
    }

    fn set_descriptor_tables(
        &mut self,
        registers: &[DescriptorTableRegister],
        values: &[DescriptorTable],
    ) -> Result<(), Error> {
        let mut regs = self.vcpu.get_sregs()?;

        for (register, value) in registers.iter().zip(values.iter()) {
            let register = match register {
                DescriptorTableRegister::Gdt => &mut regs.gdt,
                DescriptorTableRegister::Idt => &mut regs.idt,
            };

            register.base = value.base;
            register.limit = value.limit;
        }

        self.vcpu.set_sregs(&regs)?;

        Ok(())
    }
}
