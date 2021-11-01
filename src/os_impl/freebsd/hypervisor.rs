use crate::error::Error;
use super::vm::VmBuilder;

pub struct Hypervisor;

impl Hypervisor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn build_vm(&self) -> Result<VmBuilder, Error> {
        Ok(VmBuilder)
    }
}
