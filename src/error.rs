//! This module provides an `Error` type for the crate using the [`thiserror`] crate.
use thiserror::Error;

/// The `Error` type.
#[derive(Debug, Error)]
pub enum Error {
    /// The PTE was not found.
    #[error("PTE not found")]
    PteNotFound,
    /// The page is not present.
    #[error("page not present")]
    PageNotPresent,
    /// Not implemented.
    #[error("not implemented")]
    NotImplemented,
    /// Out of memory.
    #[error("out of memory")]
    OutOfMemory,
    /// The guest address is invalid.
    #[error("invalid guest address")]
    InvalidGuestAddress,
    /// Wraps ['std::io::Error'].
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Wraps ['mmap_rs::Error'].
    #[error(transparent)]
    Mmap(#[from] mmap_rs::error::Error),
    /// Wraps an error that originates from any calls to the ['sysctl'] crate.
    #[cfg(target_os = "freebsd")]
    #[error(transparent)]
    Sysctl(#[from] sysctl::SysctlError),
    /// Wraps an error that originates from any calls to the ['nix'] crate.
    #[cfg(target_os = "freebsd")]
    #[error(transparent)]
    Nix(#[from] nix::Error),
    /// Wraps an error that originates from any calls to the [`kvm_ioctls`] crate.
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    KvmError(#[from] kvm_ioctls::Error),
    #[cfg(target_os = "macos")]
    /// Wraps an error that originates from any calls to Apple's Hypervisor Framework.
    #[error("hv_return_t code: {0}")]
    HypervisorError(crate::os_impl::macos::bindings::hv_return_t),
    /// Wraps an error that originates from any calls to the [`windows`] crate.
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    WindowsError(#[from] windows::Error),
}
