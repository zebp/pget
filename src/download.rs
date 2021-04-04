use std::path::PathBuf;

use futures::StreamExt;
use reqwest::Url;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{context::Context, error::Error};

pub async fn download(ctx: &Context, link: Url, path: PathBuf) -> Result<(), Error> {
    let mut file = File::create(path).await?;
    let mut stream = ctx.client.get(link).send().await?.bytes_stream();

    while let Some(item) = stream.next().await {
        let bytes = item?;
        file.write(&bytes).await?;
    }

    Ok(())
}
