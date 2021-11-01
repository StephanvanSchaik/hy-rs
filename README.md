# Introduction

The hy-rs crate, pronounced as high rise, provides a unified and portable interface to the hypervisor APIs provided by various platforms.
More specifically, this crate provides a portable interface to create and configure virtual machines, map in guest physical memory and set up the virtual CPUs and set up their registers.

## Hardware virtualization

To understand where this crate fits in the virtualization stack, we will have a look at an overview of what a typical virtualization stack looks like.

At the bottom of our virtualization stack, we have actual hardware support for virtualization.
The idea behind virtualization is that we can create environments that are separate from our actual operating system and userspace with its own guest physical memory, virtual CPUs, etc. to essentially virtualize a machine.

 * On AMD CPUs this is known as AMD-V (AMD Virtualization) or AMD SVM (Secure Virtual Machine).
 * On Intel CPUs this is known as Intel VT-x or Intel VMX (Virtual Machine Extension).
 * On ARMv7-A this is known as the virtualization extensions.
 * On AArch64 this is known as AArch64 Virtualization, which is part of the ISA.

The AMD and Intel model represents virtual machines as separate environments that we can run from our existing operating system. More specifically, the virtualization extensions provide us with a way to set up those environments and then transition into them and out of them. These transitions are known as `vmenter` and `vmexit`.

The AArch64 architecture introduces another privilege level called `EL2` for the hypervisor to run, and it can then use `EL1` and `EL0` to run any of the guest operating systems and its userspace, or the host operating system.
One of the problems of this model is that to access the hypervisor, the host operating system has to switch from `EL1` to `EL2` and back at times, which is rather inefficient.
Therefore, AArch64 provides the Virtualization Host Extensions (VHE) which allows the host operating system to run at `EL2` while the host's userspace can remain at `EL0` by directly escalating to `EL2` rather than `EL1`.
In this model only the guest operating systems run at `EL1`.

## Hypervisors

A hypervisor is responsible for setting up and managing these virtualized environments called virtual machines, and may rely on these hardware extensions for better performance.
Hypervisors are further categorized into type 1 and type 2 hypervisors:

 * A type 1 hypervisor runs directly on the bare metal hardware to facilitate direct communication between the virtual machines and the hardware.
 * A type 2 hypervisor runs as part of an existing host operating system and may rely on facilities provided by the operating system to simplify virtualization.

The hy-rs crate specifically focuses on type 2 hypervisors, as it abstracts the hypervisor APIs provided by existing operating systems.
The operating system either has a driver to set up and access the hardware virtualization extensions from an already running host OS, or it first bootstraps the hypervisor either as part of the host operating system, or it runs the host operating system as a privileged guest OS.

## Hypervisor APIs

On most conventional operating systems, the host operating system provides an API to its userspace to set up and run their own virtual machines, and hy-rs provides a unified and portable interface on top of the different APIs by the various operating systems.
More specifically, hy-rs supports the following operating systems:

 * Microsoft Windows through the [WinHV API](https://docs.microsoft.com/en-us/virtualization/api/hypervisor-platform/hypervisor-platform) or Hyper-V.
 * Linux through the [KVM API](https://github.com/rust-vmm/kvm-ioctls).
 * Mac OS X through [Apple's Hypervisor Framework](https://developer.apple.com/documentation/hypervisor/).
 * FreeBSD through their [VMM driver](https://www.freebsd.org/cgi/man.cgi?query=vmm&sektion=4&apropos=0&manpath=FreeBSD+13.0-RELEASE+and+Ports).

## Microsoft Windows

Ensure that Intel VT-x or AMD-V is enabled in your BIOS/UEFI.

The WinHV API is available since Microsoft Windows 10 Version 1803 (April 2018 Update) with kernel build number 17134.
In addition, for this API to be available you will need Microsoft Windows Enterprise, Pro or Education.
Microsoft Windows Home is **not** supported.

For the WinHV API to be functional, you will need to enable the Hyper-V feature.
Open a PowerShell as an administrator and run the following command:

```
Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All
```

Once the installation has completed, reboot your system.

## Linux

Ensure that Intel VT-x or AMD-V is enabled in your BIOS/UEFI.

You can check that your CPU supports Intel VT-x or AMD-V and that the features have been enabled by running the following command:

```
egrep '(vmx|svm)' /proc/cpuinfo
```

For KVM to work, you will need either the `kvm_amd` or the `kvm_intel` module loaded (depending on whether you have an AMD or an Intel CPU).
Run the following command in a terminal:

```
lsmod | grep kvm
```

If the above shows `kvm_amd` or `kvm_intel`, then KVM should be working.
Otherwise, you may need to install the kvm modules for your distribution and load them using `modprobe`, e.g.:

```
sudo modprobe kvm_amd
```

Finally, if kvm is fully functional you should have a file named `/dev/kvm`.
To ensure that your user has access to KVM, you can run the following command:

```
sudo usermod -aG kvm $(whoami)
```

Make sure to reopen the terminal/relogin after executing this command.

## Mac OS X

Mac OS X supports the Hypervisor Framework since version 10.10 (Yosemite).
To see if Hypervisor Framework support is enabled, open a terminal and run the following command:

```
sysctl kern.hv_support
```

If the above command outputs 1, then the Hypervisor Framework is enabled and this crate should work.

If the Hypervisor Framework is not enabled, then check if you have the required CPU features.
In a terminal, run the following command:

```
sysctl -a | grep machdep.cpu.features
```

The above should display the `VMX` flag to indicate support for Intel VMX.
If the flag is missing, and you are running Mac OS X in a VM, then make sure that you have support for nested virtualization and that it is turned on.

**Note**: AMD SVM is not supported.
If you are running Mac OS X on an AMD CPU either bare metal or in a VM, then the Hypervisor Framework will not work.
This is because the Hypervisor Framework only supports Intel VMX on Intel-based Macs and offers a rather low-level abstraction to Intel VMX itself, which makes it hard to port to AMD SVM.

# FreeBSD

FreeBSD supports the VMM driver since FreeBSD 10.0.

Ensure that Intel VT-x or AMD-V is enabled in your BIOS/UEFI.

You can check that your CPU supports Intel VT-x or AMD-V and that the features have been enabled by running the following command:

```
egrep '(VT-x|SVM)' /var/run/dmesg.boot
```

To load the `vmm` driver manually, run the following command:

```
kldload vmm
```

Alternatively, you can add the following to `/boot/loader.conf` to load the `vmm` driver automatically at boot:

```
vmm_load="YES"
```
