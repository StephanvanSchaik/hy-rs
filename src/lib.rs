//! The hy-rs crate, pronounced as high rise, provides a unified and portable interface to the
//! hypervisor APIs provided by various platforms. More specifically, this crate provides a
//! portable interface to create and configure VMs.
//!
//! This crate supports the following platforms:
//!  * Microsoft Windows through the [WinHV
//!  API](https://docs.microsoft.com/en-us/virtualization/api/hypervisor-platform/hypervisor-platform) or Hyper-V.
//!  * Linux through the [KVM API](https://github.com/rust-vmm/kvm-ioctls).
//!  * Mac OS X through [Apple's Hypervisor
//!  Framework](https://developer.apple.com/documentation/hypervisor/).

pub mod arch;
pub mod error;
pub mod hypervisor;
pub mod mmap;
pub mod vm;
pub mod vcpu;
mod os_impl;

#[cfg(target_os = "freebsd")]
pub(crate) use os_impl::freebsd as platform;
#[cfg(target_os = "linux")]
pub(crate) use os_impl::linux as platform;
#[cfg(target_os = "macos")]
pub(crate) use os_impl::macos as platform;
#[cfg(target_os = "windows")]
pub(crate) use os_impl::windows as platform;

pub use error::Error;
pub use hypervisor::Hypervisor;
pub use vm::{ProtectionFlags, Vm, VmBuilder};
pub use vcpu::{ExitReason, Vcpu};
