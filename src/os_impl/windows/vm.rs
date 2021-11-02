use crate::error::Error;
use crate::mmap::MmapMut;
use crate::vm::ProtectionFlags;
use mmap_rs::MmapOptions;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use super::bindings::*;
use super::vcpu::Vcpu;

pub struct PartitionHandle(pub WHV_PARTITION_HANDLE);

impl Drop for PartitionHandle {
    fn drop(&mut self) {
        let _ = unsafe {
            WHvDeletePartition(self.0)
        };
    }
}
pub struct VmBuilder {
    pub(crate) handle: PartitionHandle,
}

impl VmBuilder {
    pub fn with_vcpu_count(self, count: usize) -> Result<Self, Error> {
        let property = WHV_PARTITION_PROPERTY {
            ProcessorCount: count as u32,
        };

        unsafe {
            WHvSetPartitionProperty(
                self.handle.0,
                WHvPartitionPropertyCodeProcessorCount,
                &property as *const WHV_PARTITION_PROPERTY as *const std::ffi::c_void,
                std::mem::size_of::<WHV_PARTITION_PROPERTY>() as u32,
            )
        }?;

        Ok(self)
    }

    pub fn build(self, _name: &str) -> Result<Vm, Error> {
        unsafe {
            WHvSetupPartition(self.handle.0)
        }?;

        Ok(Vm {
            handle: Arc::new(self.handle),
            regions: HashMap::new(),
        })
    }
}

struct MemoryRegion {
    bytes: *mut std::ffi::c_void,
    size: u64,
}

pub struct Vm {
    pub(crate) handle: Rc<PartitionHandle>,
    regions: HashMap<u64, MemoryRegion>,
}

impl Vm {
    pub fn create_vcpu(&mut self, id: usize) -> Result<Vcpu, Error> {
        unsafe {
            WHvCreateVirtualProcessor(
                self.handle.deref().0,
                id as u32,
                0,
            )
        }?;

        Ok(Vcpu {
            handle: self.handle.clone(),
            id: id as u32,
        })
    }

    pub fn allocate_physical_memory(
        &mut self,
        guest_address: u64,
        size: usize,
        protection: ProtectionFlags,
    ) -> Result<MmapMut, Error> {
        let mut inner = MmapOptions::new()
            .with_size(size)
            .map_mut()?;

        unsafe {
            self.map_physical_memory(
                guest_address,
                inner.as_mut_ptr() as *mut std::ffi::c_void,
                inner.size(),
                protection,
            )
        }?;

        Ok(MmapMut {
            vm: None,
            inner: Some(inner),
            guest_address,
        })
    }

    pub unsafe fn map_physical_memory(
        &mut self,
        guest_address: u64,
        bytes: *mut std::ffi::c_void,
        size: usize,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        let mut flags = WHvMapGpaRangeFlagNone;

        if protection.contains(ProtectionFlags::READ) {
            flags |= WHvMapGpaRangeFlagRead;
        }

        if protection.contains(ProtectionFlags::WRITE) {
            flags |= WHvMapGpaRangeFlagWrite;
        }

        if protection.contains(ProtectionFlags::EXECUTE) {
            flags |= WHvMapGpaRangeFlagExecute;
        }

        unsafe {
            WHvMapGpaRange(
                self.handle.deref().0,
                bytes,
                guest_address,
                size as u64,
                flags,
            )
        }?;

        self.regions.insert(guest_address, MemoryRegion {
            bytes,
            size: size as u64,
        });

        Ok(())
    }

    pub fn unmap_physical_memory(
        &mut self,
        guest_address: u64,
    ) -> Result<(), Error> {
        let region = match self.regions.remove(&guest_address) {
            Some(region) => region,
            _ => return Err(Error::InvalidGuestAddress),
        };

        unsafe {
            WHvUnmapGpaRange(
                self.handle.deref().0,
                guest_address,
                region.size,
            )
        }?;

        Ok(())
    }

    pub fn protect_physical_memory(
        &mut self,
        guest_address: u64,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        let region = match self.regions.get(&guest_address) {
            Some(region) => region,
            _ => return Err(Error::InvalidGuestAddress),
        };

        let mut flags = WHvMapGpaRangeFlagNone;

        if protection.contains(ProtectionFlags::READ) {
            flags |= WHvMapGpaRangeFlagRead;
        }

        if protection.contains(ProtectionFlags::WRITE) {
            flags |= WHvMapGpaRangeFlagWrite;
        }

        if protection.contains(ProtectionFlags::EXECUTE) {
            flags |= WHvMapGpaRangeFlagExecute;
        }

        unsafe {
            WHvUnmapGpaRange(
                self.handle.deref().0,
                guest_address,
                region.size,
            )
        }?;

        unsafe {
            WHvMapGpaRange(
                self.handle.deref().0,
                region.bytes,
                guest_address,
                region.size,
                flags,
            )
        }?;

        Ok(())
    }
}
