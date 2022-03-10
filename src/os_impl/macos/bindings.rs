#![allow(dead_code)]
#![allow(non_camel_case_types)]

use crate::error::Error;
use std::os::raw::c_uint;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::Vmcs;

pub type hv_return_t = u32;

pub const HV_SUCCESS:      hv_return_t = 0;
pub const HV_ERROR:        hv_return_t = 0xfae94001;
pub const HV_BUSY:         hv_return_t = 0xfae94002;
pub const HV_BAD_ARGUMENT: hv_return_t = 0xfae94003;
pub const HV_NO_RESOURCES: hv_return_t = 0xfae94005;
pub const HV_NO_DEVICE:    hv_return_t = 0xfae94006;
pub const HV_DENIED:       hv_return_t = 0xfae94007;
pub const HV_UNSUPPORTED:  hv_return_t = 0xfae9400f;

pub trait IntoResult {
    fn into_result(self) -> Result<(), Error>;
}

impl IntoResult for hv_return_t {
    fn into_result(self) -> Result<(), Error> {
        match self {
            HV_SUCCESS => Ok(()),
            status => Err(Error::HypervisorError(status)),
        }
    }
}

pub type hv_vm_options_t = u64;
pub const HV_VM_DEFAULT: hv_vm_options_t = 0 << 0;

pub type hv_uvaddr_t = *const std::ffi::c_void;
pub type hv_gpaddr_t = u64;
pub type hv_memory_flags_t = u64;

pub const HV_MEMORY_READ:  hv_memory_flags_t = 1 << 0;
pub const HV_MEMORY_WRITE: hv_memory_flags_t = 1 << 1;
pub const HV_MEMORY_EXEC:  hv_memory_flags_t = 1 << 2;

pub type hv_vcpuid_t = c_uint;
pub const HV_VCPU_DEFAULT: u64 = 0;

pub type hv_exit_reason_t = u32;
pub type hv_exception_syndrome_t = u64;
pub type hv_exception_address_t = u64;
pub type hv_ipa_t = u64;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hv_vcpu_exit_exception_t {
    pub syndrome: hv_exception_syndrome_t,
    pub virtual_address: hv_exception_address_t,
    pub physical_address: hv_ipa_t,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct hv_vcpu_exit_t {
    pub reason: hv_exit_reason_t,
    pub exception: hv_vcpu_exit_exception_t,
}

pub type hv_vcpu_config_t = *mut core::ffi::c_void;

extern {
    pub fn hv_vm_create(flags: hv_vm_options_t) -> hv_return_t;
    pub fn hv_vm_destroy() -> hv_return_t;

    pub fn hv_vm_map(uva: hv_uvaddr_t, gpa: hv_gpaddr_t, size_t: usize, flags: hv_memory_flags_t) -> hv_return_t;
    pub fn hv_vm_unmap(gpa: hv_gpaddr_t, size: usize) -> hv_return_t;
    pub fn hv_vm_protect(gpa: hv_gpaddr_t, size: usize, flags: hv_memory_flags_t) -> hv_return_t;

    pub fn hv_vcpu_destroy(vcpu: hv_vcpuid_t) -> hv_return_t;
    pub fn hv_vcpu_run(vcpu: hv_vcpuid_t) -> hv_return_t;
}

#[cfg(target_arch = "x86_64")]
extern {
    pub fn hv_vcpu_create(vcpu: *mut hv_vcpuid_t, flags: hv_vm_options_t) -> hv_return_t;
}

#[cfg(target_arch = "aarch64")]
extern {
    pub fn hv_vcpu_create(
        vcpu: *mut hv_vcpuid_t,
        exit: *mut *const hv_vcpu_exit_t,
        config: *const hv_vcpu_config_t,
    ) -> hv_return_t;
}

#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
/// The type that defines x86 architectural registers.
pub enum hv_x86_reg_t {
    /// The value that identifies the x86 instruction pointer register.
    HV_X86_RIP,
    /// The value that identifies the x86 status register.
    HV_X86_RFLAGS,
    /// The value that identifies the x86 accumulator register.
    HV_X86_RAX,
    /// The value that identifies the x86 counter register.
    HV_X86_RCX,
    /// The value that identifies the x86 data register.
    HV_X86_RDX,
    /// The value that identifies the x86 base register.
    HV_X86_RBX,
    /// The value that identifies the x86 source index register.
    HV_X86_RSI,
    /// The value that identifies the x86 destination index register.
    HV_X86_RDI,
    /// The value that identifies the x86 stack pointer register.
    HV_X86_RSP,
    /// The value that identifies the x86 base pointer register.
    HV_X86_RBP,
    /// The value that identifies the x86 general-purpose register R8.
    HV_X86_R8,
    /// The value that identifies the x86 general-purpose register R9.
    HV_X86_R9,
    /// The value that identifies the x86 general-purpose register R10.
    HV_X86_R10,
    /// The value that identifies the x86 general-purpose register R11.
    HV_X86_R11,
    /// The value that identifies the x86 general-purpose register R12.
    HV_X86_R12,
    /// The value that identifies the x86 general-purpose register R13.
    HV_X86_R13,
    /// The value that identifies the x86 general-purpose register R14.
    HV_X86_R14,
    /// The value that identifies the x86 general-purpose register R15.
    HV_X86_R15,
    /// The value that identifies the x86 code-segment register.
    HV_X86_CS,
    /// The value that identifies the x86 stack-segment register.
    HV_X86_SS,
    /// The value that identifies the x86 data-segment register.
    HV_X86_DS,
    /// The value that identifies the x86 segment register ES.
    HV_X86_ES,
    /// The value that identifies the x86 segment register FS.
    HV_X86_FS,
    /// The value that identifies the x86 segment register GS.
    HV_X86_GS,
    HV_X86_IDT_BASE,
    HV_X86_IDT_LIMIT,
    HV_X86_GDT_BASE,
    HV_X86_GDT_LIMIT,
    HV_X86_LDTR,
    HV_X86_LDT_BASE,
    HV_X86_LDT_LIMIT,
    HV_X86_LDT_AR,
    HV_X86_TR,
    HV_X86_TSS_BASE,
    HV_X86_TSS_LIMIT,
    HV_X86_TSS_AR,
    HV_X86_CR0,
    HV_X86_CR1,
    HV_X86_CR2,
    HV_X86_CR3,
    /// The value that identifies the x86 control register CR4.
    HV_X86_CR4,
}

#[cfg(target_arch = "x86_64")]
extern {
    pub fn hv_vcpu_read_register(vcpu: hv_vcpuid_t, reg: hv_x86_reg_t, value: *mut u64) -> hv_return_t;
    pub fn hv_vcpu_write_register(vcpu: hv_vcpuid_t, reg: hv_x86_reg_t, value: u64) -> hv_return_t;
    pub fn hv_vcpu_read_msr(vcpu: hv_vcpuid_t, msr: u32, value: *mut u64) -> hv_return_t;
    pub fn hv_vcpu_write_msr(vcpu: hv_vcpuid_t, msr: u32, value: u64) -> hv_return_t;
    pub fn hv_vcpu_enable_native_msr(vcpu: hv_vcpuid_t, msr: u32, value: bool) -> hv_return_t;
    pub fn hv_vmx_vcpu_read_vmcs(vcpu: hv_vcpuid_t, field: Vmcs, value: *mut u64) -> hv_return_t;
    pub fn hv_vmx_vcpu_write_vmcs(vcpu: hv_vcpuid_t, field: Vmcs, value: u64) -> hv_return_t;
}
