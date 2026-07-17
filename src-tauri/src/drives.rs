use crate::model::UsbDrive;
use serde_json::Value as JsonValue;
use std::process::Command;

pub fn list_usb_drives() -> Result<Vec<UsbDrive>, String> {
    #[cfg(target_os = "linux")]
    return linux_drives();
    #[cfg(target_os = "macos")]
    return macos_drives();
    #[cfg(target_os = "windows")]
    return windows_drives();
    #[allow(unreachable_code)]
    Err("SlickSlax does not support drive discovery on this platform yet.".into())
}

fn run(command: &mut Command) -> Result<String, String> {
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "linux")]
fn linux_drives() -> Result<Vec<UsbDrive>, String> {
    let text = run(Command::new("lsblk").args([
        "--json", "--bytes", "--output",
        "NAME,PATH,MODEL,VENDOR,SERIAL,SIZE,TRAN,TYPE,RM,MOUNTPOINTS,FSTYPE,PTTYPE",
    ]))?;
    let json: JsonValue = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    let mut result = Vec::new();
    for disk in json["blockdevices"].as_array().into_iter().flatten() {
        if disk["type"].as_str() != Some("disk") { continue; }
        let removable = disk["rm"].as_bool().unwrap_or(false) || disk["rm"].as_u64() == Some(1);
        let usb = disk["tran"].as_str() == Some("usb");
        if !removable && !usb { continue; }
        let device = disk["path"].as_str().unwrap_or_default().to_string();
        let mut mount_points = Vec::new();
        let mut filesystem = None;
        if let Some(children) = disk["children"].as_array() {
            for child in children {
                if filesystem.is_none() { filesystem = child["fstype"].as_str().map(str::to_string); }
                if let Some(points) = child["mountpoints"].as_array() {
                    mount_points.extend(points.iter().filter_map(|point| point.as_str().map(str::to_string)));
                }
            }
        }
        result.push(UsbDrive {
            id: format!("linux:{}", device),
            device,
            name: disk["model"].as_str().unwrap_or("USB drive").trim().to_string(),
            vendor: disk["vendor"].as_str().map(|value| value.trim().to_string()).filter(|value| !value.is_empty()),
            size_bytes: disk["size"].as_u64().unwrap_or_default(),
            removable: true,
            mount_points,
            filesystem,
            partition_scheme: disk["pttype"].as_str().map(|value| if value == "dos" { "MBR".into() } else { value.to_uppercase() }),
            system: false,
        });
    }
    Ok(result)
}

#[cfg(target_os = "macos")]
fn macos_drives() -> Result<Vec<UsbDrive>, String> {
    let output = Command::new("diskutil").args(["list", "-plist", "external", "physical"]).output().map_err(|error| error.to_string())?;
    if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
    let value = plist::Value::from_reader_xml(output.stdout.as_slice()).map_err(|error| error.to_string())?;
    let dict = value.as_dictionary().ok_or("Unexpected diskutil response")?;
    let disks = dict.get("AllDisksAndPartitions").and_then(plist::Value::as_array).ok_or("No disks returned")?;
    let mut result = Vec::new();
    for disk in disks {
        let Some(entry) = disk.as_dictionary() else { continue };
        let Some(identifier) = entry.get("DeviceIdentifier").and_then(plist::Value::as_string) else { continue };
        let device = format!("/dev/{identifier}");
        let info = run(Command::new("diskutil").args(["info", "-plist", &device]))?;
        let info_value = plist::Value::from_reader_xml(info.as_bytes()).map_err(|error| error.to_string())?;
        let details = info_value.as_dictionary().ok_or("Unexpected diskutil info")?;
        if details.get("Internal").and_then(plist::Value::as_boolean).unwrap_or(true) { continue; }
        let mut mount_points = Vec::new();
        let mut filesystem = None;
        if let Some(parts) = entry.get("Partitions").and_then(plist::Value::as_array) {
            for part in parts {
                let Some(part) = part.as_dictionary() else { continue };
                if let Some(point) = part.get("MountPoint").and_then(plist::Value::as_string) { mount_points.push(point.to_string()); }
                if filesystem.is_none() { filesystem = part.get("FilesystemType").and_then(plist::Value::as_string).map(str::to_string); }
            }
        }
        result.push(UsbDrive {
            id: format!("macos:{device}"),
            device,
            name: details.get("MediaName").and_then(plist::Value::as_string).unwrap_or("USB drive").to_string(),
            vendor: details.get("DeviceVendor").and_then(plist::Value::as_string).map(str::to_string),
            size_bytes: details.get("TotalSize").and_then(plist::Value::as_unsigned_integer).unwrap_or_default(),
            removable: details.get("Removable").and_then(plist::Value::as_boolean).unwrap_or(true),
            mount_points,
            filesystem,
            partition_scheme: entry.get("Content").and_then(plist::Value::as_string).map(|value| if value.contains("FDisk") { "MBR".into() } else { "GPT".into() }),
            system: false,
        });
    }
    Ok(result)
}

#[cfg(target_os = "windows")]
fn windows_drives() -> Result<Vec<UsbDrive>, String> {
    let script = r#"
$items = Get-CimInstance Win32_DiskDrive | Where-Object { $_.InterfaceType -eq 'USB' } | ForEach-Object {
  $disk = $_
  $letters = @()
  $fs = $null
  Get-CimAssociatedInstance -InputObject $disk -ResultClassName Win32_DiskPartition | ForEach-Object {
    Get-CimAssociatedInstance -InputObject $_ -ResultClassName Win32_LogicalDisk | ForEach-Object {
      $letters += $_.DeviceID + '\'
      if (-not $fs) { $fs = $_.FileSystem }
    }
  }
  $style = (Get-Disk -Number $disk.Index).PartitionStyle.ToString()
  [PSCustomObject]@{ Device=$disk.DeviceID; Name=$disk.Model; Vendor=$disk.Manufacturer; Size=[uint64]$disk.Size; MountPoints=$letters; Filesystem=$fs; PartitionStyle=$style; Index=$disk.Index }
}
@($items) | ConvertTo-Json -Compress
"#;
    let text = run(Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", script]))?;
    let values: Vec<JsonValue> = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    Ok(values.into_iter().map(|disk| {
        let device = disk["Device"].as_str().unwrap_or_default().to_string();
        UsbDrive {
            id: format!("windows:{}", disk["Index"].as_u64().unwrap_or_default()),
            device,
            name: disk["Name"].as_str().unwrap_or("USB drive").to_string(),
            vendor: disk["Vendor"].as_str().map(str::to_string),
            size_bytes: disk["Size"].as_u64().unwrap_or_default(),
            removable: true,
            mount_points: disk["MountPoints"].as_array().into_iter().flatten().filter_map(|value| value.as_str().map(str::to_string)).collect(),
            filesystem: disk["Filesystem"].as_str().map(str::to_string),
            partition_scheme: disk["PartitionStyle"].as_str().map(|value| value.to_uppercase()),
            system: false,
        }
    }).collect())
}
