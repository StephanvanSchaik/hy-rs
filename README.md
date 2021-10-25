# Introduction

The hy-rs crate, pronounced as high rise, provides a unified and portable interface to the hypervisor APIs provided by various platforms.
More specifically, this crate provides a portable interface to create and configure VMs.

This crate supports the following platforms:
 * Microsoft Windows through the [WinHV API](https://docs.microsoft.com/en-us/virtualization/api/hypervisor-platform/hypervisor-platform) or Hyper-V.
 * Linux through the [KVM API](https://github.com/rust-vmm/kvm-ioctls).
 * Mac OS X through [Apple's Hypervisor Framework](https://developer.apple.com/documentation/hypervisor/).
