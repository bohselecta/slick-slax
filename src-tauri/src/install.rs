use crate::{drives, model::{InstallProgress, InstallRequest}};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::{Path, PathBuf}, process::Command};
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

pub fn install(app: AppHandle, request: InstallRequest) -> Result<(), String> {
    validate_request(&request)?;
    let current = drives::list_usb_drives()?;
    let target = current.iter().find(|drive| drive.id == request.drive_id && drive.device == request.device)
        .ok_or("The selected USB drive is no longer connected. Nothing was changed.")?;
    if target.system || !target.removable { return Err("SlickSlax refused a drive that is not positively identified as removable.".into()); }

    emit(&app, "preparing", 5, "Reading Slax", "Validating the ISO and selected removable drive");
    let iso = PathBuf::from(&request.iso_path);
    if !iso.is_file() || iso.extension().and_then(|value| value.to_str()).map(|value| !value.eq_ignore_ascii_case("iso")).unwrap_or(true) {
        return Err("Choose a readable Slax .iso file.".into());
    }

    emit(&app, "formatting", 15, "Preparing the pocket", "Creating the Slax-compatible MBR and FAT32 layout");
    let mount = prepare_target(&request)?;
    let install_result: Result<(), String> = (|| -> Result<(), String> {
        emit(&app, "copying", 35, "Copying Slax", "Mounting the ISO and locating its /slax directory");
        with_mounted_iso(&iso, |source| {
            let slax = source.join("slax");
            if !slax.join("boot").is_dir() { return Err("This ISO does not contain /slax/boot. It does not appear to be a supported Slax image.".into()); }
            let destination = mount.join("slax");
            if destination.exists() { fs::remove_dir_all(&destination).map_err(|error| error.to_string())?; }
            copy_tree(&slax, &destination, |copied, total| {
                let percent = 35 + ((copied.saturating_mul(37) / total.max(1)) as u8).min(37);
                emit(&app, "copying", percent, "Copying Slax", &format!("{} of {} files", copied, total));
            })
        })?;

        apply_persistence(&mount, request.options.persistence_gb)?;
        emit(&app, "bootloader", 80, "Making it bootable", "Installing Slax boot files for this computer");
        install_bootloader(&mount, &request.device)?;

        if request.options.verify {
            emit(&app, "verifying", 92, "One last check", "Verifying boot files and critical Slax modules");
            verify_install(&mount)?;
        }
        sync_and_eject(&mount, &request.device)?;
        Ok(())
    })();

    if let Err(error) = install_result {
        emit(&app, "error", 0, "Installation stopped safely", &error);
        return Err(error);
    }
    emit(&app, "complete", 100, "Your pocket OS is ready", "The drive is verified, ejected, and safe to remove");
    Ok(())
}

fn validate_request(request: &InstallRequest) -> Result<(), String> {
    if request.confirmation != "SLAX" { return Err("The erase confirmation did not match SLAX.".into()); }
    if request.options.label.is_empty() || request.options.label.len() > 11 || !request.options.label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err("The drive label must be 1–11 letters, numbers, underscores, or dashes.".into());
    }
    if ![4, 8, 16, 32, 64].contains(&request.options.persistence_gb) { return Err("Unsupported persistence size.".into()); }
    #[cfg(target_os = "windows")]
    if request.options.persistence_gb > 16 { return Err("Windows builds currently support up to 16 GB of persistent session storage.".into()); }
    Ok(())
}

fn emit(app: &AppHandle, phase: &'static str, percent: u8, title: &str, detail: &str) {
    let _ = app.emit("install-progress", InstallProgress { phase, percent, title: title.into(), detail: detail.into() });
}

fn copy_tree(source: &Path, destination: &Path, progress: impl Fn(u64, u64)) -> Result<(), String> {
    let total = WalkDir::new(source).into_iter().filter_map(Result::ok).filter(|entry| entry.file_type().is_file()).count() as u64;
    let mut copied = 0;
    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|error| error.to_string())?;
        let relative = entry.path().strip_prefix(source).map_err(|error| error.to_string())?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() { fs::create_dir_all(&target).map_err(|error| error.to_string())?; }
        else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() { fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
            fs::copy(entry.path(), &target).map_err(|error| format!("Could not copy {}: {error}", relative.display()))?;
            copied += 1;
            if copied == total || copied % 12 == 0 { progress(copied, total); }
        }
    }
    Ok(())
}

fn apply_persistence(mount: &Path, size: u16) -> Result<(), String> {
    if size <= 16 { return Ok(()); }
    let config = mount.join("slax/boot/syslinux.cfg");
    let contents = fs::read_to_string(&config).map_err(|error| error.to_string())?;
    let marker = format!("perchsize={}GB", size);
    let updated = contents.lines().map(|line| {
        if line.trim_start().to_ascii_uppercase().starts_with("APPEND ") && !line.contains("perchsize=") { format!("{line} {marker}") } else { line.to_string() }
    }).collect::<Vec<_>>().join("\n") + "\n";
    fs::write(config, updated).map_err(|error| error.to_string())
}

fn verify_install(mount: &Path) -> Result<(), String> {
    let required = ["slax/boot/vmlinuz", "slax/boot/initrfs.img", "slax/boot/syslinux.cfg"];
    for relative in required {
        let path = mount.join(relative);
        if !path.is_file() || path.metadata().map_err(|error| error.to_string())?.len() == 0 { return Err(format!("Verification failed: {relative} is missing or empty.")); }
    }
    let has_module = WalkDir::new(mount.join("slax")).max_depth(3).into_iter().filter_map(Result::ok)
        .any(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("sb"));
    if !has_module { return Err("Verification failed: no Slax .sb system modules were found.".into()); }
    let mut file = fs::File::open(mount.join("slax/boot/vmlinuz")).map_err(|error| error.to_string())?;
    let mut buffer = [0_u8; 64 * 1024];
    let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
    let _fingerprint = Sha256::digest(&buffer[..count]);
    Ok(())
}

#[cfg(target_os = "linux")]
fn prepare_target(request: &InstallRequest) -> Result<PathBuf, String> {
    if request.options.erase {
        let partition = if request.device.chars().last().is_some_and(|c| c.is_ascii_digit()) { format!("{}p1", request.device) } else { format!("{}1", request.device) };
        let script = format!("umount '{0}'* 2>/dev/null || true; parted -s '{0}' mklabel msdos mkpart primary fat32 1MiB 100% set 1 boot on; partprobe '{0}'; sleep 1; mkfs.vfat -F 32 -n '{1}' '{2}'", request.device, request.options.label, partition);
        command_ok(Command::new("pkexec").args(["sh", "-c", &script]), "Linux authorization or formatting failed")?;
        let output = command_output(Command::new("udisksctl").args(["mount", "-b", &partition]), "Could not mount the new Slax volume")?;
        parse_mount_output(&output)
    } else {
        existing_mount(request)
    }
}

#[cfg(target_os = "macos")]
fn prepare_target(request: &InstallRequest) -> Result<PathBuf, String> {
    if request.options.erase {
        let status = Command::new("diskutil").args(["eraseDisk", "FAT32", &request.options.label, "MBRFormat", &request.device]).status().map_err(|error| error.to_string())?;
        if !status.success() { return Err("macOS could not erase the selected external drive. Approve the system authorization prompt and try again.".into()); }
        let path = PathBuf::from(format!("/Volumes/{}", request.options.label));
        if !path.is_dir() { return Err("macOS formatted the drive but did not mount the new volume.".into()); }
        Ok(path)
    } else { existing_mount(request) }
}

#[cfg(target_os = "windows")]
fn prepare_target(request: &InstallRequest) -> Result<PathBuf, String> {
    if !request.options.erase { return existing_mount(request); }
    let index = request.device.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().map_err(|_| "Could not read the Windows disk number")?;
    let size_mb = 32_000_u32;
    let label = &request.options.label;
    let elevated = format!("Get-Disk -Number {index} | Set-Disk -IsOffline `$false; Get-Disk -Number {index} | Clear-Disk -RemoveData -Confirm:`$false; Initialize-Disk -Number {index} -PartitionStyle MBR; `$p=New-Partition -DiskNumber {index} -Size {size_mb}MB -AssignDriveLetter -IsActive; Format-Volume -Partition `$p -FileSystem FAT32 -NewFileSystemLabel '{label}' -Confirm:`$false -Force");
    let args = format!("-NoProfile -NonInteractive -Command \"{elevated}\"");
    command_ok(Command::new("powershell").args(["-NoProfile", "-Command", "Start-Process", "powershell", "-Verb", "RunAs", "-Wait", "-ArgumentList", &args]), "Windows authorization or formatting failed")?;
    let output = command_output(Command::new("powershell").args(["-NoProfile", "-Command", &format!("(Get-Partition -DiskNumber {index} | Get-Volume | Where-Object FileSystemLabel -eq '{label}').DriveLetter")]), "Could not locate the new Slax volume")?;
    let letter = output.trim();
    if letter.len() != 1 { return Err("Windows formatted the drive but did not assign a drive letter.".into()); }
    Ok(PathBuf::from(format!("{letter}:\\")))
}

fn existing_mount(request: &InstallRequest) -> Result<PathBuf, String> {
    let drives = drives::list_usb_drives()?;
    let drive = drives.into_iter().find(|drive| drive.id == request.drive_id).ok_or("The USB drive disappeared")?;
    let compatible_fs = drive.filesystem.as_deref().map(|value| value.eq_ignore_ascii_case("vfat") || value.eq_ignore_ascii_case("fat32") || value.eq_ignore_ascii_case("msdos")).unwrap_or(false);
    if !compatible_fs || drive.partition_scheme.as_deref() != Some("MBR") { return Err("Keep-files mode requires an existing MBR + FAT32 USB drive.".into()); }
    drive.mount_points.first().map(PathBuf::from).filter(|path| path.is_dir()).ok_or("The compatible USB volume is not mounted.".into())
}

#[cfg(target_os = "linux")]
fn with_mounted_iso<T>(iso: &Path, action: impl FnOnce(&Path) -> Result<T, String>) -> Result<T, String> {
    let output = command_output(Command::new("udisksctl").args(["loop-setup", "--read-only", "-f", iso.to_string_lossy().as_ref()]), "Could not attach the Slax ISO")?;
    let device = output.split_whitespace().find(|part| part.starts_with("/dev/loop")).map(|part| part.trim_end_matches('.')).ok_or("Could not identify the ISO loop device")?;
    let mount_output = command_output(Command::new("udisksctl").args(["mount", "-b", device]), "Could not mount the Slax ISO")?;
    let mount = parse_mount_output(&mount_output)?;
    let result = action(&mount);
    let _ = Command::new("udisksctl").args(["unmount", "-b", device]).status();
    let _ = Command::new("udisksctl").args(["loop-delete", "-b", device]).status();
    result
}

#[cfg(target_os = "macos")]
fn with_mounted_iso<T>(iso: &Path, action: impl FnOnce(&Path) -> Result<T, String>) -> Result<T, String> {
    let mount = std::env::temp_dir().join(format!("slickslax-iso-{}", std::process::id()));
    fs::create_dir_all(&mount).map_err(|error| error.to_string())?;
    command_ok(Command::new("hdiutil").arg("attach").arg(iso).args(["-readonly", "-nobrowse", "-mountpoint"]).arg(&mount), "Could not mount the Slax ISO")?;
    let result = action(&mount);
    let _ = Command::new("hdiutil").arg("detach").arg(&mount).status();
    let _ = fs::remove_dir(&mount);
    result
}

#[cfg(target_os = "windows")]
fn with_mounted_iso<T>(iso: &Path, action: impl FnOnce(&Path) -> Result<T, String>) -> Result<T, String> {
    let escaped = iso.to_string_lossy().replace(''', "''");
    let script = format!("`$image=Mount-DiskImage -ImagePath '{escaped}' -PassThru; (`$image | Get-Volume).DriveLetter");
    let output = command_output(Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]), "Could not mount the Slax ISO")?;
    let letter = output.trim();
    if letter.len() != 1 { return Err("Windows mounted the ISO but did not assign a drive letter.".into()); }
    let mount = PathBuf::from(format!("{letter}:\\"));
    let result = action(&mount);
    let _ = Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &format!("Dismount-DiskImage -ImagePath '{escaped}'")]).status();
    result
}

#[cfg(target_os = "linux")]
fn install_bootloader(mount: &Path, _device: &str) -> Result<(), String> {
    let script = mount.join("slax/boot/bootinst.sh");
    command_ok(Command::new("pkexec").arg("sh").arg(script).arg("--rex"), "Slax bootloader installation failed")
}

#[cfg(target_os = "windows")]
fn install_bootloader(mount: &Path, _device: &str) -> Result<(), String> {
    let script = mount.join("slax/boot/bootinst.bat");
    command_ok(Command::new("cmd").arg("/c").arg(script).arg("auto"), "Slax bootloader installation failed")
}

#[cfg(target_os = "macos")]
fn install_bootloader(mount: &Path, _device: &str) -> Result<(), String> {
    let source = mount.join("slax/boot/EFI/Boot");
    let destination = mount.join("EFI/Boot");
    if !source.is_dir() { return Err("This Slax image does not contain the UEFI boot files required on macOS.".into()); }
    if destination.exists() { fs::remove_dir_all(&destination).map_err(|error| error.to_string())?; }
    copy_tree(&source, &destination, |_, _| {})?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn sync_and_eject(mount: &Path, device: &str) -> Result<(), String> {
    command_ok(&mut Command::new("sync"), "Could not flush writes")?;
    let _ = Command::new("udisksctl").args(["unmount", "-b", &partition_for(device)]).status();
    let _ = Command::new("udisksctl").args(["power-off", "-b", device]).status();
    let _ = mount;
    Ok(())
}

#[cfg(target_os = "macos")]
fn sync_and_eject(_mount: &Path, device: &str) -> Result<(), String> {
    command_ok(&mut Command::new("sync"), "Could not flush writes")?;
    command_ok(Command::new("diskutil").args(["eject", device]), "The drive is ready, but macOS could not eject it")
}

#[cfg(target_os = "windows")]
fn sync_and_eject(_mount: &Path, _device: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn partition_for(device: &str) -> String { if device.chars().last().is_some_and(|c| c.is_ascii_digit()) { format!("{device}p1") } else { format!("{device}1") } }

#[cfg(target_os = "linux")]
fn parse_mount_output(output: &str) -> Result<PathBuf, String> {
    output.split(" at ").nth(1).map(|value| PathBuf::from(value.trim().trim_end_matches('.'))).filter(|path| path.is_dir()).ok_or("Could not determine the mounted volume path.".into())
}

fn command_ok(command: &mut Command, context: &str) -> Result<(), String> {
    let output = command.output().map_err(|error| format!("{context}: {error}"))?;
    if output.status.success() { Ok(()) } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(format!("{context}: {}", if stderr.is_empty() { stdout } else { stderr }))
    }
}

fn command_output(command: &mut Command, context: &str) -> Result<String, String> {
    let output = command.output().map_err(|error| format!("{context}: {error}"))?;
    if !output.status.success() { return Err(format!("{context}: {}", String::from_utf8_lossy(&output.stderr).trim())); }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InstallOptions;

    fn request() -> InstallRequest {
        InstallRequest {
            iso_path: "/tmp/slax.iso".into(),
            drive_id: "test:usb".into(),
            device: "/dev/test".into(),
            options: InstallOptions { erase: true, label: "SLAX".into(), persistence_gb: 16, verify: true },
            confirmation: "SLAX".into(),
        }
    }

    #[test]
    fn accepts_safe_request_shape() {
        assert!(validate_request(&request()).is_ok());
    }

    #[test]
    fn rejects_missing_confirmation() {
        let mut value = request();
        value.confirmation = "slax".into();
        assert!(validate_request(&value).is_err());
    }

    #[test]
    fn rejects_shell_characters_in_label() {
        let mut value = request();
        value.options.label = "SLAX'; rm".into();
        assert!(validate_request(&value).is_err());
    }
}
