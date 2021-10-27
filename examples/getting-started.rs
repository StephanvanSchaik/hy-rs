use hy_rs::{Hypervisor, ProtectionFlags};
use mmap_rs::MmapOptions;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Hypervisor(#[from] hy_rs::Error),
    #[error(transparent)]
    Mmap(#[from] mmap_rs::error::Error),
}

fn main() -> Result<(), Error> {
    // Access the hypervisor API native to this system.
    let hypervisor = Hypervisor::new()?;

    // Build a VM with support for one vCPU.
    let mut vm = hypervisor
        .build_vm()?
        .with_vcpu_count(1)?
        .build()?;

    // Create the vCPU.
    let mut vcpu = vm.create_vcpu(0)?;

    // Allocate a single page.
    let mut mapping = MmapOptions::new()
        .with_size(4096)
        .map_mut()?;

    // Our instruction pointer will point to 0xfff0 by default. Therefore, we write the `hlt`
    // (0xf4) instruction 0xff0 within our mapping.
    mapping[0xff0] = 0xf4;

    // Since the base address of the code segment points to 0xffff_0000 and the RIP points to
    // 0xfff0. We have to map in the 4 kiB page into the guest VM at the guest physical address
    // 0xffff_f000, such that `cs:ip` points to the `hlt` instruction.
    unsafe {
        vm.map_physical_memory(
            0xffff_f000,
            mapping.as_mut_ptr() as *mut _,
            mapping.size(),
            ProtectionFlags::all(),
        )?;
    }

    // Run the vCPU. Note that this consumes the thread until the vCPU exits. If you are planning
    // to run more than one vCPU, then you will need to spawn a thread for each vCPU.
    let exit_reason = vcpu.run()?;

    // This should print that the vCPU halted.
    println!("Exit Reason: {:?}", exit_reason);

    Ok(())
}