use crate::error::Error;
use crate::vcpu::ExitReason;
use num_traits::FromPrimitive;
use super::bindings::*;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::*;

pub struct Vcpu {
    pub(crate) vcpu: hv_vcpuid_t,
}

#[cfg(target_arch = "x86_64")]
impl Vcpu {
    /// Helper function to read a register.
    pub(crate) fn read_register(&self, register: hv_x86_reg_t) -> Result<u64, Error> {
        let mut value = 0;

        unsafe {
            hv_vcpu_read_register(self.vcpu, register, &mut value)
        }.into_result()?;

        Ok(value)
    }

    /// Helper function to write a register.
    pub(crate) fn write_register(&mut self, register: hv_x86_reg_t, value: u64) -> Result<(), Error> {
        unsafe {
            hv_vcpu_write_register(self.vcpu, register, value)
        }.into_result()?;

        Ok(())
    }

    /// Helper function to read from a field in the VMCS.
    pub(crate) fn read_vmcs(&self, field: Vmcs) -> Result<u64, Error> {
        let mut value = 0;

        unsafe {
            hv_vmx_vcpu_read_vmcs(self.vcpu, field, &mut value)
        }.into_result()?;

        Ok(value)
    }

    /// Helper function to write to a field in the VMCS.
    pub(crate) fn write_vmcs(&mut self, field: Vmcs, value: u64) -> Result<(), Error> {
        unsafe {
            hv_vmx_vcpu_write_vmcs(self.vcpu, field, value)
        }.into_result()?;

        Ok(())
    }

    /// Helper function read from a MSR.
    pub(crate) fn read_msr(&self, register: u32) -> Result<u64, Error> {
        let mut value = 0;

        unsafe {
            hv_vcpu_read_msr(self.vcpu, register, &mut value)
        }.into_result()?;

        Ok(value)
    }

    /// Helper function to write to a MSR.
    pub(crate) fn write_msr(&mut self, register: u32, value: u64) -> Result<(), Error> {
        unsafe {
            hv_vcpu_write_msr(self.vcpu, register, value)
        }.into_result()?;

        Ok(())
    }

    /// Helper to enable access to a MSR.
    pub(crate) fn enable_native_msr(&mut self, msr: u32, enabled: bool) -> Result<(), Error> {
        unsafe {
            hv_vcpu_enable_native_msr(self.vcpu, msr, enabled)
        }.into_result()?;

        Ok(())
    }

    /// Resets the CPU to default state.
    pub fn reset(&mut self) -> Result<(), Error> {
        let mut value = self.read_vmcs(Vmcs::CpuBased)?;
        let mut cpu_based = CpuBased::empty();
        cpu_based |= CpuBased::HLT;
        cpu_based |= CpuBased::SECONDARY_CONTROLS;
        value |= cpu_based.bits() as u64;
        self.write_vmcs(Vmcs::CpuBased, value)?;

        let mut value = self.read_vmcs(Vmcs::CpuBased2)?;
        let mut cpu_based2 = CpuBased2::empty();
        cpu_based2 |= CpuBased2::UNRESTRICTED_GUEST;
        value |= cpu_based2.bits() as u64;
        self.write_vmcs(Vmcs::CpuBased2, value)?;

        // Reset the segments.
        self.write_vmcs(Vmcs::GuestCs, 0xf0000)?;
        self.write_vmcs(Vmcs::GuestCsBase, 0xffff_0000)?;
        self.write_vmcs(Vmcs::GuestCsLimit, 0xf_ffff)?;
        self.write_vmcs(Vmcs::GuestCsAccessRights, 0x9b)?;

        self.write_vmcs(Vmcs::GuestDs, 0)?;
        self.write_vmcs(Vmcs::GuestDsBase, 0)?;
        self.write_vmcs(Vmcs::GuestDsLimit, 0xf_ffff)?;
        self.write_vmcs(Vmcs::GuestDsAccessRights, 0x93)?;

        self.write_vmcs(Vmcs::GuestEs, 0)?;
        self.write_vmcs(Vmcs::GuestEsBase, 0)?;
        self.write_vmcs(Vmcs::GuestEsLimit, 0xf_ffff)?;
        self.write_vmcs(Vmcs::GuestEsAccessRights, 0x93)?;

        self.write_vmcs(Vmcs::GuestFs, 0)?;
        self.write_vmcs(Vmcs::GuestFsBase, 0)?;
        self.write_vmcs(Vmcs::GuestFsLimit, 0xf_ffff)?;
        self.write_vmcs(Vmcs::GuestFsAccessRights, 0x93)?;

        self.write_vmcs(Vmcs::GuestGs, 0)?;
        self.write_vmcs(Vmcs::GuestGsBase, 0)?;
        self.write_vmcs(Vmcs::GuestGsLimit, 0xf_ffff)?;
        self.write_vmcs(Vmcs::GuestGsAccessRights, 0x93)?;

        self.write_vmcs(Vmcs::GuestSs, 0)?;
        self.write_vmcs(Vmcs::GuestSsBase, 0)?;
        self.write_vmcs(Vmcs::GuestSsLimit, 0xf_ffff)?;
        self.write_vmcs(Vmcs::GuestSsAccessRights, 0x93)?;

        self.write_vmcs(Vmcs::GuestLdtrAccessRights, 0x1_0000)?;
        self.write_vmcs(Vmcs::GuestTrAccessRights, 0x8b)?;

        // These MSRs must be enabled. Otherwise enabling the long mode bits in EFER would cause
        // the VM entry to fail without any indicative exit reason.
        self.enable_native_msr(MSR_IA32_LSTAR, true)?;
        self.enable_native_msr(MSR_IA32_CSTAR, true)?;
        self.enable_native_msr(MSR_IA32_STAR, true)?;
        self.enable_native_msr(MSR_IA32_SYSCALL_MASK, true)?;
        self.enable_native_msr(MSR_IA32_KERNEL_GS_BASE, true)?;

        self.write_register(hv_x86_reg_t::HV_X86_RIP, 0xfff0)?;
        self.write_register(hv_x86_reg_t::HV_X86_RFLAGS, 2)?;

        self.write_register(hv_x86_reg_t::HV_X86_CR0, 0)?;
        self.write_register(hv_x86_reg_t::HV_X86_CR4, CR4_VMXE)?;
        self.write_vmcs(Vmcs::GuestEfer, 0)?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<ExitReason, Error> {
        let exit_reason = loop {
            unsafe {
                hv_vcpu_run(self.vcpu)
            }.into_result()?;

            let value = self.read_vmcs(Vmcs::ExitReason)?;

            let exit_reason = match VmxReason::from_u32((value as u32) & 0x7fff_ffff) {
                Some(exit_reason) => exit_reason,
                _ => return Ok(ExitReason::Unknown),
            };

            break match exit_reason {
                VmxReason::Irq =>
                    continue,
                VmxReason::TripleFault =>
                    ExitReason::UnhandledException,
                VmxReason::Hlt => {
                    // Skip the `hlt` instruction.
                    let rip = self.read_register(hv_x86_reg_t::HV_X86_RIP)?;
                    self.write_register(hv_x86_reg_t::HV_X86_RIP, rip + 1)?;

                    ExitReason::Halted
                }
                VmxReason::EptViolation => {
                    let phys_addr = self.read_vmcs(Vmcs::GuestPhysicalAddress)?;
                    let virt_addr = self.read_vmcs(Vmcs::GuestLinearAddress)?;

                    // Ignore EPT violations for regions that are mapped in for the VM, as we are
                    // just seeing the page table walks from the MMU for valid pages.
                    /*if self.regions.read().unwrap().contains(&phys_addr) {
                        continue;
                    }*/

                    // The virtual CPU just tried accessing some area we did not map.
                    ExitReason::InvalidMemoryAccess {
                        gpa: phys_addr,
                        gva: virt_addr as usize,
                    }
                }
                _ => ExitReason::Unknown
            }
        };

        Ok(exit_reason)
    }
}

impl Drop for Vcpu {
    fn drop(&mut self) {
        unsafe {
            hv_vcpu_destroy(self.vcpu)
        };
    }
}

#[cfg(target_arch = "x86_64")]
impl CpuRegs for Vcpu {
    fn get_registers(
        &self,
        registers: &[Register],
    ) -> Result<Vec<u64>, Error> {
        let mut values = vec![];

        for register in registers {
            let register = match register {
                Register::Rax    => hv_x86_reg_t::HV_X86_RAX,
                Register::Rcx    => hv_x86_reg_t::HV_X86_RCX,
                Register::Rdx    => hv_x86_reg_t::HV_X86_RDX,
                Register::Rbx    => hv_x86_reg_t::HV_X86_RBX,
                Register::Rsp    => hv_x86_reg_t::HV_X86_RSP,
                Register::Rbp    => hv_x86_reg_t::HV_X86_RBP,
                Register::Rsi    => hv_x86_reg_t::HV_X86_RSI,
                Register::Rdi    => hv_x86_reg_t::HV_X86_RDI,
                Register::R8     => hv_x86_reg_t::HV_X86_R8,
                Register::R9     => hv_x86_reg_t::HV_X86_R9,
                Register::R10    => hv_x86_reg_t::HV_X86_R10,
                Register::R11    => hv_x86_reg_t::HV_X86_R11,
                Register::R12    => hv_x86_reg_t::HV_X86_R12,
                Register::R13    => hv_x86_reg_t::HV_X86_R13,
                Register::R14    => hv_x86_reg_t::HV_X86_R14,
                Register::R15    => hv_x86_reg_t::HV_X86_R15,
                Register::Rip    => hv_x86_reg_t::HV_X86_RIP,
                Register::Rflags => hv_x86_reg_t::HV_X86_RFLAGS,
            };

            values.push(self.read_register(register)?);
        }

        Ok(values)
    }

    fn set_registers(
        &mut self,
        registers: &[Register],
        values: &[u64],
    ) -> Result<(), Error> {
        for (register, value) in registers.iter().zip(values.iter()) {
            let register = match register {
                Register::Rax    => hv_x86_reg_t::HV_X86_RAX,
                Register::Rcx    => hv_x86_reg_t::HV_X86_RCX,
                Register::Rdx    => hv_x86_reg_t::HV_X86_RDX,
                Register::Rbx    => hv_x86_reg_t::HV_X86_RBX,
                Register::Rsp    => hv_x86_reg_t::HV_X86_RSP,
                Register::Rbp    => hv_x86_reg_t::HV_X86_RBP,
                Register::Rsi    => hv_x86_reg_t::HV_X86_RSI,
                Register::Rdi    => hv_x86_reg_t::HV_X86_RDI,
                Register::R8     => hv_x86_reg_t::HV_X86_R8,
                Register::R9     => hv_x86_reg_t::HV_X86_R9,
                Register::R10    => hv_x86_reg_t::HV_X86_R10,
                Register::R11    => hv_x86_reg_t::HV_X86_R11,
                Register::R12    => hv_x86_reg_t::HV_X86_R12,
                Register::R13    => hv_x86_reg_t::HV_X86_R13,
                Register::R14    => hv_x86_reg_t::HV_X86_R14,
                Register::R15    => hv_x86_reg_t::HV_X86_R15,
                Register::Rip    => hv_x86_reg_t::HV_X86_RIP,
                Register::Rflags => hv_x86_reg_t::HV_X86_RFLAGS,
            };

            self.write_register(register, *value)?;
        }

        Ok(())
    }

    fn get_control_registers(
        &self,
        registers: &[ControlRegister],
    ) -> Result<Vec<u64>, Error> {
        let mut values = vec![];

        for register in registers {
            let register = match register {
                ControlRegister::Cr0 => hv_x86_reg_t::HV_X86_CR0,
                ControlRegister::Cr1 => hv_x86_reg_t::HV_X86_CR1,
                ControlRegister::Cr2 => hv_x86_reg_t::HV_X86_CR2,
                ControlRegister::Cr3 => hv_x86_reg_t::HV_X86_CR3,
                ControlRegister::Cr4 => hv_x86_reg_t::HV_X86_CR4,
                ControlRegister::Cr8 => {
                    values.push(0);
                    continue;
                }
            };

            let mut value = self.read_register(register)?;

            match register {
                hv_x86_reg_t::HV_X86_CR4 =>
                    value &= !CR4_VMXE,
                _ => (),
            }

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
            let mut value = *value;

            let register = match register {
                ControlRegister::Cr0 => hv_x86_reg_t::HV_X86_CR0,
                ControlRegister::Cr1 => hv_x86_reg_t::HV_X86_CR1,
                ControlRegister::Cr2 => hv_x86_reg_t::HV_X86_CR2,
                ControlRegister::Cr3 => hv_x86_reg_t::HV_X86_CR3,
                ControlRegister::Cr4 => {
                    value |= CR4_VMXE;
                    hv_x86_reg_t::HV_X86_CR4
                }
                ControlRegister::Cr8 => continue,
            };

            unsafe {
                hv_vcpu_write_register(self.vcpu, register, value)
            }.into_result()?;
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
                    self.read_vmcs(Vmcs::GuestEfer)?,
                register =>
                    self.read_msr(register)?,
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
            let mut value = *value;

            match *register {
                MSR_IA32_EFER => {
                    let mut flags = self.read_vmcs(Vmcs::VmEntryControls)?;

                    // If the long mode enable bit is set, we should make sure to set the active
                    // bit and to load the guest as a 64-bit guest.
                    if value & EFER_LME == EFER_LME {
                        value |= EFER_LMA;
                        flags |= VmEntryControls::GUEST_IA32E.bits() as u64;
                    }

                    self.write_vmcs(Vmcs::VmEntryControls, flags)?;
                    self.write_vmcs(Vmcs::GuestEfer, value)?;
                }
                register =>
                    self.write_msr(register, value)?,
            };
        }

        Ok(())
    }

    fn get_segment_registers(
        &self,
        registers: &[SegmentRegister],
    ) -> Result<Vec<Segment>, Error> {
        let mut segments = vec![];

        for register in registers {
            let fields = match register {
                SegmentRegister::Cs => (
                    Vmcs::GuestCs,
                    Vmcs::GuestCsBase,
                    Vmcs::GuestCsLimit,
                    Vmcs::GuestCsAccessRights,
                ),
                SegmentRegister::Ds => (
                    Vmcs::GuestDs,
                    Vmcs::GuestDsBase,
                    Vmcs::GuestDsLimit,
                    Vmcs::GuestDsAccessRights,
                ),
                SegmentRegister::Es => (
                    Vmcs::GuestEs,
                    Vmcs::GuestEsBase,
                    Vmcs::GuestEsLimit,
                    Vmcs::GuestEsAccessRights,
                ),
                SegmentRegister::Fs => (
                    Vmcs::GuestFs,
                    Vmcs::GuestFsBase,
                    Vmcs::GuestFsLimit,
                    Vmcs::GuestFsAccessRights,
                ),
                SegmentRegister::Gs => (
                    Vmcs::GuestGs,
                    Vmcs::GuestGsBase,
                    Vmcs::GuestGsLimit,
                    Vmcs::GuestGsAccessRights,
                ),
                SegmentRegister::Ss => (
                    Vmcs::GuestSs,
                    Vmcs::GuestSsBase,
                    Vmcs::GuestSsLimit,
                    Vmcs::GuestSsAccessRights,
                ),
                SegmentRegister::Tr => (
                    Vmcs::GuestTr,
                    Vmcs::GuestTrBase,
                    Vmcs::GuestTrLimit,
                    Vmcs::GuestTrAccessRights,
                ),
                SegmentRegister::Ldt => (
                    Vmcs::GuestLdtr,
                    Vmcs::GuestLdtrBase,
                    Vmcs::GuestLdtrLimit,
                    Vmcs::GuestLdtrAccessRights,
                ),
            };

            let selector = self.read_vmcs(fields.0)?;
            let base = self.read_vmcs(fields.1)?;
            let limit = self.read_vmcs(fields.2)?;
            let access_rights = self.read_vmcs(fields.3)?;

            segments.push(Segment {
                base,
                limit: limit as u32,
                selector: selector as u16,
                segment_type: (access_rights & 0xf) as u8,
                non_system_segment: (access_rights >> 4) & 0x1 == 0x1,
                dpl: ((access_rights >> 5) & 0x3) as u8,
                present: (access_rights >> 7) & 0x1 == 0x1,
                available: (access_rights >> 12) & 0x1 == 0x1,
                long: (access_rights >> 13) & 0x1 == 0x1,
                default: (access_rights >> 14) & 0x1 == 0x1,
                granularity: (access_rights >> 15) & 0x1 == 0x1,
            })
        }

        Ok(segments)
    }

    fn set_segment_registers(
        &mut self,
        registers: &[SegmentRegister],
        values: &[Segment],
    ) -> Result<(), Error> {
        for (register, segment) in registers.iter().zip(values.iter()) {
            let fields = match register {
                SegmentRegister::Cs => (
                    Vmcs::GuestCs,
                    Vmcs::GuestCsBase,
                    Vmcs::GuestCsLimit,
                    Vmcs::GuestCsAccessRights,
                ),
                SegmentRegister::Ds => (
                    Vmcs::GuestDs,
                    Vmcs::GuestDsBase,
                    Vmcs::GuestDsLimit,
                    Vmcs::GuestDsAccessRights,
                ),
                SegmentRegister::Es => (
                    Vmcs::GuestEs,
                    Vmcs::GuestEsBase,
                    Vmcs::GuestEsLimit,
                    Vmcs::GuestEsAccessRights,
                ),
                SegmentRegister::Fs => (
                    Vmcs::GuestFs,
                    Vmcs::GuestFsBase,
                    Vmcs::GuestFsLimit,
                    Vmcs::GuestFsAccessRights,
                ),
                SegmentRegister::Gs => (
                    Vmcs::GuestGs,
                    Vmcs::GuestGsBase,
                    Vmcs::GuestGsLimit,
                    Vmcs::GuestGsAccessRights,
                ),
                SegmentRegister::Ss => (
                    Vmcs::GuestSs,
                    Vmcs::GuestSsBase,
                    Vmcs::GuestSsLimit,
                    Vmcs::GuestSsAccessRights,
                ),
                SegmentRegister::Tr => (
                    Vmcs::GuestTr,
                    Vmcs::GuestTrBase,
                    Vmcs::GuestTrLimit,
                    Vmcs::GuestTrAccessRights,
                ),
                SegmentRegister::Ldt => (
                    Vmcs::GuestLdtr,
                    Vmcs::GuestLdtrBase,
                    Vmcs::GuestLdtrLimit,
                    Vmcs::GuestLdtrAccessRights,
                ),
            };

            self.write_vmcs(fields.0, segment.selector as u64)?;
            self.write_vmcs(fields.1, segment.base)?;
            self.write_vmcs(fields.2, segment.limit as u64)?;

            let value =
                (segment.segment_type as u64) & 0xf |
                (segment.non_system_segment as u64) << 4 |
                ((segment.dpl as u64) & 0x3) << 5 |
                (segment.present as u64) << 7 |
                (segment.available as u64) << 12 |
                (segment.long as u64) << 13 |
                (segment.default as u64) << 14 |
                (segment.granularity as u64) << 15;

            self.write_vmcs(fields.3, value)?;
        }

        Ok(())
    }

    fn get_descriptor_tables(
        &self,
        registers: &[DescriptorTableRegister],
    ) -> Result<Vec<DescriptorTable>, Error> {
        let mut values = vec![];

        for register in registers {
            let fields = match register {
                DescriptorTableRegister::Gdt =>
                    (Vmcs::GuestGdtrBase, Vmcs::GuestGdtrLimit),
                DescriptorTableRegister::Idt =>
                    (Vmcs::GuestIdtrBase, Vmcs::GuestIdtrLimit),
            };

            values.push(DescriptorTable {
                base: self.read_vmcs(fields.0)?,
                limit: self.read_vmcs(fields.1)? as u16,
            });
        }

        Ok(values)
    }

    fn set_descriptor_tables(
        &mut self,
        registers: &[DescriptorTableRegister],
        values: &[DescriptorTable],
    ) -> Result<(), Error> {
        for (register, value) in registers.iter().zip(values.iter()) {
            let fields = match register {
                DescriptorTableRegister::Gdt =>
                    (Vmcs::GuestGdtrBase, Vmcs::GuestGdtrLimit),
                DescriptorTableRegister::Idt =>
                    (Vmcs::GuestIdtrBase, Vmcs::GuestIdtrLimit),
            };

            self.write_vmcs(fields.0, value.base)?;
            self.write_vmcs(fields.1, value.limit as u64)?;
        }

        Ok(())
    }
}

#[cfg(target_arch = "aarch64")]
impl Vcpu {
    /// Resets the CPU to default state.
    pub fn reset(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn run(&mut self) -> Result<ExitReason, Error> {
        Ok(ExitReason::Unknown)
    }
}
