use crate::error::Error;
use super::bindings::*;
use super::vm::VmBuilder;

pub struct Hypervisor;

impl Hypervisor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn build_vm(&self) -> Result<VmBuilder, Error> {
        unsafe {
            hv_vm_create(HV_VM_DEFAULT)
        }.into_result()?;

        Ok(VmBuilder)
    }
}
