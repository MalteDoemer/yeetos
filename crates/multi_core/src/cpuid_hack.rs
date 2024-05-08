#[cfg(any(
    all(target_arch = "x86", not(target_env = "sgx")),
    all(target_arch = "x86_64", not(target_env = "sgx"))
))]
pub mod native_cpuid {
    #[cfg(all(target_arch = "x86", not(target_env = "sgx")))]
    use core::arch::x86 as arch;
    #[cfg(all(target_arch = "x86_64", not(target_env = "sgx")))]
    use core::arch::x86_64 as arch;
    use x86::cpuid::CpuIdResult;

    pub fn cpuid_count(a: u32, c: u32) -> CpuIdResult {
        // Safety: we checked during boot if cpuid is available
        let result = unsafe { self::arch::__cpuid_count(a, c) };

        CpuIdResult {
            eax: result.eax,
            ebx: result.ebx,
            ecx: result.ecx,
            edx: result.edx,
        }
    }
}
