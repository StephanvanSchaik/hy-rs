use crate::error::Error;
use kvm_ioctls::Kvm;
use super::vm::VmBuilder;

pub struct Hypervisor {
    kvm: Kvm,
}

impl Hypervisor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            kvm: Kvm::new()?,
        })
    }

    pub fn build_vm(&self) -> Result<VmBuilder, Error> {
        let vm = self.kvm.create_vm()?;

        Ok(VmBuilder {
            vm,
        })
    }
}
