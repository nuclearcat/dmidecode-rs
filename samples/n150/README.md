# Intel N150 router board

Source: [nuclearcat/dmidecode-rs issue #2](https://github.com/nuclearcat/dmidecode-rs/issues/2),
contributed by **pavel-odintsov**, retrieved September 7, 2026.
The board model is unspecified. All three reports identify AMI BIOS 5.27 dated
2025-09-29, SMBIOS 3.6.0 and dmidecode-rs 0.2.4; the executable's commit is unknown.

| Capture | Capacity | Part number | Reported ranks | Configured speed | Source |
| --- | --- | --- | --- | --- | --- |
| [Netac](netac-16gb.txt) | 16 GB | Blank | 2 | 3200 MT/s | [Issue body](https://github.com/nuclearcat/dmidecode-rs/issues/2) |
| [Kingston](kingston-32gb.txt) | 32 GB | 9905744-111.A00G | 2 | 3200 MT/s | [First comment](https://github.com/nuclearcat/dmidecode-rs/issues/2#issuecomment-5570518342) |
| [Ramaxel](ramaxel-32gb.txt) | 32 GB | RMSA3330ME88HCF-3200 | 2 | 3200 MT/s | [Second comment](https://github.com/nuclearcat/dmidecode-rs/issues/2#issuecomment-5570589964) |

The text files preserve the posted output starting at the version banner.
Markdown fences, the shell command and surrounding prose were removed, and the
ending was normalized to one newline. Field contents, including blank values and
trailing spaces, were retained. [sources.json](sources.json) records provenance
and SHA-256 hashes of these extracted files.

## Observations

- Types 17, 19 and 20 agree on the installed capacity in each capture. Type 16
  reports a 32 GB maximum and one memory device in all three configurations.
  [Intel lists 16 GB as the N150 maximum](https://www.intel.com/content/www/us/en/products/sku/241636/intel-processor-n150-6m-cache-up-to-3-60-ghz/specifications.html)
  (checked September 7, 2026). These captures demonstrate firmware recognition,
  but do not establish OS-visible capacity or stability with 32 GB.
- Type 4 reports `Family: Other`, `Version: Intel(R) N150` and processor ID
  `E0 06 0B 00 FF FB EB BF`. Interpreting the first four bytes as x86 CPUID EAX
  gives `0x000B06E0`: family 6, model 190 (`0xBE`), stepping 0. Linux identifies
  [family 6/model 0xBE as Gracemont / Alder Lake N](https://github.com/torvalds/linux/blob/master/arch/x86/include/asm/intel-family.h).
  This is a shared model identifier, not a unique N150 product identifier.
  The captured decoder left signature and flags undecoded for this version
  string. The fallback rules in `data/processor-id-fallbacks.tsv` now recognize it.
- Default identity strings, the patterned system UUID and sensor thresholds
  numbered 1 through 6 suggest firmware template data. Connector and sensor
  records need independent confirmation.
- The Netac part number is whitespace and its serial is all zeroes. These are
  useful display cases. Kingston and Ramaxel provide nonblank part numbers.
- OEM type 219 at handle 0x0088 changes at byte offset 0x0E from 0x89 in the
  Netac capture to 0x80 in both 32 GB captures. Its meaning is unverified;
  the correlation does not establish that this is a memory-capacity field.

## Missing evidence

Request one binary SMBIOS dump per configuration, comparison output from the C
dmidecode, `lscpu`, OS-visible memory and memory-test results using the
[collection instructions](../README.md#collecting-another-sample).
These text captures are reference material, not golden output for the current
decoder: identical version banners do not identify an exact build.
