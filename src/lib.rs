mod notice;
mod pcstatus;

use log::info;
use notice::NoticeInfo;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// ディスク容量を取得するマウントポイント
    mount_points: Vec<String>,
    /// 残り容量アラート閾値(%指定)
    remaining_space_alert: u32,
    /// SlackのWebhook URL
    slack_webhook_url: String,
}

pub async fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let mut mount_points = Vec::new();
    for mp in &config.mount_points {
        mount_points.push(path::Path::new(mp));
    }

    let disk_spaces = pcstatus::disk_space(&mount_points)?;
    info!("Target disk: {:?}", disk_spaces);
    let btrfs_scrub_status = pcstatus::check_btrfs_scrub(&mount_points)?;
    info!("btrfs scrub status: {:?}", btrfs_scrub_status);

    notice::notify(&NoticeInfo {
        disk_spaces: &disk_spaces,
        btrfs_scrub_status: &btrfs_scrub_status,
        remaining_space_alert: config.remaining_space_alert,
        slack_webhook_url: config.slack_webhook_url.clone(),
    })
    .await?;

    Ok(())
}
