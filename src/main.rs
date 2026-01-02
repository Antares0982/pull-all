use futures::future::join_all;
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 获取GIT_DIRS环境变量
    let git_dirs = env::var("GIT_DIRS").expect("GIT_DIRS environment variable is not set");

    // 读取该目录下所有子目录
    let mut entries = fs::read_dir(&git_dirs).await?;
    let mut directories = vec![];

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        }
    }

    if directories.is_empty() {
        eprintln!("No directories found in GIT_DIRS: {}", git_dirs);
        return Ok(());
    }

    // 并发执行git pull任务
    let mut tasks = vec![];

    for dir in directories.iter() {
        let dir_clone = dir.clone();
        let task = tokio::spawn(async move {
            // 执行 git pull
            let output = Command::new("git")
                .arg("pull")
                .current_dir(&dir_clone)
                .output()
                .await;

            (dir_clone, output)
        });
        tasks.push(task);
    }

    let results = join_all(tasks).await;

    let mut failed_dirs = vec![];
    let mut succeeded_dirs = vec![];

    for result in results {
        match result {
            Ok((dir, Ok(output))) => {
                if output.status.success() {
                    succeeded_dirs.push(dir);
                } else {
                    failed_dirs.push((dir, String::from_utf8_lossy(&output.stderr).to_string()));
                }
            }
            Ok((dir, Err(e))) => {
                failed_dirs.push((dir, format!("Failed to execute git pull: {}", e)));
            }
            Err(e) => {
                failed_dirs.push((PathBuf::from("unknown"), format!("Task join error: {}", e)));
            }
        }
    }

    let total = directories.len();
    let success_num = succeeded_dirs.len();
    let fail_num = failed_dirs.len();

    let notify_title;
    let notify_message;

    if fail_num == total {
        notify_title = "Error";
        let failed_list = failed_dirs
            .iter()
            .map(|(dir, err)| format!("{}: {}", dir.to_string_lossy(), err))
            .collect::<Vec<_>>()
            .join("\n");
        notify_message = format!(
            "Failed to pull all the following {} repositories:\n{}",
            fail_num, failed_list
        );
    } else if fail_num > 0 {
        notify_title = "Error";
        let failed_list = failed_dirs
            .iter()
            .map(|(dir, err)| format!("{}: {}", dir.to_string_lossy(), err))
            .collect::<Vec<_>>()
            .join("\n");
        notify_message = format!(
            "Pulled {} repositories successfully, but failed to pull the following {} repositories:\n{}",
            success_num, fail_num, failed_list
        );
    } else {
        notify_title = "Successful";
        notify_message = format!("All {} repositories were pulled successfully", total);
    }

    let _ = Command::new("notify-send")
        .arg(notify_title)
        .arg(&notify_message)
        .status()
        .await;

    Ok(())
}
