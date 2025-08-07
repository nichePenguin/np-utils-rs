use tokio::time::{Duration, sleep};

use std::{
    error::Error,
    path::PathBuf,
    fs
};

pub fn get_env_var(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        log::warn!("Couldn't find \"{}\" env variable, defaulting to {}", key, fallback);
        fallback.to_owned()
    })
}

pub fn log_line(
    path: &PathBuf,
    data: String,
    max_lines: usize) -> Result<(), Box<dyn Error>> 
{
    let file = fs::read_to_string(path)?;
    let lines = file.lines();
    let input_lines = data.lines().collect::<Vec<_>>();
    let input_count = input_lines.len();
    if input_count > max_lines {
        return Err(format!("Attempt to write more than {} lines to {}", input_count, path.display()).into());
    }
    let to_write = lines.rev().take(max_lines - input_count).collect::<Vec<&str>>();
    let to_write = to_write.into_iter().rev().chain(input_lines.into_iter());
    fs::write(path, to_write.collect::<Vec<&str>>().join("\n"))?;
    Ok(())
}

pub fn file_watch<F>(
    path: PathBuf,
    period_ms: u64,
    handler: F) -> tokio::task::JoinHandle<()>
where
    F: Fn(String) + Send + Sync + 'static
{
    let mut last_hash = String::new(); 
    tokio::spawn(async move { loop {
        sleep(Duration::from_millis(period_ms)).await;
        let new_hash = sha256::try_digest(&path);
        if new_hash.is_err() {
            log::error!("Failed to hash {}", &path.display());
            continue;
        }
        let new_hash = new_hash.unwrap();
        if last_hash != new_hash {
            let file_data = fs::read_to_string(&path);
            if file_data.is_err() {
                log::error!("Failed to read file {}: {}", &path.display(), file_data.err().unwrap());
                continue;
            }
            handler(file_data.unwrap());
            last_hash = new_hash;
            log::debug!("Updated hash for file {} to {}", path.display(), last_hash);
        } else {
            log::trace!("File {} did not change", path.display());
        }
    }})
}
