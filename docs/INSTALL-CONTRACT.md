# SlickSlax install contract

This document separates the promises SlickSlax can verify from the assumptions delegated to Slax or the host operating system.

## Invariants before a write

- The target was returned by the current platform’s removable/external-disk query.
- The target’s application ID and physical device path still match a fresh scan.
- The operator typed `SLAX` in the final confirmation.
- The requested label contains only an allowlisted FAT-compatible character set.
- The source exists, is a regular file, and has an `.iso` extension.

## Invariants after extraction

- `/slax/boot` existed inside the mounted source image.
- `/slax/boot/vmlinuz`, `/slax/boot/initrfs.img`, and `/slax/boot/syslinux.cfg` exist and are non-empty on the target.
- At least one Slax `.sb` system module exists on the target.
- A read pass and SHA-256 operation can be completed on the first 64 KiB of the copied kernel.

The current verifier proves readable structure, not byte-for-byte equality for the entire copied tree. Full manifest hashing is a planned hardening step.

## Platform boot paths

### Windows

Windows creates an MBR disk with an active FAT32 boot partition and then runs the `bootinst.bat` supplied by the selected Slax image. That script installs Syslinux for legacy BIOS and copies the image’s UEFI files.

Microsoft’s formatter limits FAT32 creation to 32 GB, so the current Windows backend caps the boot partition at 32 GB and the UI caps persistent-session storage at 16 GB. Space beyond that partition is not modified after `Clear-Disk`; exposing it as a separate data volume belongs in a later, separately tested feature.

### Linux

Linux uses PolicyKit for the destructive `parted` and `mkfs.vfat` step, mounts through `udisks2`, then runs Slax’s own `bootinst.sh` as root. The target is synced, unmounted, and powered off when supported.

### macOS

macOS uses `diskutil` for an MBR/FAT32 external disk and `hdiutil` for the read-only source image. Since Slax’s included Extlinux binaries are Linux executables, the macOS backend installs the image’s UEFI boot tree directly. The resulting drive targets UEFI computers; macOS cannot provide Slax’s legacy-BIOS Syslinux installation without an additional native helper.

## Failure rule

SlickSlax never reports success after a failed phase. It emits the failing native command’s diagnostic, leaves the drive attached when automatic cleanup cannot be proved, and asks the operator to review setup. It does not automatically retry a destructive command.

