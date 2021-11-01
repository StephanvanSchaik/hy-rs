#![allow(non_camel_case_types)]

use crate::error::Error;
use nix::{ioctl_readwrite, ioctl_write_ptr};
use sysctl::Sysctl;

const VM_MAGIC: u8 = b'v';

const VM_RUN:            u8 = 1;
const VM_SET_CAPABILITY: u8 = 2;
const VM_GET_CAPABILITY: u8 = 3;

const VM_SET_REGISTER:           u8 = 20;
const VM_GET_REGISTER:           u8 = 21;
const VM_SET_SEGMENT_DESCRIPTOR: u8 = 22;
const VM_GET_SEGMENT_DESCRIPTOR: u8 = 23;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum vm_reg_name {
    VM_REG_GUEST_RAX,
    VM_REG_GUEST_RBX,
    VM_REG_GUEST_RCX,
    VM_REG_GUEST_RDX,
    VM_REG_GUEST_RSI,
    VM_REG_GUEST_RDI,
    VM_REG_GUEST_RBP,
    VM_REG_GUEST_R8,
    VM_REG_GUEST_R9,
    VM_REG_GUEST_R10,
    VM_REG_GUEST_R11,
    VM_REG_GUEST_R12,
    VM_REG_GUEST_R13,
    VM_REG_GUEST_R14,
    VM_REG_GUEST_R15,
    VM_REG_GUEST_CR0,
    VM_REG_GUEST_CR3,
    VM_REG_GUEST_CR4,
    VM_REG_GUEST_DR7,
    VM_REG_GUEST_RSP,
    VM_REG_GUEST_RIP,
    VM_REG_GUEST_RFLAGS,
    VM_REG_GUEST_ES,
    VM_REG_GUEST_CS,
    VM_REG_GUEST_SS,
    VM_REG_GUEST_DS,
    VM_REG_GUEST_FS,
    VM_REG_GUEST_GS,
    VM_REG_GUEST_LDTR,
    VM_REG_GUEST_TR,
    VM_REG_GUEST_IDTR,
    VM_REG_GUEST_GDTR,
    VM_REG_GUEST_EFER,
    VM_REG_LAST,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum vm_exitcode {
    VM_EXITCODE_INOUT,
    VM_EXITCODE_VMX,
    VM_EXITCODE_BOGUS,
    VM_EXITCODE_RDMSR,
    VM_EXITCODE_WRMSR,
    VM_EXITCODE_HLT,
    VM_EXITCODE_MTRAP,
    VM_EXITCODE_PAUSE,
    VM_EXITCODE_PAGING,
    VM_EXITCODE_INST_EMUL,
    VM_EXITCODE_SPINUP_AP,
    VM_EXITCODE_MAX,
}

#[repr(C)]
pub struct vm_exit {
    pub exitcode: vm_exitcode,
    pub inst_length: i32,
    pub rip: u64,
}

#[repr(C)]
pub struct vm_run {
    pub cpuid: i32,
    pub rip: u64,
    pub vm_exit: vm_exit,
}

#[repr(C)]
pub struct vm_register {
    pub cpuid: i32,
    pub regnum: vm_reg_name,
    pub value: u64,
}

#[repr(C)]
pub struct seg_desc {
    pub base: u64,
    pub limit: u32,
    pub access: u32,
}

#[repr(C)]
pub struct vm_seg_desc {
    pub cpuid: i32,
    pub regnum: vm_reg_name,
    pub desc: seg_desc,
}

pub fn vm_create(name: &str) -> Result<(), Error> {
    let ctl = sysctl::Ctl::new("hw.vmm.create")?;

    ctl.set_value_string(name)?;

    Ok(())
}

pub fn vm_destroy(name: &str) -> Result<(), Error> {
    let ctl = sysctl::Ctl::new("hw.vmm.destroy")?;

    ctl.set_value_string(name)?;

    Ok(())
}

ioctl_readwrite!(vm_run, VM_MAGIC, VM_RUN, vm_run);
ioctl_write_ptr!(vm_set_register, VM_MAGIC, VM_SET_REGISTER, vm_register);
ioctl_readwrite!(vm_get_register, VM_MAGIC, VM_GET_REGISTER, vm_register);
ioctl_write_ptr!(vm_set_segment_descriptor, VM_MAGIC, VM_SET_SEGMENT_DESCRIPTOR, vm_seg_desc);
ioctl_readwrite!(vm_get_segment_descriptor, VM_MAGIC, VM_GET_SEGMENT_DESCRIPTOR, vm_seg_desc);
