//! This module provides [`Mmap`] and [`MmapMut`] to abstract memory mappings of the guest physical
//! address space of the VM.

use crate::error::Error;
use crate::vm::Vm;
use std::ops::{Deref, DerefMut};

macro_rules! mmap_impl {
    ($t:ident) => {
        impl $t {
            /// Yields a raw immutable pointer of this mapping.
            #[inline]
            pub fn as_ptr(&self) -> *const u8 {
                self.inner
                    .as_ref()
                    .expect("inner must have been present")
                    .as_ptr()
            }

            /// Yields the size of this mapping.
            #[inline]
            pub fn size(&self) -> usize {
                self.inner
                    .as_ref()
                    .expect("inner must have been present")
                    .size()
            }

            /// Locks the physical pages in memory such that accessing the mapping causes no page faults.
            pub fn lock(&mut self) -> Result<(), Error> {
                self.inner
                    .as_mut()
                    .expect("inner must have been present")
                    .lock()?;

                Ok(())
            }

            /// Unlocks the physical pages in memory, allowing the operating system to swap out the pages
            /// backing this memory mapping.
            pub fn unlock(&mut self) -> Result<(), Error> {
                self.inner
                    .as_mut()
                    .expect("inner must have been present")
                    .unlock()?;

                Ok(())
            }

            /// This function can be used to flush the instruction cache on architectures where
            /// this is required.
            ///
            /// While the x86 and x86-64 architectures guarantee cache coherency between the L1 instruction
            /// and the L1 data cache, other architectures such as Arm and AArch64 do not. If the user
            /// modified the pages, then executing the code may result in undefined behavior. To ensure
            /// correct behavior a user has to flush the instruction cache after modifying and before
            /// executing the page.
            pub fn flush_icache(&self) -> Result<(), Error> {
                self.inner
                    .as_ref()
                    .expect("inner must have been present")
                    .flush_icache()?;

                Ok(())
            }

            /// Remaps this memory mapping as inaccessible.
            ///
            /// In case of failure, this returns the ownership of `self`.
            pub fn make_none(mut self) -> Result<MmapNone, (Self, Error)> {
                let inner = self.inner
                    .take()
                    .expect("inner must have been present");

                let inner = match inner.make_none() {
                    Ok(inner) => inner,
                    Err((inner, e)) => {
                        let mmap = Self {
                            vm: self.vm.take(),
                            inner: Some(inner),
                            guest_address: self.guest_address,
                        };

                        return Err((mmap, e.into()));
                    }
                };

                Ok(MmapNone {
                    vm: self.vm.take(),
                    inner: Some(inner),
                    guest_address: self.guest_address,
                })
            }

            /// Remaps this memory mapping as immutable.
            ///
            /// In case of failure, this returns the ownership of `self`.
            pub fn make_read_only(mut self) -> Result<Mmap, (Self, Error)> {
                let inner = self.inner
                    .take()
                    .expect("inner must have been present");

                let inner = match inner.make_read_only() {
                    Ok(inner) => inner,
                    Err((inner, e)) => {
                        let mmap = Self {
                            vm: self.vm.take(),
                            inner: Some(inner),
                            guest_address: self.guest_address,
                        };

                        return Err((mmap, e.into()));
                    }
                };

                Ok(Mmap {
                    vm: self.vm.take(),
                    inner: Some(inner),
                    guest_address: self.guest_address,
                })
            }

            /// Remaps this memory mapping as executable.
            ///
            /// In case of failure, this returns the ownership of `self`.
            pub fn make_exec(mut self) -> Result<Mmap, (Self, Error)> {
                let inner = self.inner
                    .take()
                    .expect("inner must have been present");

                let inner = match inner.make_exec() {
                    Ok(inner) => inner,
                    Err((inner, e)) => {
                        let mmap = Self {
                            vm: self.vm.take(),
                            inner: Some(inner),
                            guest_address: self.guest_address,
                        };

                        return Err((mmap, e.into()));
                    }
                };

                Ok(Mmap {
                    vm: self.vm.take(),
                    inner: Some(inner),
                    guest_address: self.guest_address,
                })
            }

            /// Remaps this memory mapping as executable, but does not flush the instruction cache.
            /// Note that this is **unsafe**.
            ///
            /// While the x86 and x86-64 architectures guarantee cache coherency between the L1 instruction
            /// and the L1 data cache, other architectures such as Arm and AArch64 do not. If the user
            /// modified the pages, then executing the code may result in undefined behavior. To ensure
            /// correct behavior a user has to flush the instruction cache after modifying and before
            /// executing the page.
            ///
            /// In case of failure, this returns the ownership of `self`.
            pub unsafe fn make_exec_no_flush(mut self) -> Result<Mmap, (Self, Error)> {
                let inner = self.inner
                    .take()
                    .expect("inner must have been present");

                let inner = match inner.make_exec_no_flush() {
                    Ok(inner) => inner,
                    Err((inner, e)) => {
                        let mmap = Self {
                            vm: self.vm.take(),
                            inner: Some(inner),
                            guest_address: self.guest_address,
                        };

                        return Err((mmap, e.into()));
                    }
                };

                Ok(Mmap {
                    vm: self.vm.take(),
                    inner: Some(inner),
                    guest_address: self.guest_address,
                })
            }

            /// Remaps this mapping to be mutable.
            ///
            /// In case of failure, this returns the ownership of `self`.
            pub fn make_mut(mut self) -> Result<MmapMut, (Self, Error)> {
                let inner = self.inner
                    .take()
                    .expect("inner must have been present");

                let inner = match inner.make_mut() {
                    Ok(inner) => inner,
                    Err((inner, e)) => {
                        let mmap = Self {
                            vm: self.vm.take(),
                            inner: Some(inner),
                            guest_address: self.guest_address,
                        };

                        return Err((mmap, e.into()));
                    }
                };

                Ok(MmapMut {
                    vm: self.vm.take(),
                    inner: Some(inner),
                    guest_address: self.guest_address,
                })
            }

            /// Remaps this mapping to be executable and mutable.
            ///
            /// While this may seem useful for self-modifying
            /// code and JIT engines, it is instead recommended to convert between mutable and executable
            /// mappings using [`Mmap::make_mut()`] and [`MmapMut::make_exec()`] instead.
            ///
            /// As it may be tempting to use this function, this function has been marked as **unsafe**.
            /// Make sure to read the text below to understand the complications of this function before
            /// using it. The [`UnsafeMmapFlags::JIT`] flag must be set for this function to succeed.
            ///
            /// RWX pages are an interesting targets to attackers, e.g. for buffer overflow attacks, as RWX
            /// mappings can potentially simplify such attacks. Without RWX mappings, attackers instead
            /// have to resort to return-oriented programming (ROP) gadgets. To prevent buffer overflow
            /// attacks, contemporary CPUs allow pages to be marked as non-executable which is then used by
            /// the operating system to ensure that pages are either marked as writeable or as executable,
            /// but not both. This is also known as W^X.
            ///
            /// While the x86 and x86-64 architectures guarantee cache coherency between the L1 instruction
            /// and the L1 data cache, other architectures such as Arm and AArch64 do not. If the user
            /// modified the pages, then executing the code may result in undefined behavior. To ensure
            /// correct behavior a user has to flush the instruction cache after modifying and before
            /// executing the page.
            ///
            /// In case of failure, this returns the ownership of `self`.
            pub unsafe fn make_exec_mut(mut self) -> Result<MmapMut, (Self, Error)> {
                let inner = self.inner
                    .take()
                    .expect("inner must have been present");

                let inner = match inner.make_exec_mut() {
                    Ok(inner) => inner,
                    Err((inner, e)) => {
                        let mmap = Self {
                            vm: self.vm.take(),
                            inner: Some(inner),
                            guest_address: self.guest_address,
                        };

                        return Err((mmap, e.into()));
                    }
                };

                Ok(MmapMut {
                    vm: self.vm.take(),
                    inner: Some(inner),
                    guest_address: self.guest_address,
                })
            }
        }

        impl Drop for $t {
            fn drop(&mut self) {
                if let Some(vm) = &mut self.vm {
                    let _ = vm.unmap_physical_memory(
                        self.guest_address,
                    );
                }
            }
        }
    }
}

/// Represents an inaccessible memory mapping to guest physical memory.
pub struct MmapNone {
    vm: Option<Vm>,
    inner: Option<mmap_rs:: MmapNone>,
    guest_address: u64,
}

mmap_impl!(MmapNone);

/// Represents an immutable memory mapping to guest physical memory.
pub struct Mmap {
    vm: Option<Vm>,
    inner: Option<mmap_rs::Mmap>,
    guest_address: u64,
}

mmap_impl!(Mmap);

impl Deref for Mmap {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.as_ptr(), self.size())
        }
    }
}

impl AsRef<[u8]> for Mmap {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.as_ptr(), self.size())
        }
    }
}

/// Represents a mutable memory mapping to guest physical memory.
pub struct MmapMut {
    vm: Option<Vm>,
    inner: Option<mmap_rs::MmapMut>,
    guest_address: u64,
}

mmap_impl!(MmapMut);

impl MmapMut {
    /// Yields a raw mutable pointer to this mapping.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.inner
            .as_mut()
            .expect("inner must have been present")
            .as_mut_ptr()
    }
}

impl Deref for MmapMut {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.as_ptr(), self.size())
        }
    }
}

impl DerefMut for MmapMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size())
        }
    }
}

impl AsRef<[u8]> for MmapMut {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.as_ptr(), self.size())
        }
    }
}

impl AsMut<[u8]> for MmapMut {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size())
        }
    }
}
