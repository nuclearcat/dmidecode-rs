use crate::dmifn::dmi_memory_device_type;
use smbioslib::{
    MemorySize, MemorySizeExtended, MemorySpeed, MemorySpeedExtended, SMBiosData,
    SMBiosMemoryDevice, SMBiosString,
};
use std::fmt::Write;

const HEADERS: [&str; 7] = [
    "LOCATOR",
    "BANK",
    "SIZE",
    "TYPE",
    "SPEED",
    "MANUFACTURER",
    "PART NUMBER",
];

/// Render a compact, one-row-per-socket summary of SMBIOS Type 17 records.
pub(crate) fn render_dimm_map(data: &SMBiosData) -> String {
    let rows: Vec<[String; 7]> = data
        .defined_struct_iter::<SMBiosMemoryDevice<'_>>()
        .map(|device| {
            let (installed, size) = format_size(&device);
            let [memory_type, speed, manufacturer, part_number] = if installed {
                [
                    device
                        .memory_type()
                        .map(dmi_memory_device_type)
                        .unwrap_or_else(|| "-".to_owned()),
                    format_device_speed(&device),
                    format_string(device.manufacturer()),
                    format_string(device.part_number()),
                ]
            } else {
                [
                    "-".to_owned(),
                    "-".to_owned(),
                    "-".to_owned(),
                    "-".to_owned(),
                ]
            };

            [
                format_string(device.device_locator()),
                format_string(device.bank_locator()),
                size,
                memory_type,
                speed,
                manufacturer,
                part_number,
            ]
        })
        .collect();

    if rows.is_empty() {
        return "No SMBIOS Type 17 memory devices found.".to_owned();
    }

    let mut widths = HEADERS.map(str::len);
    for row in &rows {
        for (column, value) in row.iter().enumerate() {
            widths[column] = widths[column].max(value.chars().count());
        }
    }

    let mut output = String::new();
    write_row(&mut output, &HEADERS.map(str::to_owned), &widths);
    write_row(&mut output, &widths.map(|width| "-".repeat(width)), &widths);
    for row in &rows {
        write_row(&mut output, row, &widths);
    }
    output.pop();
    output
}

fn write_row(output: &mut String, row: &[String; 7], widths: &[usize; 7]) {
    for (column, value) in row.iter().enumerate() {
        if column != 0 {
            output.push_str("  ");
        }
        write!(output, "{:<width$}", value, width = widths[column]).unwrap();
    }
    output.push('\n');
}

fn format_string(value: SMBiosString) -> String {
    value
        .to_utf8_lossy()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "-".to_owned())
}

fn format_size(device: &SMBiosMemoryDevice<'_>) -> (bool, String) {
    match device.size() {
        Some(MemorySize::NotInstalled) => (false, "Empty".to_owned()),
        Some(MemorySize::Unknown) | None => (true, "Unknown".to_owned()),
        Some(MemorySize::Kilobytes(kilobytes)) => {
            (true, format_binary_quantity(kilobytes as u64, "kB"))
        }
        Some(MemorySize::Megabytes(megabytes)) => {
            (true, format_binary_quantity(megabytes as u64, "MB"))
        }
        Some(MemorySize::SeeExtendedSize) => match device.extended_size() {
            Some(MemorySizeExtended::Megabytes(megabytes)) => {
                (true, format_binary_quantity(megabytes as u64, "MB"))
            }
            Some(MemorySizeExtended::SeeSize) | None => (true, "Unknown".to_owned()),
        },
    }
}

fn format_binary_quantity(value: u64, unit: &str) -> String {
    match unit {
        "kB" if value % 1024 == 0 => format!("{} MB", value / 1024),
        "MB" if value % (1024 * 1024) == 0 => format!("{} TB", value / (1024 * 1024)),
        "MB" if value % 1024 == 0 => format!("{} GB", value / 1024),
        _ => format!("{} {}", value, unit),
    }
}

fn format_device_speed(device: &SMBiosMemoryDevice<'_>) -> String {
    match device.configured_memory_speed() {
        Some(configured) => {
            format_speed(Some(configured), device.extended_configured_memory_speed())
        }
        None => format_speed(device.speed(), device.extended_speed()),
    }
}

fn format_speed(speed: Option<MemorySpeed>, extended_speed: Option<MemorySpeedExtended>) -> String {
    match speed {
        Some(MemorySpeed::MTs(speed)) => format!("{} MT/s", speed),
        Some(MemorySpeed::SeeExtendedSpeed) => match extended_speed {
            Some(MemorySpeedExtended::MTs(speed)) => format!("{} MT/s", speed),
            Some(MemorySpeedExtended::SeeSpeed) | None => "Unknown".to_owned(),
        },
        Some(MemorySpeed::Unknown) => "Unknown".to_owned(),
        None => "-".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_device(
        handle: u16,
        size_mb: u16,
        locator: &str,
        bank: &str,
        manufacturer: &str,
        part_number: &str,
    ) -> Vec<u8> {
        let mut record = vec![
            17,
            28,
            handle as u8,
            (handle >> 8) as u8,
            0,
            16,
            0xFE,
            0xFF,
            72,
            0,
            64,
            0,
            size_mb as u8,
            (size_mb >> 8) as u8,
            9,
            0,
            1,
            2,
            0x1A,
            0,
            0,
            0x68,
            0x10,
            3,
            4,
            0,
            5,
            2,
        ];
        for value in [locator, bank, manufacturer, "SERIAL", part_number] {
            record.extend_from_slice(value.as_bytes());
            record.push(0);
        }
        record.push(0);
        record
    }

    #[test]
    fn renders_installed_and_empty_sockets() {
        let mut table = memory_device(0x1100, 8192, "DIMM_A1 ", "BANK 0", "Example", "ABC-123  ");
        table.extend(memory_device(0x1101, 0, "DIMM_A2", "BANK 1", "", ""));
        table.extend([127, 4, 0xFF, 0xFF, 0, 0]);

        let output = render_dimm_map(&SMBiosData::from_vec_and_version(table, None));

        assert!(output.contains("DIMM_A1  BANK 0  8 GB   DDR4  4200 MT/s  Example       ABC-123"));
        assert!(output.contains("DIMM_A2  BANK 1  Empty  -     -          -             -"));
    }

    #[test]
    fn reports_a_table_without_memory_devices() {
        let data = SMBiosData::from_vec_and_version(vec![127, 4, 0xFF, 0xFF, 0, 0], None);
        assert_eq!(
            render_dimm_map(&data),
            "No SMBIOS Type 17 memory devices found."
        );
    }
}
