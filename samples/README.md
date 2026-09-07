# Hardware samples

These are contributed observations, not guarantees of hardware support. Each
sample directory records its sources and any transformations. Decoded text cannot
recover every original SMBIOS field or replace a binary parser fixture.

- [Intel N150 router board](n150/README.md): three memory configurations from issue #2.

## Collecting another sample

On Linux, capture each memory configuration separately in a new directory:

```sh
mkdir n150-capture
cd n150-capture
sudo dmidecode --dump-bin smbios.bin
sudo dmidecode --from-dump smbios.bin > dmidecode.txt
sudo dmidecode-rs --from-dump smbios.bin > dmidecode-rs.txt
sudo dmidecode-rs --from-dump smbios.bin --json-pretty > smbios.json
sudo dmidecode-rs --from-dump smbios.bin --dimm-map > dimm-map.txt
LC_ALL=C lscpu > lscpu.txt
LC_ALL=C free -b > memory.txt
LC_ALL=C lspci -nn > pci.txt
uname -srmo > kernel.txt
dmidecode --version > dmidecode-version.txt
dmidecode-rs --version > dmidecode-rs-version.txt
```

Record the board model, BIOS version, module label and capacity, and the
dmidecode-rs commit used. Include a source URL or contributor attribution.
Review captures before sharing: SMBIOS can contain serial numbers, UUIDs and
asset tags. Document any redactions; changing binary strings requires preserving
the structure layout.

The binary dump lets both decoders read the same data offline. `lscpu` supplies
independent CPU identification, `free` shows OS-visible memory, and `lspci`
checks the firmware's device inventory. Memory stability needs separate test
results with the tool version, tested capacity, duration and error count.
