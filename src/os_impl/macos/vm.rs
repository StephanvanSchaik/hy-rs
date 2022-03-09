use crate::error::Error;
use crate::vm::ProtectionFlags;
use mmap_rs::{MmapMut, MmapOptions};
use rangemap::RangeMap;
use std::collections::HashMap;
use super::bindings::*;
use super::vcpu::Vcpu;

pub struct VmBuilder;

impl VmBuilder {
    pub fn with_vcpu_count(self, _count: usize) -> Result<Self, Error> {
        Ok(self)
    }

    pub fn build(self, _name: &str) -> Result<Vm, Error> {
        Ok(Vm {
            physical_ranges: RangeMap::new(),
            segments: HashMap::new(),
        })
    }
}

pub struct Segment {
    mapping: MmapMut,
}

pub struct Vm {
    physical_ranges: RangeMap<u64, u64>,
    segments: HashMap<u64, Segment>,
}

impl Vm {
    pub fn create_vcpu(&mut self, _id: usize) -> Result<Vcpu, Error> {
        let mut vcpu = 0;

        unsafe {
            hv_vcpu_create(&mut vcpu, HV_VCPU_DEFAULT)
        }.into_result()?;

        let mut vcpu = Vcpu {
            vcpu,
        };

        vcpu.reset()?;

        Ok(vcpu)
    }

    pub fn allocate_physical_memory(
        &mut self,
        guest_address: u64,
        size: usize,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        let mapping = MmapOptions::new(size)
            .map_mut()?;

        unsafe {
            self.map_physical_memory(
                guest_address,
                mapping,
                protection,
            )
        }?;

        Ok(())
    }

    pub unsafe fn map_physical_memory(
        &mut self,
        guest_address: u64,
        mapping: MmapMut,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        let mut flags = 0;

        if protection.contains(ProtectionFlags::READ) {
            flags |= HV_MEMORY_READ;
        }

        if protection.contains(ProtectionFlags::WRITE) {
            flags |= HV_MEMORY_WRITE;
        }

        if protection.contains(ProtectionFlags::EXECUTE) {
            flags |= HV_MEMORY_EXEC;
        }

        hv_vm_map(
            mapping.as_ptr() as *const std::ffi::c_void,
            guest_address,
            mapping.len(),
            flags,
        ).into_result()?;

        let range = guest_address..guest_address + mapping.len() as u64;
        let segment = Segment {
            mapping,
        };

        self.physical_ranges.insert(range.clone(), range.start);
        self.segments.insert(range.start, segment);

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

        unsafe {
            hv_vm_unmap(range.start, (range.end - range.start) as usize)
        }.into_result()?;

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

        let mut flags = 0;

        if protection.contains(ProtectionFlags::READ) {
            flags |= HV_MEMORY_READ;
        }

        if protection.contains(ProtectionFlags::WRITE) {
            flags |= HV_MEMORY_WRITE;
        }

        if protection.contains(ProtectionFlags::EXECUTE) {
            flags |= HV_MEMORY_EXEC;
        }

        unsafe {
            hv_vm_protect(range.start, (range.end - range.start) as usize, flags)
        }.into_result()?;

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

        bytes[..size].copy_from_slice(&segment.mapping[offset..offset + size]);

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

        segment.mapping[offset..offset + size].copy_from_slice(&bytes[..size]);

        Ok(size)
    }

}

impl Drop for Vm {
    fn drop(&mut self) {
        unsafe {
            hv_vm_destroy();
        }
    }
}
