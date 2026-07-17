export type Step = "source" | "drive" | "options" | "write" | "done";

export interface UsbDrive {
  id: string;
  device: string;
  name: string;
  vendor?: string;
  sizeBytes: number;
  removable: boolean;
  mountPoints: string[];
  filesystem?: string;
  partitionScheme?: string;
  system: boolean;
}

export interface IsoInfo {
  path: string;
  filename: string;
  sizeBytes: number;
  slaxRootFound: boolean;
  edition?: string;
  architecture?: string;
}

export interface InstallOptions {
  erase: boolean;
  label: string;
  persistenceGb: number;
  verify: boolean;
}

export interface InstallRequest {
  isoPath: string;
  driveId: string;
  device: string;
  options: InstallOptions;
  confirmation: string;
}

export interface InstallProgress {
  phase: "preparing" | "formatting" | "copying" | "bootloader" | "verifying" | "complete" | "error";
  percent: number;
  title: string;
  detail: string;
}

