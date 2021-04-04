use std::{path::Path, time::Instant};

use colored::*;
use futures::StreamExt;
use reqwest::Url;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{context::Context, error::Error};

pub async fn download(ctx: &Context, link: &Url, path: &Path) -> Result<(), Error> {
    let mut file = File::create(path).await?;
    let mut stream = ctx.client.get(link.clone()).send().await?.bytes_stream();

    let then = Instant::now();
    let mut total_bytes = 0;

    while let Some(item) = stream.next().await {
        let bytes = item?;
        file.write(&bytes).await?;
        total_bytes += bytes.len();
    }

    let duration = Instant::now().duration_since(then);

    let bytes_per_second = total_bytes as f64 / duration.as_secs_f64();
    let bytes_per_second = bytes_per_second.round() as u64;

    println!(
        "Downloaded {} totaling {} bytes at {}{}",
        &link.to_string().green(),
        bytesize::to_string(total_bytes as u64, false).green(),
        bytesize::to_string(bytes_per_second, false).green(),
        "/s".green()
    );

    Ok(())
}
