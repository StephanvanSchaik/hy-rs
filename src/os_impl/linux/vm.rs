use crate::error::Error;
use crate::vm::ProtectionFlags;
use kvm_bindings::{KVM_MEM_READONLY, kvm_userspace_memory_region};
use kvm_ioctls::VmFd;
use mmap_rs::{MmapMut, MmapOptions};
use rangemap::RangeMap;
use std::collections::HashMap;
use super::vcpu::Vcpu;

pub struct VmBuilder {
    pub(crate) vm: VmFd,
}

impl VmBuilder {
    pub fn with_vcpu_count(self, _count: usize) -> Result<Self, Error> {
        Ok(self)
    }

    pub fn build(self, _name: &str) -> Result<Vm, Error> {
        self.vm.set_tss_address(0xfffb_d000)?;

        Ok(Vm {
            vm: self.vm,
            segments: HashMap::new(),
            physical_ranges: RangeMap::new(),
            available_slots: vec![],
        })
    }
}

pub struct Segment {
    mapping: MmapMut,
    region: kvm_userspace_memory_region,
}

pub struct Vm {
    pub(crate) vm: VmFd,
    pub(crate) segments: HashMap<u64, Segment>,
    pub(crate) physical_ranges: RangeMap<u64, u64>,
    pub(crate) available_slots: Vec<u32>,
}

impl Vm {
    pub fn create_vcpu(&mut self, id: usize) -> Result<Vcpu, Error> {
        let vcpu = self.vm.create_vcpu(id as u64)?;

        Ok(Vcpu {
            vcpu,
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
        mapping: MmapMut,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        let mut flags = 0;

        if !protection.contains(ProtectionFlags::WRITE) {
            flags |= KVM_MEM_READONLY;
        }

        let slot = match self.available_slots.pop() {
            Some(slot) => slot,
            _ => self.segments.len() as u32,
        };

        let userspace_addr = mapping.as_ptr()
            as *const std::ffi::c_void
            as usize
            as u64;
        let memory_size = mapping.len() as u64;
        let segment = Segment {
            mapping,
            region: kvm_userspace_memory_region {
                slot,
                guest_phys_addr: guest_address,
                userspace_addr,
                memory_size,
                flags,
            },
        };

        unsafe {
            self.vm.set_user_memory_region(segment.region)
        }?;

        self.segments.insert(guest_address, segment);
        self.physical_ranges.insert(guest_address..guest_address + memory_size, guest_address);

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

        // Look up the segment and clone the region.
        let mut region = match self.segments.get(&range.start) {
            Some(segment) => segment.region.clone(),
            _ => return Err(Error::InvalidGuestAddress),
        };

        // Unmap the guest physical memory from the VM.
        region.memory_size = 0;
        let slot = region.slot;

        unsafe {
            self.vm.set_user_memory_region(region)
        }?;

        // Remove the physical address range and segment.
        self.segments.remove(&range.start);
        self.physical_ranges.remove(range);

        // Mark the slot as available again.
        self.available_slots.push(slot);

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

        // Look up the segment.
        let segment = match self.segments.get_mut(&range.start) {
            Some(segment) => segment,
            _ => return Err(Error::InvalidGuestAddress),
        };

        if protection.contains(ProtectionFlags::WRITE) {
            segment.region.flags &= !KVM_MEM_READONLY;
        } else {
            segment.region.flags |= KVM_MEM_READONLY;
        }

        unsafe {
            self.vm.set_user_memory_region(segment.region)
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
