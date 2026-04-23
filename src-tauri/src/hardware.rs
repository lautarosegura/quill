//! Hardware detection for model recommendation during the wizard + Modelos page.
//! Small and deliberately simple — no GPU introspection beyond "is Apple Silicon".

use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize)]
pub struct HardwareProfile {
    pub os: String,
    pub arch: String,
    pub apple_silicon: bool,
    pub ram_gb: u32,
    pub cpu_cores: u32,
    pub recommended_model: String,
    pub recommended_rationale: String,
}

pub fn detect() -> HardwareProfile {
    let mut sys = System::new();
    sys.refresh_memory();

    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let apple_silicon = os == "macos" && arch == "aarch64";

    // total_memory() returns bytes in sysinfo 0.32+.
    let ram_bytes = sys.total_memory();
    let ram_gb = (ram_bytes / 1_000_000_000) as u32;

    let cpu_cores = sys
        .physical_core_count()
        .map(|n| n as u32)
        .unwrap_or_else(num_cpus_fallback);

    let (recommended_model, recommended_rationale) =
        recommend(&arch, apple_silicon, ram_gb, cpu_cores);

    HardwareProfile {
        os,
        arch,
        apple_silicon,
        ram_gb,
        cpu_cores,
        recommended_model,
        recommended_rationale,
    }
}

fn recommend(_arch: &str, apple_silicon: bool, ram_gb: u32, cpu_cores: u32) -> (String, String) {
    if apple_silicon && ram_gb >= 8 {
        return (
            "ggml-small".into(),
            "Metal GPU puede con small sin problemas en Apple Silicon.".into(),
        );
    }
    if ram_gb >= 16 && cpu_cores >= 8 {
        return (
            "ggml-base".into(),
            "CPU rápido pero sin GPU — base es el sweet spot.".into(),
        );
    }
    (
        "ggml-tiny".into(),
        "Hardware limitado — tiny mantiene latencia razonable.".into(),
    )
}

/// Backstop if sysinfo can't enumerate physical cores (rare).
fn num_cpus_fallback() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommend_small_on_apple_silicon_with_enough_ram() {
        let (model, _) = recommend("aarch64", true, 16, 8);
        assert_eq!(model, "ggml-small");
    }

    #[test]
    fn recommend_base_on_strong_intel() {
        let (model, _) = recommend("x86_64", false, 32, 12);
        assert_eq!(model, "ggml-base");
    }

    #[test]
    fn recommend_tiny_on_limited_hardware() {
        let (model, _) = recommend("x86_64", false, 8, 4);
        assert_eq!(model, "ggml-tiny");
    }

    #[test]
    fn apple_silicon_with_low_ram_falls_through() {
        // 4GB Mac is unusual but should still downgrade gracefully.
        let (model, _) = recommend("aarch64", true, 4, 4);
        assert_eq!(model, "ggml-tiny");
    }

    #[test]
    fn detect_returns_populated_profile() {
        let p = detect();
        assert!(!p.os.is_empty());
        assert!(!p.arch.is_empty());
        assert!(p.cpu_cores >= 1);
        assert!(!p.recommended_model.is_empty());
    }
}
