# SlickSlax

[![Check](https://github.com/bohselecta/slick-slax/actions/workflows/check.yml/badge.svg)](https://github.com/bohselecta/slick-slax/actions/workflows/check.yml)

**The safe, one-window Slax USB maker.**

SlickSlax turns Slax’s manual USB setup into one guided desktop flow on Windows, Linux, and macOS. It does **not** raw-flash the ISO like a generic image writer: Slax needs a writable FAT32 filesystem so its persistent sessions can work. SlickSlax follows the project’s native layout instead:

1. Positively identify a removable USB device.
2. Create an MBR partition map and FAT32 boot volume.
3. Mount the ISO and copy its `/slax` directory to the drive root.
4. Install the bundled Slax boot files.
5. Apply the requested `perchsize` setting.
6. Verify critical boot assets and Slax modules before ejecting.

That distinction is why a Slax-specific tool is useful: a conventional ISO-hybrid flash may boot read-only, fail to persist, or fail entirely.

## What is implemented

- Tauri 2 desktop app with a React 19 / TypeScript interface
- Slax ISO picker and lightweight edition/architecture detection
- Native removable-drive discovery on Windows, Linux, and macOS
- Internal/system-drive exclusion and identity re-check immediately before writing
- Explicit typed erase confirmation
- MBR + FAT32 preparation, `/slax` extraction, boot setup, and verification
- Configurable persistent-session ceiling (4–64 GB where the drive permits)
- Preserve-files mode for a drive that is already MBR + FAT32
- Live native progress events and an honest failure receipt
- GitHub Actions builds for Windows, Ubuntu, and macOS
- Browser demo mode for interface development without touching a disk
- No accounts, analytics, advertising, or telemetry

## Platform notes

| Platform | Disk preparation | Boot compatibility | Native requirements |
|---|---|---|---|
| Windows 10/11 | Elevated PowerShell, 32 GB FAT32 boot partition | Legacy BIOS + UEFI through Slax’s `bootinst.bat` | PowerShell 5+ |
| Desktop Linux | PolicyKit + `parted` + `mkfs.vfat` | Legacy BIOS + UEFI through Slax’s `bootinst.sh` | `udisks2`, `parted`, `dosfstools`, PolicyKit |
| macOS 10.15+ | `diskutil` MBR + FAT32 | UEFI boot files; legacy-BIOS setup is not available from macOS | `diskutil`, `hdiutil` |

Windows intentionally creates a 32 GB FAT32 Slax boot partition because Microsoft’s built-in formatter refuses larger FAT32 volumes. Larger devices remain bootable; a later release can expose the unused remainder as an optional data partition. Linux and macOS can format the full device as FAT32.

## Run the interface

```bash
npm install
npm run dev
```

When it runs in an ordinary browser, SlickSlax uses two clearly fake removable drives and a simulated write sequence. No disk API is available and nothing is changed.

## Run the desktop app

Install the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system, then:

```bash
npm install
npm run tauri dev
```

The npm lifecycle generates all platform icon formats from `app-icon.svg` before Tauri starts, so the repository does not carry generated icon binaries.

Build an installer with:

```bash
npm run tauri build
```

## Safety model

Disk writers deserve paranoia. SlickSlax applies these checks before any destructive command:

- the source must be a readable `.iso` file;
- the confirmation must exactly equal `SLAX`;
- the target must still be present in a fresh native device scan;
- its stable app ID and physical device path must both match the selection;
- the scanner must positively classify it as removable/external;
- the Slax directory is not copied until the target is mounted successfully;
- the ISO must contain `/slax/boot`;
- verification requires a kernel, initramfs, boot config, and at least one `.sb` system module.

The native layer never accepts an arbitrary shell command from the webview.

## Project structure

```text
src/                  React interface and Tauri bridge
src-tauri/src/        native drive discovery and install engine
.github/workflows/    verification and cross-platform packaging
```

## Relationship to Slax

SlickSlax is an independent open-source contribution made for the Slax community. Slax is created by Tomáš Matějíček; its name, artwork, releases, and bundled boot utilities remain the property of their respective owners. SlickSlax does not redistribute Slax itself and deliberately sends downloads to the official [Slax website](https://www.slax.org/).

The implementation follows Slax’s official [starting instructions](https://www.slax.org/starting.php) and the boot process in [Linux Live Kit](https://github.com/Tomas-M/linux-live).

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before changing disk behavior. UI changes can be tested in browser demo mode; destructive-path changes require removable hardware and the manual test matrix.

MIT licensed. See [LICENSE](LICENSE).
