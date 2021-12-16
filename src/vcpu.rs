//! This modules provides the [`Vcpu`] struct which represents a single virtual CPU that is part of
//! the VM.

use crate::error::Error;
use crate::platform;

/// The exit reason that describes why [`Vcpu::run`] quit.
#[derive(Debug)]
pub enum ExitReason<'a> {
    /// The virtual CPU executed an `out` instruction on the given port with the given data.
    IoOut { port: u16, data: &'a [u8] },
    /// The virtual CPU exected an `in` instruction on the given port. The `data` slice should be
    /// filled with data before calling [`Vcpu::run`] to resume execution of the virtual CPU.
    IoIn { port: u16, data: &'a [u8] },
    /// The virtual CPU tried to read from the given MMIO address. The `data` slice should be
    /// filled with data before calling [`Vcpu::run`] to resume execution of the virtual CPU.
    MmioRead { address: u64, data: &'a [u8] },
    /// The virtual CPU tried to write the given data to the given MMIO address.
    MmioWrite { address: u64, data: &'a [u8] },
    /// The virtual CPU tried accessing an invalid guest physical address.
    InvalidMemoryAccess { gpa: u64, gva: usize },
    /// The virtual CPU executed the `hlt` instruction.
    Halted,
    /// The virtual CPU raised an exception that was not handled by the guest. This is also known
    /// as a triple fault on the x86(-64) architecture, as both the original exception handler and
    /// double fault handler were not able to handle the exception. Some implementations may leave
    /// the virtual CPU in an undefined state or reset the virtual CPU state (e.g. KVM when using
    /// AMD SVM). Therefore, you should not rely on the virtual CPU state in the event of an
    /// unhandled exception.
    UnhandledException,
    /// The virtual CPU exited for some unknown reason.
    Unknown,
}

/// The `Vcpu` struct represents a virtual CPU that is part of the VM.
pub struct Vcpu {
    /// The internal platform-specific implementation of the [`platform::Vcpu`] struct.
    pub(crate) inner: platform::Vcpu,
}

impl Vcpu {
    /// Consumes the current thread to run the virtual CPU until the next exit point. This
    /// function returns an [`ExitReason`] to describe why the virtual CPU exited.
    pub fn run(&mut self) -> Result<ExitReason, Error> {
        self.inner.run()
    }

    #[cfg(target_arch = "x86_64")]
    pub fn reset(&mut self) -> Result<(), Error> {
        // Set up the CPU registers.
        let registers = vec![
            (Register::Rip,    0xfff0),
            (Register::Rflags, 0x0002),
        ];

        let (registers, values): (Vec<Register>, Vec<u64>) = registers.into_iter().unzip();

        self.set_registers(&registers, &values)?;

        // Set up the code, data and stack segments.
        let code_segment = Segment {
            base: 0xffff_0000,
            limit: 0xffff,
            selector: 0xf000,
            segment_type: 0xa,
            non_system_segment: true,
            present: true,
            ..Default::default()
        };

        let data_segment = Segment {
            limit: 0xffff,
            segment_type: 0x3,
            non_system_segment: true,
            present: true,
            ..Default::default()
        };

        let registers = vec![
            (SegmentRegister::Cs, code_segment),
            (SegmentRegister::Ss, data_segment.clone()),
            (SegmentRegister::Ds, data_segment.clone()),
            (SegmentRegister::Es, data_segment.clone()),
            (SegmentRegister::Fs, data_segment.clone()),
            (SegmentRegister::Gs, data_segment),
        ];

        let (registers, segments): (Vec<SegmentRegister>, Vec<Segment>) = registers.into_iter().unzip();

        self.set_segment_registers(&registers, &segments)?;

        Ok(())
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn reset(&mut self) -> Result<(), Error> {
        Ok(())
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
        self.inner.get_registers(registers)
    }

    fn set_registers(
        &mut self,
        registers: &[Register],
        values: &[u64],
    ) -> Result<(), Error> {
        self.inner.set_registers(registers, values)
    }

    fn get_control_registers(
        &self,
        registers: &[ControlRegister],
    ) -> Result<Vec<u64>, Error> {
        self.inner.get_control_registers(registers)
    }

    fn set_control_registers(
        &mut self,
        registers: &[ControlRegister],
        values: &[u64],
    ) -> Result<(), Error> {
        self.inner.set_control_registers(registers, values)
    }

    fn get_msrs(
        &self,
        registers: &[u32],
    ) -> Result<Vec<u64>, Error> {
        self.inner.get_msrs(registers)
    }

    fn set_msrs(
        &mut self,
        registers: &[u32],
        values: &[u64],
    ) -> Result<(), Error> {
        self.inner.set_msrs(registers, values)
    }

    fn get_segment_registers(
        &self,
        registers: &[SegmentRegister],
    ) -> Result<Vec<Segment>, Error> {
        self.inner.get_segment_registers(registers)
    }

    fn set_segment_registers(
        &mut self,
        registers: &[SegmentRegister],
        values: &[Segment],
    ) -> Result<(), Error> {
        self.inner.set_segment_registers(registers, values)
    }

    fn get_descriptor_tables(
        &self,
        registers: &[DescriptorTableRegister],
    ) -> Result<Vec<DescriptorTable>, Error> {
        self.inner.get_descriptor_tables(registers)
    }

    fn set_descriptor_tables(
        &mut self,
        registers: &[DescriptorTableRegister],
        values: &[DescriptorTable],
    ) -> Result<(), Error> {
        self.inner.set_descriptor_tables(registers, values)
    }
}
