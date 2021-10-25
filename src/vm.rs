//! This module provides the [`Vm`] struct which represents a virtual machine, i.e. a number of
//! virtual CPUs and a physical memory space.

use bitflags::bitflags;
use crate::error::Error;
use crate::platform;
use crate::vcpu::Vcpu;

bitflags! {
    /// The protection flags used when mapping guest physical memory.
    pub struct ProtectionFlags: u32 {
        /// The guest VM is allowed to read from the physical memory.
        const READ    = 1 << 0;
        /// The guest VM is allowed to write to the physical memory.
        const WRITE   = 1 << 1;
        /// The guest VM is allowed to execute from the physical memory.
        const EXECUTE = 1 << 2;
    }
}

/// The `VmBuilder` allows for the configuration of certain properties for the new VM before
/// constructing it, as these properties may be immutable once the VM has been built.
pub struct VmBuilder {
    /// The internal platform-specific implementation of the [`platform::VmBuilder`] struct.
    pub(crate) inner: platform::VmBuilder,
}

impl VmBuilder {
    /// This is used to specify the maximum number of virtual CPUs to use for this VM.
    pub fn with_vcpu_count(self, count: usize) -> Result<Self, Error> {
        Ok(Self {
            inner: self.inner.with_vcpu_count(count)?,
        })
    }

    /// Builds the VM and returns a [`Vm`].
    pub fn build(self) -> Result<Vm, Error> {
        Ok(Vm {
            inner: self.inner.build()?,
        })
    }
}

/// The `Vm` struct represents a virtual machine. More specifically, it represents an abstraction
/// over a number of virtual CPUs and a physical memory space.
pub struct Vm {
    /// The internal platform-specific implementation of the [`platform::Vm`] struct.
    pub(crate) inner: platform::Vm,
}

impl Vm {
    /// Create a virtual CPU with the given vCPU ID.
    pub fn create_vcpu(&mut self, id: usize) -> Result<Vcpu, Error> {
        Ok(Vcpu {
            inner: self.inner.create_vcpu(id)?,
        })
    }

    /// Maps guest physical memory into the VM's address space. More specifically this function
    /// takes a virtual address as `bytes`, resolves it to the host physical address and maps it to
    /// the specified guest physical address `guest_address` with the specified protection
    /// [`ProtectionFlags`] and the specified `size`, which must be page size aligned.
    ///
    /// This function is `unsafe`. You must ensure that `bytes` and `size` span a region of virtual
    /// memory that is valid.
    pub unsafe fn map_physical_memory(
        &mut self,
        guest_address: u64,
        bytes: *mut std::ffi::c_void,
        size: usize,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        self.inner.map_physical_memory(guest_address, bytes, size, protection)
    }

    /// Unmaps the guest physical memory.
    pub fn unmap_physical_memory(
        &mut self,
        guest_address: u64,
    ) -> Result<(), Error> {
        self.inner.unmap_physical_memory(guest_address)
    }

    /// Changes the protection flags of the guest physical memory.
    pub fn protect_physical_memory(
        &mut self,
        guest_address: u64,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        self.inner.protect_physical_memory(guest_address, protection)
    }
}