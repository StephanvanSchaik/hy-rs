//! This module provides the [`Vm`] struct which represents a virtual machine, i.e. a number of
//! virtual CPUs and a physical memory space.

use bitflags::bitflags;
use crate::error::Error;
use crate::platform;
use crate::vcpu::Vcpu;
use intrusive_collections::intrusive_adapter;
use intrusive_collections::{SinglyLinkedListLink, SinglyLinkedList};
use mmap_rs::MmapMut;
use rangemap::RangeMap;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, RwLock};

/// Represents the metadata of a physical page of the guest VM.
pub struct PageInfo {
    /// The link used to add this page to the free list.
    link: SinglyLinkedListLink,
}

intrusive_adapter!(PageInfoAdapter<'a> = &'a PageInfo: PageInfo { link: SinglyLinkedListLink });

/// The page allocator used to manage the physical pages of the guest VM.
pub struct PageAllocator<'a> {
    /// A singly linked list containing the set of free pages.
    free_list: SinglyLinkedList<PageInfoAdapter<'a>>,
    /// A mapping of the page info ranges to the corresponding base guest physical address.
    page_info_ranges: RangeMap<usize, u64>,
    /// A mapping of the physical address ranges to the corresponding base guest physical address.
    physical_ranges: RangeMap<u64, u64>,
    /// The memory segments.
    segments: HashMap<u64, Box<[PageInfo]>>,
}

impl<'a> Drop for PageAllocator<'a> {
    fn drop(&mut self) {
        self.free_list.fast_clear();
    }
}

impl<'a> PageAllocator<'a> {
    /// Sets up the page allocator.
    pub fn new() -> Self {
        Self {
            free_list: SinglyLinkedList::new(PageInfoAdapter::new()),
            page_info_ranges: RangeMap::new(),
            physical_ranges: RangeMap::new(),
            segments: HashMap::new(),
        }
    }

    /// Allocates a physical page.
    pub fn alloc_page(&mut self) -> Option<u64> {
        let page_info = match self.free_list.pop_front() {
            Some(page_info) => page_info,
            _ => return None,
        };

        let offset = page_info
            as *const PageInfo
            as *const std::ffi::c_void
            as usize;

        let (range, guest_address) = self.page_info_ranges
            .get_key_value(&offset)
            .expect("page info range must have been present");

        let index = (offset - range.start) / std::mem::size_of::<PageInfo>();
        let guest_address = *guest_address + (index as u64) * 4096;

        Some(guest_address)
    }

    /// Frees the given physical page.
    pub fn free_page(&mut self, phys_addr: u64) {
        let (range, _) = self.physical_ranges
            .get_key_value(&phys_addr)
            .expect("physical range must have been present");
        let index = ((phys_addr - range.start) / 4096) as usize;

        let segment = self.segments
            .get(&range.start)
            .expect("segment must have been present");

        let page_info = unsafe { &*segment.as_ptr().offset(index as isize) };

        self.free_list.push_front(page_info);
    }

    pub fn add_range(&mut self, range: Range<u64>) -> Result<(), Error> {
        let mut page_infos = vec![];

        for _ in range.clone().step_by(4096) {
            page_infos.push(PageInfo {
                link: SinglyLinkedListLink::new(),
            });
        }

        let page_infos = page_infos.into_boxed_slice();

        for index in 0..page_infos.len() {
            let page_info = unsafe { &*page_infos.as_ptr().offset(index as isize) };
            self.free_list.push_front(page_info);
        }

        let base = page_infos.as_ptr() as *const PageInfo as usize;
        let end  = base + page_infos.len() * std::mem::size_of::<PageInfo>();

        self.page_info_ranges.insert(base..end, range.start);
        self.physical_ranges.insert(range.clone(), range.start);
        self.segments.insert(range.start, page_infos);

        Ok(())
    }
}

bitflags! {
    /// The protection flags used when mapping guest physical memory.
    ///
    /// Not all platforms support the full set of protection flags:
    ///  * Linux does not support the executable bit, which means that guest physical memory is
    ///    always executable.
    ///  * FreeBSD does not support any of the protection flags, which means that guest physical
    ///    memory is always readable, writable and executable.
    pub struct ProtectionFlags: u32 {
        /// The guest VM is allowed to read from the physical memory.
        const READ    = 1 << 0;
        /// The guest VM is allowed to write to the physical memory.
        const WRITE   = 1 << 1;
        /// The guest VM is allowed to execute from the physical memory.
        const EXECUTE = 1 << 2;
    }
}

/// The `VmBuilder` allows for the configuration of certain properties for the new VM before
/// constructing it, as these properties may be immutable once the VM has been built.
pub struct VmBuilder {
    /// The internal platform-specific implementation of the [`platform::VmBuilder`] struct.
    pub(crate) inner: platform::VmBuilder,
}

impl VmBuilder {
    /// This is used to specify the maximum number of virtual CPUs to use for this VM.
    pub fn with_vcpu_count(self, count: usize) -> Result<Self, Error> {
        Ok(Self {
            inner: self.inner.with_vcpu_count(count)?,
        })
    }

    /// Builds the VM and assigns the given name and returns a [`Vm`].
    pub fn build(self, name: &str) -> Result<Vm, Error> {
        Ok(Vm {
            inner: Arc::new(RwLock::new(self.inner.build(name)?)),
            page_allocator: Arc::new(RwLock::new(PageAllocator::new())),
        })
    }
}

/// The `Vm` struct represents a virtual machine. More specifically, it represents an abstraction
/// over a number of virtual CPUs and a physical memory space.
#[derive(Clone)]
pub struct Vm<'a> {
    /// The internal platform-specific implementation of the [`platform::Vm`] struct.
    pub(crate) inner: Arc<RwLock<platform::Vm>>,
    /// The page allocator.
    pub(crate) page_allocator: Arc<RwLock<PageAllocator<'a>>>,
}

impl<'a> Vm<'a> {
    /// Create a virtual CPU with the given vCPU ID.
    pub fn create_vcpu(&mut self, id: usize) -> Result<Vcpu, Error> {
        Ok(Vcpu {
            inner: self.inner.write().unwrap().create_vcpu(id)?,
        })
    }

    /// Allocates guest physical memory into the VM's address space at the given guest address with
    /// the given size. The size must be aligned to the minimal page size. In addition, the
    /// protection of the memory mapping is set to the given protection. This protection affects
    /// how the guest VM can or cannot access the guest physical memory.
    pub fn allocate_physical_memory(
        &mut self,
        guest_address: u64,
        size: usize,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        self.inner
            .write()
            .unwrap()
            .allocate_physical_memory(guest_address, size, protection)?;

        self.page_allocator
            .write()
            .unwrap()
            .add_range(guest_address..guest_address + size as u64)?;

        Ok(())
    }

    /// Maps guest physical memory into the VM's address space. More specifically this function
    /// takes a virtual address as `bytes`, resolves it to the host physical address and maps it to
    /// the specified guest physical address `guest_address` with the specified protection
    /// [`ProtectionFlags`] and the specified `size`, which must be page size aligned.
    ///
    /// This function is not supported on FreeBSD due to underlying differences in the memory
    /// management API provided by FreeBSD. While Microsoft Windows, Linux and Mac OS X allow us to
    /// map in virtual memory, and then map that directly into our guest physical address space,
    /// FreeBSD instead allocates guest physical memory for us and allows us to map that into our
    /// virtual address space.
    pub unsafe fn map_physical_memory(
        &mut self,
        guest_address: u64,
        mapping: MmapMut,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        self.inner
            .write()
            .unwrap()
            .map_physical_memory(guest_address, mapping, protection)
    }

    /// Unmaps the guest physical memory.
    pub fn unmap_physical_memory(
        &mut self,
        guest_address: u64,
    ) -> Result<(), Error> {
        self.inner
            .write()
            .unwrap()
            .unmap_physical_memory(guest_address)
    }

    /// Changes the protection flags of the guest physical memory.
    pub fn protect_physical_memory(
        &mut self,
        guest_address: u64,
        protection: ProtectionFlags,
    ) -> Result<(), Error> {
        self.inner
            .write()
            .unwrap()
            .protect_physical_memory(guest_address, protection)
    }

    /// Reads the bytes starting at the guest address into the given bytes buffer.
    pub fn read_physical_memory(
        &mut self,
        bytes: &mut [u8],
        guest_address: u64,
    ) -> Result<usize, Error> {
        self.inner
            .read()
            .unwrap()
            .read_physical_memory(bytes, guest_address)
    }

    /// Writes the bytes from the given bytes buffer to the bytes starting at guest address.
    pub fn write_physical_memory(
        &mut self,
        guest_address: u64,
        bytes: &[u8],
    ) -> Result<usize, Error> {
        self.inner
            .write()
            .unwrap()
            .write_physical_memory(guest_address, bytes)
    }
}
