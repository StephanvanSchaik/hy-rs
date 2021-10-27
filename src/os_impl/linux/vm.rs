use crate::error::Error;
use crate::vm::ProtectionFlags;
use kvm_bindings::{KVM_MEM_READONLY, kvm_userspace_memory_region};
use kvm_ioctls::VmFd;
use std::collections::HashMap;
use super::vcpu::Vcpu;

pub struct VmBuilder {
    pub(crate) vm: VmFd,
}

impl VmBuilder {
    pub fn with_vcpu_count(self, _count: usize) -> Result<Self, Error> {
        Ok(self)
    }

    pub fn build(self) -> Result<Vm, Error> {
        self.vm.set_tss_address(0xfffb_d000)?;

        Ok(Vm {
            vm: self.vm,
            slots: HashMap::new(),
            available_slots: vec![],
        })
    }
}

pub struct Vm {
    pub(crate) vm: VmFd,
    pub(crate) slots: HashMap<u64, kvm_userspace_memory_region>,
    pub(crate) available_slots: Vec<u32>,
}

impl Vm {
    pub fn create_vcpu(&mut self, id: usize) -> Result<Vcpu, Error> {
        let vcpu = self.vm.create_vcpu(id as u64)?;

        Ok(Vcpu {
            vcpu,
        })
    }

    pub unsafe fn map_physical_memory(
        &mut self,
        guest_address: u64,
        bytes: *mut std::ffi::c_void,
        size: usize,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        let mut flags = 0;

        if !protection.contains(ProtectionFlags::WRITE) {
            flags |= KVM_MEM_READONLY;
        }

        let slot = match self.available_slots.pop() {
            Some(slot) => slot,
            _ => self.slots.len() as u32,
        };

        let mem_region = kvm_userspace_memory_region {
            slot,
            guest_phys_addr: guest_address,
            userspace_addr: bytes as u64,
            memory_size: size as u64,
            flags,
        };

        self.slots.insert(guest_address, mem_region);

        self.vm.set_user_memory_region(mem_region)?;

        Ok(())
    }

    pub fn unmap_physical_memory(
        &mut self,
        guest_address: u64,
    ) -> Result<(), Error> {
        let mut mem_region = match self.slots.remove(&guest_address) {
            Some(mem_region) => mem_region,
            _ => return Err(Error::InvalidGuestAddress),
        };

        mem_region.memory_size = 0;

        unsafe {
            self.vm.set_user_memory_region(mem_region)
        }?;

        Ok(())
    }

    pub fn protect_physical_memory(
        &mut self,
        guest_address: u64,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        let mem_region = match self.slots.get_mut(&guest_address) {
            Some(mem_region) => mem_region,
            _ => return Err(Error::InvalidGuestAddress),
        };

        if protection.contains(ProtectionFlags::WRITE) {
            mem_region.flags &= !KVM_MEM_READONLY;
        } else {
            mem_region.flags |= KVM_MEM_READONLY;
        }

        unsafe {
            self.vm.set_user_memory_region(mem_region.clone())
        }?;

        Ok(())
    }
}
