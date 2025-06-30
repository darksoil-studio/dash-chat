use std::time::Duration;

use anyhow::anyhow;

pub async fn with_retries<T>(
    condition: impl AsyncFn() -> anyhow::Result<T>,
    retries: usize,
) -> anyhow::Result<T> {
    let mut retry_count = 0;
    loop {
        let response = condition().await;

        match response {
            Ok(r) => {
                return Ok(r);
            }
            Err(err) => {
                log::warn!("Condition not met yet: {err:?} Retrying in 1s.");
                std::thread::sleep(Duration::from_secs(1));

                retry_count += 1;
                if retry_count == retries {
                    return Err(anyhow!("Timeout. Last error: {err:?}"));
                }
            }
        }
    }
}
