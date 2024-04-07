use x86::cpuid::CpuId;

pub fn verify() {
    let cpuid = CpuId::new();

    let feature_info = cpuid.get_feature_info();
    let feature_info_ref = feature_info.as_ref();

    let has_sse = feature_info_ref.map_or(false, |info| info.has_sse());
    let has_sysenter_sysexit = feature_info_ref.map_or(false, |info| info.has_sysenter_sysexit());

    assert!(has_sse, "sse not supported");
    assert!(has_sysenter_sysexit, "sysenter/sysexit not supported");
}
