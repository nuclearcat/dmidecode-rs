use assert_cmd::prelude::*;
use std::process::Command;

// Synthetic Type 4 records, not reconstructed full hardware dumps.
fn decode(version: &str, family: u8, extended: u16, eax: u32) -> String {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cpu.bin");
    let mut table = vec![0; 48];
    table[0..4].copy_from_slice(&[4, 48, 0x4A, 0]);
    table[5] = 3;
    table[6] = family;
    table[8..12].copy_from_slice(&eax.to_le_bytes());
    table[12..16].copy_from_slice(&0xBFEBFBFFu32.to_le_bytes());
    table[16] = 1;
    table[40..42].copy_from_slice(&extended.to_le_bytes());
    table.extend_from_slice(version.as_bytes());
    table.extend_from_slice(&[0, 0, 127, 4, 0xFF, 0xFF, 0, 0]);
    std::fs::write(&path, table).unwrap();
    let output = Command::cargo_bin("dmidecode-rs")
        .unwrap()
        .arg("--from-dump")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

#[test]
fn other_n150_decodes_signature_and_flags() {
    let output = decode("Intel(R) N150", 1, 0, 0x000B06E0);
    assert!(output.contains("Family: Other"));
    assert!(output.contains("Signature: Type 0, Family 6, Model 190, Stepping 0"));
    assert!(output.contains("FPU (Floating-point unit on-chip)"));
}

#[test]
fn unknown_modern_and_legacy_brand_suffixes() {
    for version in [
        " Intel(R) N100 ",
        "Intel(R) Core(TM) i5 CPU",
        "Intel(R) Pentium(R) CPU G4560",
    ] {
        assert!(decode(version, 2, 0, 0x000B06E0).contains("Signature: Type 0"));
    }
    for version in ["AMD Ryzen 9 7950X", "AMD EPYC 9654", "AMD Athlon(TM) 64"] {
        assert!(decode(version, 2, 0, 0x00A60F12)
            .contains("Signature: Family 25, Model 97, Stepping 2"));
    }
}

#[test]
fn ambiguous_and_unrelated_descriptions_remain_undecoded() {
    for version in [
        "Intel",
        "Intel(R) N1500",
        "AMD Ryzenish",
        "pc-i440fx-2.11",
        "ARM Cortex-A72",
        "Default string",
    ] {
        let output = decode(version, 1, 0, 0x000B06E0);
        assert!(!output.contains("Signature:"), "{}", version);
        assert!(!output.contains("\tFlags:"), "{}", version);
    }
    // An explicit non-x86 family must take precedence over a version string.
    assert!(!decode("Intel(R) N150", 0xC8, 0, 0x000B06E0).contains("Signature:"));
}

#[test]
fn extended_family_is_used_only_with_escape_value() {
    assert!(decode("Intel(R) N150", 1, 0xC8, 0x000B06E0).contains("Signature: Type 0"));
    assert!(!decode("Intel(R) N150", 0xFE, 0xC8, 0x000B06E0).contains("Signature:"));
    assert!(decode("Unrecognized version", 0xFE, 0xB3, 0x000B06E0).contains("Signature: Type 0"));
}

#[test]
fn every_database_rule_is_valid_and_reachable() {
    let mut seen = std::collections::HashSet::new();
    for line in include_str!("../data/processor-id-fallbacks.tsv").lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (encoding, version) = line.split_once('\t').unwrap();
        assert!(!version.is_empty() && seen.insert(version));
        let expected = match encoding {
            "intel" => "Signature: Type 0, Family 6, Model 190, Stepping 0",
            "amd" => "Signature: Family 6, Model 14, Stepping 0",
            _ => panic!("Invalid encoding: {}", encoding),
        };
        assert!(
            decode(version, 1, 0, 0x000B06E0).contains(expected),
            "{}",
            version
        );
    }
}
