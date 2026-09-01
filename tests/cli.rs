use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{str, process::Command};
use tempfile::tempdir;

static CLI_COMMAND: &str = "dmidecode-rs";

#[test]
fn test_command_run() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin(CLI_COMMAND)?;
    Ok(())
}

#[test]
fn test_dump_bin_no_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CLI_COMMAND)?;

    cmd.arg("--dump-bin");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("requires a value"));
    Ok(())
}

#[test]
fn test_read_bin_no_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CLI_COMMAND)?;

    cmd.arg("--from-dump");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("requires a value"));
    Ok(())
}

#[test]
fn test_dump_bin_read_bin() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd_dump = Command::cargo_bin(CLI_COMMAND)?;
    let dir = tempdir()?;

    let filename = dir.path().join("raw.bin");
    let filename_str = filename.to_str().unwrap();
    cmd_dump.arg("--dump-bin").arg(&filename_str);
    cmd_dump.assert().success();

    let mut cmd_read = Command::cargo_bin(CLI_COMMAND)?;
    cmd_read.arg("--from-dump").arg(&filename_str);
    cmd_read.assert().success();

    drop(filename);
    dir.close()?;
    Ok(())
}

#[test]
fn test_dmi_str_no_value() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CLI_COMMAND)?;
    cmd.arg("-s");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("requires a value"));
    Ok(())
}

#[test]
fn test_dmi_str_known_unknown_keyword() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("-s").arg("bios-version");
    cmd1.assert().success();

    let mut cmd2 = Command::cargo_bin(CLI_COMMAND)?;
    cmd2.arg("-s").arg("invalid");
    cmd2.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid value"));

    Ok(())
}

#[test]
fn test_oem_string_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("--oem-string").arg("0");
    cmd1.assert()
        .failure()
        .stderr(predicate::str::contains("string number 0"));

    let mut cmd2 = Command::cargo_bin(CLI_COMMAND)?;
    cmd2.arg("--oem-string").arg("foo");
    cmd2.assert()
        .failure()
        .stderr(predicate::str::contains("string number foo"));

    Ok(())
}

#[test]
fn test_oem_string_valid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("--oem-string").arg("count");
    cmd1.assert().success();
    let output = cmd1.output()?;
    let data = str::from_utf8(&output.stdout)?;
    let value = data.trim().parse::<u8>()?;
    // If count is "0", do not run next command.
    if value == 0 {
        return Ok(())
    }
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("--oem-string").arg("1");
    cmd1.assert().success();

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[test]
fn test_no_sysfs() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("--no-sysfs");
    cmd1.assert().success();

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[test]
fn test_dev_mem() -> Result<(), Box<dyn std::error::Error>> {
    // test good path to /dev/mem
    let mut cmd = Command::cargo_bin(CLI_COMMAND)?;
    cmd.arg("--no-sysfs").arg("--dev-mem").arg("/dev/mem");
    cmd.assert().success();

    // test bad path to /dev/memx
    cmd = Command::cargo_bin(CLI_COMMAND)?;
    cmd.arg("--no-sysfs").arg("--dev-mem").arg("/dev/memx");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}

#[test]
fn test_dump_opt() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("-u");
    cmd1.assert().success();

    Ok(())
}

#[test]
fn test_handle_valid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("-H").arg("10");
    cmd1.assert()
        .success()
        .stdout(predicate::str::contains("10"));

    Ok(())
}

#[test]
fn test_handle_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("-H").arg("1000");
    cmd1.assert()
        .failure()
        .stderr(predicate::str::contains("Handle not found: 1000"));

    Ok(())
}

#[test]
fn test_json_valid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd1 = Command::cargo_bin(CLI_COMMAND)?;
    cmd1.arg("-j");
    cmd1.assert()
        .success()
        .stdout(predicate::str::contains("{\"version\":{\"major\":"));

    Ok(())
}

#[test]
fn test_dimm_map_from_dump() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let filename = dir.path().join("memory.bin");
    let mut table = vec![
        17, 28, 0x00, 0x11, 0, 16, 0xFE, 0xFF, 72, 0, 64, 0, 0, 32, 9, 0, 1, 2, 0x1A, 0, 0, 0x68,
        0x10, 3, 4, 0, 5, 2,
    ];
    for value in ["DIMM_A1", "BANK 0", "Example", "SERIAL", "ABC-123"] {
        table.extend_from_slice(value.as_bytes());
        table.push(0);
    }
    table.push(0);
    table.extend([127, 4, 0xFF, 0xFF, 0, 0]);
    std::fs::write(&filename, table)?;

    let mut cmd = Command::cargo_bin(CLI_COMMAND)?;
    cmd.arg("--from-dump").arg(filename).arg("--dimm-map");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("LOCATOR"))
        .stdout(predicate::str::contains("DIMM_A1  BANK 0  8 GB  DDR4"));

    dir.close()?;
    Ok(())
}

#[test]
fn test_dimm_map_conflicts_with_other_output_modes() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CLI_COMMAND)?;
    cmd.arg("--dimm-map").arg("--json");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
    Ok(())
}
