use crate::error::Error;
use crate::vm::ProtectionFlags;
use mmap_rs::{MmapMut, MmapOptions};
use rangemap::RangeMap;
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
            segments: HashMap::new(),
            physical_ranges: RangeMap::new(),
        })
    }
}

pub struct Vm {
    pub(crate) handle: Arc<PartitionHandle>,
    pub(crate) segments: HashMap<u64, MmapMut>,
    pub(crate) physical_ranges: RangeMap<u64, u64>,
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
    ) -> Result<(), Error> {
        let mapping = MmapOptions::new(size)
            .map_mut()?;

        self.map_physical_memory(
            guest_address,
            mapping,
            protection,
        )?;

        Ok(())
    }

    pub fn map_physical_memory(
        &mut self,
        guest_address: u64,
        mut mapping: MmapMut,
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

        let size = mapping.len() as u64;

        unsafe {
            WHvMapGpaRange(
                self.handle.deref().0,
                mapping.as_mut_ptr() as *mut std::ffi::c_void,
                guest_address,
                size,
                flags,
            )
        }?;

        self.segments.insert(guest_address, mapping);
        self.physical_ranges.insert(guest_address..guest_address + size, guest_address);

        Ok(())
    }

    pub fn unmap_physical_memory(
        &mut self,
        guest_address: u64,
    ) -> Result<(), Error> {
        // Look up the base guest address.
        let range = match self.physical_ranges.get_key_value(&guest_address) {
            Some((range, _)) => range.clone(),
            _ => return Err(Error::InvalidGuestAddress),
        };

        // Look up the segment size.
        let size = match self.segments.get(&range.start) {
            Some(segment) => segment.len() as u64,
            _ => return Err(Error::InvalidGuestAddress),
        };

        unsafe {
            WHvUnmapGpaRange(
                self.handle.deref().0,
                range.start,
                size,
            )
        }?;

        // Remove the physical address range and segment.
        self.segments.remove(&range.start);
        self.physical_ranges.remove(range);

        Ok(())
    }

    pub fn protect_physical_memory(
        &mut self,
        guest_address: u64,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        // Look up the base guest address.
        let range = match self.physical_ranges.get_key_value(&guest_address) {
            Some((range, _)) => range.clone(),
            _ => return Err(Error::InvalidGuestAddress),
        };

        // Look up the segment size.
        let mapping = match self.segments.get_mut(&range.start) {
            Some(segment) => segment,
            _ => return Err(Error::InvalidGuestAddress),
        };
        let size = mapping.len() as u64;

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
                range.start,
                size,
            )
        }?;

        unsafe {
            WHvMapGpaRange(
                self.handle.deref().0,
                mapping.as_mut_ptr() as *mut std::ffi::c_void,
                range.start,
                size,
                flags,
            )
        }?;

        Ok(())
    }

    pub fn read_physical_memory(
        &self,
        bytes: &mut [u8],
        guest_address: u64,
    ) -> Result<usize, Error> {
        // Look up the base guest address.
        let range = match self.physical_ranges.get_key_value(&guest_address) {
            Some((range, _)) => range.clone(),
            _ => return Err(Error::InvalidGuestAddress),
        };

        // Look up the segment.
        let segment = match self.segments.get(&range.start) {
            Some(segment) => segment,
            _ => return Err(Error::InvalidGuestAddress),
        };

        // Calculate the offset and size.
        let offset = (guest_address - range.start) as usize;
        let size = ((range.end - guest_address) as usize).min(bytes.len());

        bytes[..size].copy_from_slice(&segment[offset..offset + size]);

        Ok(size)
    }

    pub fn write_physical_memory(
        &mut self,
        guest_address: u64,
        bytes: &[u8],
    ) -> Result<usize, Error> {
        // Look up the base guest address.
        let range = match self.physical_ranges.get_key_value(&guest_address) {
            Some((range, _)) => range.clone(),
            _ => return Err(Error::InvalidGuestAddress),
        };

        // Look up the segment.
        let segment = match self.segments.get_mut(&range.start) {
            Some(segment) => segment,
            _ => return Err(Error::InvalidGuestAddress),
        };

        // Calculate the offset and size.
        let offset = (guest_address - range.start) as usize;
        let size = ((range.end - guest_address) as usize).min(bytes.len());

        segment[offset..offset + size].copy_from_slice(&bytes[..size]);

        Ok(size)
    }
}
