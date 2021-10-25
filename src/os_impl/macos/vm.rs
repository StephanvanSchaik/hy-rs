use crate::error::Error;
use crate::vm::ProtectionFlags;
use rangemap::RangeSet;
use std::sync::{Arc, RwLock};
use super::bindings::*;
use super::vcpu::Vcpu;

pub struct VmBuilder;

impl VmBuilder {
    pub fn with_vcpu_count(self, _count: usize) -> Result<Self, Error> {
        Ok(self)
    }

    pub fn build(self) -> Result<Vm, Error> {
        Ok(Vm {
            regions: Arc::new(RwLock::new(RangeSet::new())),
        })
    }
}

pub struct Vm {
    regions: Arc<RwLock<RangeSet<u64>>>,
}

impl Vm {
    pub fn create_vcpu(&mut self, _id: usize) -> Result<Vcpu, Error> {
        let mut vcpu = 0;

        unsafe {
            hv_vcpu_create(&mut vcpu, HV_VCPU_DEFAULT)
        }.into_result()?;

        let mut vcpu = Vcpu {
            vcpu,
            regions: self.regions.clone(),
        };

        vcpu.reset()?;

        Ok(vcpu)
    }

    pub unsafe fn map_physical_memory(
        &mut self,
        guest_address: u64,
        bytes: *mut std::ffi::c_void,
        size: usize,
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
            bytes as *const std::ffi::c_void,
            guest_address,
            size,
            flags,
        ).into_result()?;

        self.regions.write().unwrap().insert(guest_address..guest_address + size as u64);

        Ok(())
    }

    pub fn unmap_physical_memory(
        &mut self,
        guest_address: u64,
    ) -> Result<(), Error> {
        let range = match self.regions.read().unwrap().get(&guest_address) {
            Some(range) => range.clone(),
            _ => return Err(Error::InvalidGuestAddress),
        };

        let mut regions = self.regions.write().unwrap();

        regions.remove(range.clone());

        unsafe {
            hv_vm_unmap(range.start, (range.end - range.start) as usize)
        }.into_result()?;

        Ok(())
    }

    pub fn protect_physical_memory(
        &mut self,
        guest_address: u64,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        let regions = self.regions.write().unwrap();

        let range = match regions.get(&guest_address) {
            Some(range) => range.clone(),
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
}

impl Drop for Vm {
    fn drop(&mut self) {
        unsafe {
            hv_vm_destroy();
        }
    }
}
