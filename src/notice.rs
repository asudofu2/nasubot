use crate::pcstatus::{DiskSpace, ScrubStatus};
use log::{error, info};
use std::collections::HashMap;
use std::error::Error;

pub struct NoticeInfo<'a> {
    /// ディスク容量情報
    pub disk_spaces: &'a [DiskSpace],
    /// btrfs scrub status
    pub btrfs_scrub_status: &'a [ScrubStatus],
    /// アラートを出す残り容量閾値(%)
    pub remaining_space_alert: u32,
    /// SlackのWebhook URL
    pub slack_webhook_url: String,
}

pub struct NotifyedInfo {
    pub general: bool,
    pub low_remaining_disk_space: bool,
    pub btrfs_scrub_error: bool,
    pub send_to_slack: bool,
}

pub async fn notify<'a>(info: &NoticeInfo<'a>) -> Result<NotifyedInfo, Box<dyn Error>> {
    let mut result = NotifyedInfo {
        general: false,
        low_remaining_disk_space: false,
        btrfs_scrub_error: false,
        send_to_slack: false,
    };

    // エラーが発生しても次の処理を続行する

    let mut message = String::new();

    // 一般的な通知
    match make_general_message(info) {
        Ok(text) => {
            result.general = true;
            message.push_str(&text);
            message.push_str("\n\n");
        }
        Err(e) => {
            error!("Failed to make general: {}", e);
            message.push_str("Failed to make general.");
        }
    };

    // 残量が少ないディスク通知
    match check_remaining_disk_size(info.disk_spaces, info.remaining_space_alert) {
        Ok(low_remaining_disks) => {
            if !low_remaining_disks.is_empty() {
                message.push_str(":警告: Low remining disks:\n");
                for disk in low_remaining_disks {
                    message.push_str(&format!("{}\n", disk.mount_point.display()));
                }
            }
            result.low_remaining_disk_space = true;
        }
        Err(e) => {
            error!("Failed to check disk: {}", e);
            message.push_str("Failed to check disk.");
        }
    };

    // Scrubの結果通知
    // match check_btrfs_scrub(&info.disk_spaces) {
    //     Ok(text) => {
    //         message.push_str(&text);
    //         result.btrfs_scrub_error = true;
    //     }
    //     Err(e) => {
    //         error!("Failed to check btrfs scrub: {}", e);
    //         message.push_str("Failed to check btrfs scrub.");
    //     }
    // }

    match make_btrfs_status(&info.btrfs_scrub_status) {
        Ok(text) => {
            message.push_str(&text);
            result.btrfs_scrub_error = true;
        }
        Err(e) => {
            error!("Failed to check btrfs scrub: {}", e);
            message.push_str("Failed to check btrfs scrub.");
        }
    }

    match notify_to_slack(&message, &info.slack_webhook_url).await {
        Ok(()) => {
            result.send_to_slack = true;
        }
        Err(e) => {
            error!("Failed to send to slack: {}", e);
        }
    }

    Ok(result)
}

fn make_general_message<'a>(info: &NoticeInfo<'a>) -> Result<String, Box<dyn Error>> {
    let mut text = String::from("Disk space\n");
    for ds in info.disk_spaces {
        text.push_str(&format!(
            "{}: {} GB / {} GB\n",
            ds.mount_point.display(),
            ds.available_space / 1024 / 1024 / 1024,
            ds.total / 1024 / 1024 / 1024,
        ));
    }

    Ok(text)
}

fn check_remaining_disk_size(
    disk_space: &[DiskSpace],
    remaining_space_alert: u32,
) -> Result<Vec<&DiskSpace>, Box<dyn Error>> {
    let mut low_remaining_disk = Vec::new();

    for ds in disk_space {
        let used = ds.total - ds.available_space;
        let used_percent = (used as f64 / ds.total as f64) * 100.0;

        if used_percent >= 100.0 - remaining_space_alert as f64 {
            low_remaining_disk.push(ds);
        }
    }

    Ok(low_remaining_disk)
}

fn make_btrfs_status(status: &[ScrubStatus]) -> Result<String, Box<dyn Error>> {
    let mut message = String::from("btrfs scrub status:\n");

    for info in status {
        if info.error {
            message.push_str("⚠⚠⚠⚠");
        }
        let text = format!(
            "Mount point: {}\n{}\n",
            info.mount_point.to_str().unwrap_or(""),
            info.message
        );
        message.push_str(&text);
    }

    Ok(message)
}

async fn notify_to_slack(text: &str, webhook_url: &str) -> Result<(), Box<dyn Error>> {
    info!("Send message: {}", text);

    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("text", text);

    let res = client.post(webhook_url).json(&params).send().await?;
    info!("Response: {:?}", res);

    if res.status().is_success() {
        info!("Successfully notified to slack");
    } else {
        error!("Failed to notify to slack: {}", res.status());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    // #[tokio::test]
    // async fn test_notify_slack() {
    //     let info = NoticeInfo {
    //         disk_spaces: &[DiskSpace {
    //             mount_point: path::PathBuf::from("C:\\"),
    //             total: 100,
    //             available_space: 50,
    //         }],
    //         remaining_space_alert: 10,
    //         slack_webhook_url: String::from("test webhook url"),
    //     };

    //     let notifyed = notify(&info).await.unwrap();
    //     assert!(notifyed.general);
    //     assert!(notifyed.low_remaining_disk_space);
    //     assert!(!notifyed.btrfs_scrub_error);
    // }

    #[tokio::test]
    #[ignore = "Slackへの通知が発生するためCIで実行しない"]
    async fn slack_access_test() {
        dotenvy::dotenv().unwrap();

        let webhook_url = env::var("TEST_SLACK_WEBHOOK_URL").unwrap();
        let text = "test";
        notify_to_slack(text, &webhook_url).await.unwrap();
        // assert_eq!(res, ());
    }
}
