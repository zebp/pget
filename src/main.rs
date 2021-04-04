use std::io::Read;

use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Downloading things in parallel.
struct Pget {
    /// how many links should be downloaded in parallel.
    #[argh(option, default = "8", short = 't')]
    tasks: u16,
    #[argh(positional)]
    /// a list of all of the links to download.
    links: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = argh::from_env::<Pget>();
    let links = collect_links(&args.links).unwrap();
    dbg!(links, args);
}

/// Reads all the links from either the provided file or from stdin.
fn collect_links(links_file: &Option<String>) -> std::io::Result<Vec<String>> {
    links_file
        .as_ref()
        .map(std::fs::read_to_string)
        .unwrap_or_else(|| {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        })
        .map(|input| input.lines().map(String::from).collect())
}
