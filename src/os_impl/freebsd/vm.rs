use crate::error::Error;
use crate::vm::ProtectionFlags;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use super::bindings::*;
use super::vcpu::Vcpu;

pub struct VmBuilder;

impl VmBuilder {
    pub fn with_vcpu_count(self, _count: usize) -> Result<Self, Error> {
        Ok(self)
    }

    pub fn build(self) -> Result<Vm, Error> {
        let name = "test";
        vm_create(name)?;

        let mut path = PathBuf::from("/dev/vmm");
        path.push(name);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(&path)?;


        Ok(Vm {
            name: name.to_string(),
            file,
        })
    }
}

pub struct Vm {
    name: String,
    file: File,
}

impl Vm {
    pub fn create_vcpu(&mut self, id: usize) -> Result<Vcpu, Error> {
        Ok(Vcpu {
            cpuid: id as i32,
            file: self.file.try_clone()?,
            rip: 0,
        })
    }

    pub unsafe fn map_physical_memory(
        &mut self,
        guest_address: u64,
        bytes: *mut std::ffi::c_void,
        size: usize,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn unmap_physical_memory(
        &mut self,
        guest_address: u64,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn protect_physical_memory(
        &mut self,
        guest_address: u64,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        let _ = vm_destroy(&self.name);
    }
}
