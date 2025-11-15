fn main() {
    // Tell rustc these cfg keys are intentional so it doesn't warn.
    println!("cargo:rustc-check-cfg=cfg(has_neon)");
    println!("cargo:rustc-check-cfg=cfg(has_avx_sse)");
    
    #[cfg(target_arch = "aarch64")]
    {
        let mut cc = cc::Build::new();
        cc.file("../../asm/arm/neon_mix.S")
          .file("../../asm/arm/neon_sine.S")
          .flag_if_supported("-fno-asynchronous-unwind-tables")
          .flag_if_supported("-fno-exceptions")
          .compile("ambientor_neon");
        println!("cargo:rustc-cfg=has_neon");
    }

    #[cfg(target_arch = "x86_64")]
    {
        let mut cc = cc::Build::new();
        cc.file("../../asm/x86/avx_mix.S")
          .file("../../asm/x86/sse_sine.S")
          .flag_if_supported("-mavx")
          .flag_if_supported("-msse4.1")
          .flag_if_supported("-fno-asynchronous-unwind-tables")
          .flag_if_supported("-fno-exceptions")
          .compile("ambientor_x86");
        println!("cargo:rustc-cfg=has_avx_sse");
    }
}
