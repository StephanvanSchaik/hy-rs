fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=Hypervisor");

    #[cfg(target_os = "windows")]
    windows::build! {
        Windows::Win32::System::Hypervisor::*,
    }
}
