//! This module provides code specific to the x86-64 architecture.

use bitflags::bitflags;
use crate::error::Error;
use num_derive::FromPrimitive;

/// Represents the general-purpose registers of the x86-64 architecture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Register {
    /// The accumulator register.
    Rax,
    /// The counter register.
    Rcx,
    /// The data register.
    Rdx,
    /// The base register.
    Rbx,
    /// The stack pointer register.
    Rsp,
    /// The base pointer register.
    Rbp,
    /// The source index register.
    Rsi,
    /// The destination index register.
    Rdi,
    /// The R8 register.
    R8,
    /// The R9 register.
    R9,
    /// The R10 register.
    R10,
    /// The R11 register.
    R11,
    /// The R12 register.
    R12,
    /// The R13 register.
    R13,
    /// The R14 register.
    R14,
    /// The R15 register.
    R15,
    /// The instruction pointer register.
    Rip,
    /// The status register.
    Rflags,
}

/// Protected Mode Enable.
pub const CR0_PE: u64 = 1 << 0;
/// Monitor Co-Processor.
pub const CR0_MP: u64 = 1 << 1;
/// Emulation.
pub const CR0_EM: u64 = 1 << 2;
/// Task Switched.
pub const CR0_TS: u64 = 1 << 3;
/// Extension Type.
pub const CR0_ET: u64 = 1 << 4;
/// Numeric Error.
pub const CR0_NE: u64 = 1 << 5;
/// Write Protect.
pub const CR0_WP: u64 = 1 << 16;
/// Alignment Mask.
pub const CR0_AM: u64 = 1 << 18;
/// Not write-through.
pub const CR0_NW: u64 = 1 << 29;
/// Cache Disable.
pub const CR0_CD: u64 = 1 << 30;
/// Paging.
pub const CR0_PG: u64 = 1 << 31;

/// Virtual 8086 Mode Extension.
pub const CR4_VME:        u64 = 1 << 0;
/// Protected Mode Virtual Interrupts.
pub const CR4_PVI:        u64 = 1 << 1;
/// Time Stamp Disable (only enabled in ring 0).
pub const CR4_TSD:        u64 = 1 << 2;
/// Debugging Extension.
pub const CR4_DE:         u64 = 1 << 3;
/// Page Size Extension.
pub const CR4_PSE:        u64 = 1 << 4;
/// Physical Address Extension.
pub const CR4_PAE:        u64 = 1 << 5;
/// Machine Check Exception.
pub const CR4_MCE:        u64 = 1 << 6;
/// Page Global Enable.
pub const CR4_PGE:        u64 = 1 << 7;
/// Performance Monitoring Counter Enable.
pub const CR4_PCE:        u64 = 1 << 8;
/// OS support for `fxsave` and `fxrstor`.
pub const CR4_OSFXSR:     u64 = 1 << 9;
/// OS support for unmasked SIMD floating-point exceptions.
pub const CR4_OSXMMEXCPT: u64 = 1 << 10;
/// User Mode Instruction Prevention (disables `sgdt`, sidt`, `sldt`, `smsw` and `str` are disabled
/// in user mode).
pub const CR4_UMIP:       u64 = 1 << 11;
/// Virtual Machine eXtension Enable.
pub const CR4_VMXE:       u64 = 1 << 13;
/// Safer Mode eXtension Enable.
pub const CR4_SMXE:       u64 = 1 << 14;
/// Enable FSGSBASE instructions.
pub const CR4_FSGSBASE:   u64 = 1 << 16;
/// PCID enable.
pub const CR4_PCIDE:      u64 = 1 << 17;
/// XSAVE and Processor Extended States enable.
pub const CR4_OSXSAVE:    u64 = 1 << 18;
/// Supervisor Mode Execution Protection enable.
pub const CR4_SMEP:       u64 = 1 << 20;
/// Supervisor Mode Access Protectection enable.
pub const CR4_SMAP:       u64 = 1 << 21;
/// Enable protection keys for user mode pages.
pub const CR4_PKE:        u64 = 1 << 22;
/// Enable control-flow enforcement technology.
pub const CR4_CET:        u64 = 1 << 23;
/// Enable protection keys for supervisor-mode pages.
pub const CR4_PKS:        u64 = 1 << 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlRegister {
    /// Control register CR0.
    Cr0,
    /// Control register CR1.
    Cr1,
    /// Control register CR2. This contains the linear address that caused a page fault in the
    /// event of a page fault.
    Cr2,
    /// Control register CR3. This contains the physical address of the page table at the root of
    /// the page table hierarchy.
    Cr3,
    /// Control register CR4.
    Cr4,
    /// Control register CR8.
    Cr8,
}

/// Represents a segment descriptor on the x86-64 architecture.
#[derive(Clone, Debug, Default)]
pub struct Segment {
    /// The base address of the segment.
    pub base: u64,
    /// The limit of the segment.
    pub limit: u32,
    /// The segment selector, i.e. the value stored in the actual segment registers described by
    /// [`SegmentRegister`]. For 16-bit real mode, this describes a 16-bit value that is multiplied
    /// by 16 to get the base of the segment. For 32-bit protected mode and 64-bit long mode, this
    /// describes an index into the global descriptor table.
    pub selector: u16,
    /// The type of the segment.
    pub segment_type: u8,
    /// Whether the segment descriptor describes a system segment or not.
    pub non_system_segment: bool,
    /// The privilege level of the segment, where 0 is supervisor mode and 3 is user mode.
    pub dpl: u8,
    /// Whether the segment descriptor is valid/present.
    pub present: bool,
    pub available: bool,
    /// Whether this segment uses long mode. This is only checked for 64-bit code segments.
    pub long: bool,
    pub default: bool,
    /// Whether the limit is described in bytes or in units of 4 kiB.
    pub granularity: bool,
}

/// Represents the segment registers of the x86-64 architecture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentRegister {
    /// The code segment register.
    Cs,
    /// The data segment register.
    Ds,
    /// The ES segment register.
    Es,
    /// The FS segment register.
    Fs,
    /// The GS segment register.
    Gs,
    /// The stack segment register.
    Ss,
    /// The task register.
    Tr,
    /// The local descriptor table.
    Ldt,
}

/// Represents the descriptor table rgisters of the x86-64 architecture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DescriptorTableRegister {
    /// The global descrptor table,
    Gdt,
    /// The interrupt descriptor table.
    Idt,
}

/// Represents a descriptor table on the x86-64 architecture.
#[derive(Clone, Debug)]
pub struct DescriptorTable {
    /// The base address of the descriptor table.
    pub base: u64,
    /// The limit of the descriptor table.
    pub limit: u16,
}

/// The code segment to load when issuing the `sysenter` instruction.
pub const MSR_IA32_SYSENTER_CS:    u32 = 0x0000_0174;
/// The stack pointer to load when issuing the `sysenter` instruction.
pub const MSR_IA32_SYSENTER_ESP:   u32 = 0x0000_0175;
/// The instruction pointer to load when issuing the `sysenter` instruction.
pub const MSR_IA32_SYSENTER_EIP:   u32 = 0x0000_0176;

/// The Extended Feature Enable Register (EFER).
pub const MSR_IA32_EFER:           u32 = 0xc000_0080;

/// Enables the `syscall` extension.
pub const EFER_SCE: u64 = 1 << 0;
/// Enables long mode.
pub const EFER_LME: u64 = 1 << 8;
/// Indicates long mode is active.
pub const EFER_LMA: u64 = 1 << 10;
/// Enables the non-executable bit.
pub const EFER_NXE: u64 = 1 << 11;

/// The user segment base \[48:63\], the kernel segment base \[32:47\] and the syscall EIP
/// \[0:31\].
pub const MSR_IA32_STAR:           u32 = 0xc000_0081;
/// The instruction pointer to load when issuing a `syscall` in 64-bit mode.
pub const MSR_IA32_LSTAR:          u32 = 0xc000_0082;
/// The instruction pointer to load when issuing a `syscall` in 32-bit compatibility mode.
pub const MSR_IA32_CSTAR:          u32 = 0xc000_0083;
/// Bits set in the syscall mask clear the corresponding bits in the `rflags` register when issuing
/// a `syscall` instruction.
pub const MSR_IA32_SYSCALL_MASK:   u32 = 0xc000_0084;
/// The GS segment to swap when issuing the `swapgs` instruction.
pub const MSR_IA32_KERNEL_GS_BASE: u32 = 0xc000_0102;

/// Extends the virtual CPU with functions to access the architecture-specific registers.
pub trait CpuRegs {
    /// Gets the general-purpose registers specified by the array of [`Register`]s.
    fn get_registers(
        &self,
        registers: &[Register],
    ) -> Result<Vec<u64>, Error>;

    /// Sets the general-purpose registers specified by the array of [`Register`]s to the
    /// corresponding values.
    fn set_registers(
        &mut self,
        registers: &[Register],
        values: &[u64],
    ) -> Result<(), Error>;

    /// Gets the control registers specified by the array of [`ControlRegister`]s.
    fn get_control_registers(
        &self,
        registers: &[ControlRegister],
    ) -> Result<Vec<u64>, Error>;

    /// Sets the control registers specified by the array of [`ControlRegister`]s to the
    /// corresponding values.
    fn set_control_registers(
        &mut self,
        registers: &[ControlRegister],
        values: &[u64],
    ) -> Result<(), Error>;

    /// Gets the model-specific registers specified by the array of [`u32`]s.
    fn get_msrs(
        &self,
        registers: &[u32],
    ) -> Result<Vec<u64>, Error>;

    /// Sets the model-specific registers specified by the array of [`u32`]s to the corresponding
    /// values.
    fn set_msrs(
        &mut self,
        registers: &[u32],
        values: &[u64],
    ) -> Result<(), Error>;

    /// Gets the segment registers specified by the array of [`SegmentRegister`]s.
    fn get_segment_registers(
        &self,
        registers: &[SegmentRegister],
    ) -> Result<Vec<Segment>, Error>;

    /// Sets the segment registers specified by the array of [`SegmentRegister`]s to the
    /// corresponding values.
    fn set_segment_registers(
        &mut self,
        registers: &[SegmentRegister],
        values: &[Segment],
    ) -> Result<(), Error>;

    /// Gets the descriptor tables specified by the array of [`DescriptorTableRegister`]s.
    fn get_descriptor_tables(
        &self,
        registers: &[DescriptorTableRegister],
    ) -> Result<Vec<DescriptorTable>, Error>;

    /// Sets the descriptor tables specified by the array of [`DescriptorTableRegister`]s to the
    /// corresponding values.
    fn set_descriptor_tables(
        &mut self,
        registers: &[DescriptorTableRegister],
        values: &[DescriptorTable],
    ) -> Result<(), Error>;
}

bitflags! {
    pub struct CpuBased: u32 {
        const IRQ_WND            = 1 << 2;
        const TSC_OFFSET         = 1 << 3;
        const HLT                = 1 << 7;
        const INVLPG             = 1 << 9;
        const MWAIT              = 1 << 10;
        const RDPMC              = 1 << 11;
        const RDTSC              = 1 << 12;
        const CR3_LOAD           = 1 << 15;
        const CR3_STORE          = 1 << 16;
        const CR8_LOAD           = 1 << 19;
        const CR8_STORE          = 1 << 20;
        const TPR_SHADOW         = 1 << 21;
        const VIRTUAL_NMI_WND    = 1 << 22;
        const MOV_DR             = 1 << 23;
        const UNCONDITIONAL_IO   = 1 << 24;
        const IO_BITMAPS         = 1 << 25;
        const MTF                = 1 << 27;
        const MSR_BITMAPS        = 1 << 28;
        const MONITOR            = 1 << 29;
        const PAUSE              = 1 << 30;
        const SECONDARY_CONTROLS = 1 << 31;
    }

    pub struct CpuBased2: u32 {
        const UNRESTRICTED_GUEST = 1 << 7;
    }

    pub struct VmEntryControls: u32 {
        const GUEST_IA32E               = 1 << 9;
        const SMM                       = 1 << 10;
        const DEACTIVE_DUAL_MONITOR     = 1 << 11;
        const LOAD_PERF_GLOBAL_CONTROLS = 1 << 13;
        const LOAD_PAT                  = 1 << 14;
        const LOAD_EFER                 = 1 << 15;
    }
}

/// The possible fields of the VMCS struct.
#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Vmcs {
    /// The ES register of the guest.
    GuestEs               = 0x0000_0800,
    /// The code segment register of the guest.
    GuestCs               = 0x0000_0802,
    /// The stack segment register of the guest.
    GuestSs               = 0x0000_0804,
    /// The data segment register of the guest.
    GuestDs               = 0x0000_0806,
    /// The FS register of the guest.
    GuestFs               = 0x0000_0808,
    /// The GS register of the guest.
    GuestGs               = 0x0000_080a,
    /// The local descriptor table register of the guest.
    GuestLdtr             = 0x0000_080c,
    /// The task register of the guest.
    GuestTr               = 0x0000_080e,
    /// The guest physical address that caused an EPT violation.
    GuestPhysicalAddress  = 0x0000_2400,
    /// The EFER MSR of the guest.
    GuestEfer             = 0x0000_2806,
    /// Pin-based controls.
    PinBased              = 0x0000_4000,
    /// CPU-based controls.
    CpuBased              = 0x0000_4002,
    /// VM exit controls.
    VmExitControls        = 0x0000_400c,
    /// VM entry controls.
    VmEntryControls       = 0x0000_4012,
    /// Secondary CPU-based controls.
    CpuBased2             = 0x0000_401e,
    /// The reason for the VM exit.
    ExitReason            = 0x0000_4402,
    /// The ES limit of the guest.
    GuestEsLimit          = 0x0000_4800,
    /// The code segment limit of the guest.
    GuestCsLimit          = 0x0000_4802,
    /// The stack segment limit of the guest.
    GuestSsLimit          = 0x0000_4804,
    /// The data segment limit of the guest.
    GuestDsLimit          = 0x0000_4806,
    /// The FS limit of the guest.
    GuestFsLimit          = 0x0000_4808,
    /// The GS limit of the guest.
    GuestGsLimit          = 0x0000_480a,
    /// The LDT limit of the guest.
    GuestLdtrLimit        = 0x0000_480c,
    /// The task register limit of the guest.
    GuestTrLimit          = 0x0000_480e,
    /// The GDT limit of the guest.
    GuestGdtrLimit        = 0x0000_4810,
    /// The IDT limit of the guest.
    GuestIdtrLimit        = 0x0000_4812,
    /// The ES access rights of the guest.
    GuestEsAccessRights   = 0x0000_4814,
    /// The code segment access rights of the guest.
    GuestCsAccessRights   = 0x0000_4816,
    GuestSsAccessRights   = 0x0000_4818,
    GuestDsAccessRights   = 0x0000_481a,
    GuestFsAccessRights   = 0x0000_481c,
    GuestGsAccessRights   = 0x0000_481e,
    GuestLdtrAccessRights = 0x0000_4820,
    GuestTrAccessRights   = 0x0000_4822,
    Cr0Mask               = 0x0000_6000,
    Cr4Mask               = 0x0000_6002,
    Cr0Shadow             = 0x0000_6004,
    Cr4Shadow             = 0x0000_6006,
    GuestLinearAddress    = 0x0000_640a,
    GuestCr0              = 0x0000_6800,
    GuestCr3              = 0x0000_6802,
    GuestCr4              = 0x0000_6804,
    GuestEsBase           = 0x0000_6806,
    GuestCsBase           = 0x0000_6808,
    GuestSsBase           = 0x0000_680a,
    GuestDsBase           = 0x0000_680c,
    GuestFsBase           = 0x0000_680e,
    GuestGsBase           = 0x0000_6810,
    GuestLdtrBase         = 0x0000_6812,
    GuestTrBase           = 0x0000_6814,
    GuestGdtrBase         = 0x0000_6816,
    GuestIdtrBase         = 0x0000_6818,
}

#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone, Debug, Eq, FromPrimitive, PartialEq)]
#[repr(u32)]
pub enum VmxReason {
    ExcNmi            =  0,
    Irq               =  1,
    TripleFault       =  2,
    Init              =  3,
    Sipi              =  4,
    IoSmi             =  5,
    OtherSmi          =  6,
    IrqWnd            =  7,
    VirtualNmiWnd     =  8,
    Task              =  9,
    Cpuid             = 10,
    Getsec            = 11,
    Hlt               = 12,
    Invd              = 13,
    Invlpg            = 14,
    Rdpmc             = 15,
    Rdtsc             = 16,
    Rsm               = 17,
    VmCall            = 18,
    VmClear           = 19,
    VmLaunch          = 20,
    VmPtrLd           = 21,
    VmPtrSt           = 22,
    VmRead            = 23,
    VmResume          = 24,
    VmWrite           = 25,
    VmOff             = 26,
    VmOn              = 27,
    MovCr             = 28,
    MovDr             = 29,
    Io                = 30,
    Rdmsr             = 31,
    Wrmsr             = 32,
    VmEntryGuest      = 33,
    VmEntryMsr        = 34,
    Mwait             = 36,
    Mtf               = 37,
    Monitor           = 39,
    Pause             = 40,
    VmEntryMc         = 41,
    TprThreshold      = 43,
    ApicAccess        = 44,
    VirtualizedEoi    = 45,
    GdtrIdtr          = 46,
    LdtrTr            = 47,
    EptViolation      = 48,
    EptMisconfig      = 49,
    EptInvept         = 50,
    Rdtscp            = 51,
    VmxTimerExpired   = 52,
    Invpid            = 53,
    Wbinvd            = 54,
    Xsetbv            = 55,
    ApicWrite         = 56,
    Rdrand            = 57,
    Invpcid           = 58,
    VmFunc            = 59,
    Rdseed            = 61,
    Xsaves            = 63,
    Xrstors           = 64,
}
