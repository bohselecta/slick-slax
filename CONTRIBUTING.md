# Contributing to SlickSlax

Thank you for helping make Slax easier to start.

## Ground rules

1. Preserve Slax’s writable USB design. Do not replace the file-copy install with raw ISO imaging unless it is a separately named, clearly explained mode.
2. Never weaken removable-device checks to make a test pass.
3. Never interpolate a webview-provided value into a general-purpose shell command. Validate labels, resolve physical devices natively, and use argument arrays wherever the platform allows it.
4. Keep browser demo data unmistakably synthetic.
5. Describe platform limitations plainly.

## Development checks

```bash
npm install
npm test
npm run build
npm run tauri build
```

Native disk changes also need a real-device pass on the operating system they affect:

- 8–32 GB USB drive
- 64+ GB USB drive
- unplug between selection and confirmation
- cancel the elevation dialog
- invalid/non-Slax ISO
- preformatted MBR + FAT32 preserve-files path
- boot on UEFI hardware
- boot on legacy BIOS hardware where supported
- create, resume, and select persistent sessions

Never test destructive paths with a virtual disk that the native scanner can confuse with the host system disk.

## Pull requests

Keep each pull request focused. Include:

- what user-visible behavior changed;
- which platform paths changed;
- automated test results;
- the hardware/manual matrix you ran;
- screenshots for interface changes.

