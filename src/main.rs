mod download;
mod context;
mod error;

use std::sync::Arc;

use argh::FromArgs;
use tokio::task::JoinHandle;

use crate::context::Context;

#[derive(Debug, FromArgs)]
/// Downloading things in parallel.
struct Pget {
    /// how many links should be downloaded in parallel.
    #[argh(option, default = "8", short = 't')]
    tasks: u16,
    /// how many links should be downloaded in parallel.
    #[argh(option, short = 'o')]
    output: Option<String>,
    #[argh(positional)]
    /// a list of all of the links to download.
    links: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = argh::from_env::<Pget>();
    let ctx = Context::new(&args.output, &args.links).unwrap();
    let ctx = Arc::new(ctx);

    let tasks: Vec<JoinHandle<()>> = std::iter::repeat_with(|| {
        let ctx = ctx.clone();
        tokio::spawn(async move {
            loop {
                // Temporarily block to try to get a link or return if we downloaded everything.
                let (path, link) = match { ctx.next().await } {
                    Some(pair) => pair,
                    None => return,
                };

                match download::download(link, path).await {
                    Ok(_) => {},
                    Err(e) => {
                        dbg!(e);
                    }
                }
            }
        })
    })
    .take(args.tasks as usize)
    .collect();

    futures::future::join_all(tasks).await;
}
