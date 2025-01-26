use std::process::Command;
use std::{
    error::Error,
    path::{self, Path},
};
use sysinfo::Disks;

/// ディスク容量取得結果
#[derive(Debug)]
pub struct DiskSpace {
    pub mount_point: path::PathBuf,
    pub total: u64,
    pub available_space: u64,
}

#[derive(Debug)]
pub struct ScrubStatus {
    pub mount_point: path::PathBuf,
    pub message: String,
    pub error: bool,
}

pub fn disk_space(mount_points: &[&Path]) -> Result<Vec<DiskSpace>, Box<dyn Error>> {
    let disks = Disks::new_with_refreshed_list();

    let mut disk_spaces = Vec::new();

    for disk in disks.list() {
        if mount_points.iter().any(|x| *x == disk.mount_point()) {
            let ds = DiskSpace {
                mount_point: disk.mount_point().to_path_buf(),
                total: disk.total_space(),
                available_space: disk.available_space(),
            };

            disk_spaces.push(ds);
        }
    }

    Ok(disk_spaces)
}

pub fn check_btrfs_scrub(mount_points: &[&Path]) -> Result<Vec<ScrubStatus>, Box<dyn Error>> {
    let mut status_list = Vec::new();

    for mount_point in mount_points {
        let cmd = Command::new("btrfs")
            .args(["scrub", "status", mount_point.to_str().unwrap_or("")])
            .output()?;

        let stdout = String::from_utf8_lossy(&cmd.stdout).to_string();
        log::info!("btrfs scrub status: \n{}", &stdout);
        let is_error = find_error_scrub_status(&stdout);

        status_list.push(ScrubStatus {
            mount_point: mount_point.to_path_buf(),
            message: stdout,
            error: is_error,
        });
    }

    Ok(status_list)
}

fn find_error_scrub_status(message: &str) -> bool {
    for line in message.lines() {
        if line.contains("Error summary") && !line.contains("no errors found") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_space() {
        let mount_points;
        if cfg!(target_os = "windows") {
            mount_points = vec![Path::new("C:\\")];
        } else if cfg!(target_os = "linux") {
            mount_points = vec![Path::new("/")];
        } else {
            panic!("Unsupported OS");
        }
        let disks = disk_space(&mount_points).unwrap();

        assert_eq!(disks.len(), 1);
    }

    #[test]
    fn test_found_error_scrub_status() {
        let message = "UUID:             0000
Scrub started:    Thu Jan 23 22:52:24 2025
Status:           finished
Duration:         0:53:47
Total to scrub:   1.18TiB
Rate:             383.25MiB/s (some device limits set)
Error summary:    csum=72
  Corrected:      2
  Uncorrectable:  72
  Unverified:     0";

        assert!(find_error_scrub_status(&message));
    }

    #[test]
    fn test_not_found_error_scrub_status() {
        let message = "UUID:             0000
Scrub started:    Thu Jan 23 22:52:24 2025
Status:           finished
Duration:         0:53:47
Total to scrub:   1.18TiB
Rate:             383.25MiB/s (some device limits set)
Error summary:    no errors found";
        assert!(!find_error_scrub_status(&message));
    }
}
