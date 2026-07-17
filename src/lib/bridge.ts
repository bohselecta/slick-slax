import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { InstallProgress, InstallRequest, IsoInfo, UsbDrive } from "../types";

const demoDrives: UsbDrive[] = [
  {
    id: "demo-sandisk",
    device: "/dev/disk4",
    name: "Ultra Fit",
    vendor: "SanDisk",
    sizeBytes: 32_000_000_000,
    removable: true,
    mountPoints: ["/Volumes/UNTITLED"],
    filesystem: "exFAT",
    partitionScheme: "GPT",
    system: false,
  },
  {
    id: "demo-kingston",
    device: "/dev/disk5",
    name: "DataTraveler 3.0",
    vendor: "Kingston",
    sizeBytes: 64_000_000_000,
    removable: true,
    mountPoints: [],
    filesystem: "FAT32",
    partitionScheme: "MBR",
    system: false,
  },
];

export const nativeAvailable = isTauri();

export async function chooseIso(): Promise<IsoInfo | null> {
  if (!nativeAvailable) {
    return {
      path: "/Users/demo/Downloads/slax-64bit.iso",
      filename: "slax-debian-12.2.0-x64.iso",
      sizeBytes: 437_000_000,
      slaxRootFound: true,
      edition: "Debian 12.2",
      architecture: "64-bit",
    };
  }
  return invoke<IsoInfo | null>("choose_iso");
}

export async function listDrives(): Promise<UsbDrive[]> {
  return nativeAvailable ? invoke<UsbDrive[]>("list_usb_drives") : demoDrives;
}

export async function openOfficialDownload(): Promise<void> {
  if (nativeAvailable) await invoke("open_official_download");
  else window.open("https://www.slax.org/#getslax", "_blank", "noopener,noreferrer");
}

export async function startInstall(request: InstallRequest): Promise<void> {
  if (nativeAvailable) return invoke("install_slax", { request });
  void request;
}

export async function subscribeToProgress(
  handler: (progress: InstallProgress) => void,
): Promise<UnlistenFn> {
  if (nativeAvailable) return listen<InstallProgress>("install-progress", (event) => handler(event.payload));
  return () => undefined;
}

export function runDemoInstall(handler: (progress: InstallProgress) => void): () => void {
  const frames: InstallProgress[] = [
    { phase: "preparing", percent: 6, title: "Reading Slax", detail: "Checking the ISO and target drive" },
    { phase: "formatting", percent: 18, title: "Preparing the pocket", detail: "Creating an MBR partition map and FAT32 volume" },
    { phase: "copying", percent: 43, title: "Copying Slax", detail: "Moving the /slax system to your drive" },
    { phase: "copying", percent: 69, title: "Copying Slax", detail: "Writing modules and boot assets" },
    { phase: "bootloader", percent: 82, title: "Making it bootable", detail: "Installing the Slax bootloader" },
    { phase: "verifying", percent: 94, title: "One last check", detail: "Verifying structure, boot files, and persistence" },
    { phase: "complete", percent: 100, title: "Your pocket OS is ready", detail: "Eject the drive, restart, and choose USB from your boot menu" },
  ];
  let index = 0;
  handler(frames[0]);
  const timer = window.setInterval(() => {
    index += 1;
    if (index >= frames.length) return window.clearInterval(timer);
    handler(frames[index]);
  }, 850);
  return () => window.clearInterval(timer);
}

