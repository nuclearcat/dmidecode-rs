# Processor ID fallback rules

`processor-id-fallbacks.tsv` selects Intel or AMD x86 Processor ID decoding when
SMBIOS Type 4 explicitly reports family Other (0x01) or Unknown (0x02).
It does not replace that family value or infer an exact product from CPUID.
The decoder uses this table directly; no generated copy needs updating.

Rules match case-sensitive version prefixes after trimming surrounding
whitespace. A prefix must end at the end of the string or a whitespace boundary.
This allows clock/model suffixes without matching `N1500` as `N150`.
Generic vendor names, QEMU machine names, blank strings and unknown brands do
not select an encoding. Manufacturer alone is insufficient: Intel also made
non-x86 processors.

## Sources and evidence (checked September 7, 2026)

- The seven legacy Pentium/Core 2/U1400/Athlon/Opteron strings were already in
  `src/dmifn.rs` before this change. They now accept whitespace-separated suffixes.
- [N150 issue #2](https://github.com/nuclearcat/dmidecode-rs/issues/2) supplies an
  observed `Other` record with version `Intel(R) N150` and ID
  `E0 06 0B 00 FF FB EB BF`. The three preserved captures are in `samples/n150`.
- [Intel N-series catalogue](https://www.intel.com/content/www/us/en/ark/products/series/231819/intel-processor-n-series.html)
  and [Intel Processor catalogue](https://www.intel.com/content/www/us/en/products/details/processors/intel-processor.html)
  identify the N-series products. The additional N-series rules extend the known
  Intel ID encoding to named products; they do not claim an observed `Other`
  dump for every model.
- [dmidecode's processor-family and CPUID dispatch](https://github.com/mirror/dmidecode/blob/master/dmidecode.c)
  identifies Intel Core, Xeon, Celeron and Atom families as using Intel x86 ID
  encoding. This supports brand-prefix fallback when firmware omits the family.
  That upstream source also selects Family 2 only when Family is 0xFE.
- [AMD embedded processor documentation](https://www.amd.com/en/products/embedded.html)
  identifies Ryzen and EPYC as x86 processors. Their added prefixes select the
  existing AMD ID decoder; they are architecture-based extensions, not claims
  that each product has been observed with `Other` firmware records.

Add entries with a source and a test covering both the intended match and a
plausible false match. Prefer a captured Type 4 record and an independently
reported CPU identity. Do not assign meanings to generic `Other` values in
unrelated SMBIOS structures: those need their own evidence and decoding rules.
