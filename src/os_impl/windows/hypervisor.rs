use crate::error::Error;
use super::bindings::*;
use super::vm::{PartitionHandle, VmBuilder};

pub struct Hypervisor;

impl Hypervisor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self)
    }

    pub fn build_vm(&self) -> Result<VmBuilder, Error> {
        let handle = unsafe {
            WHvCreatePartition()
        }?;

        Ok(VmBuilder {
            handle: PartitionHandle(handle),
        })
    }
}
