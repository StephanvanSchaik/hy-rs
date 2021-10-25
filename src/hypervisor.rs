//! This module provides the [`Hypervisor`] struct, which mostly serves as an entry point to the
//! API, as some platforms require some state to use the underlying API. For instance, KVM requires
//! an open file descriptor to `/dev/kvm`.

use crate::error::Error;
use crate::platform;
use crate::vm::VmBuilder;

/// The `Hypervisor` struct serving as an entry point to the API.
pub struct Hypervisor {
    /// The internal platform-specific implementation of the [`platform::Hypervisor`] struct.
    inner: platform::Hypervisor,
}

impl Hypervisor {
    /// Creates a new `Hypervisor` struct to access the underlying hypervisor API for the current
    /// platform.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: platform::Hypervisor::new()?,
        })
    }

    /// Returns a [`VmBuilder`] that uses the builder pattern to create a new VM. This allows the
    /// configuration of certain properties for the VM on platforms where these become immutable
    /// the moment you build the VM.
    pub fn build_vm(&self) -> Result<VmBuilder, Error> {
        Ok(VmBuilder {
            inner: self.inner.build_vm()?,
        })
    }
}
