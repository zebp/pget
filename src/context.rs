use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use reqwest::Url;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Context {
    link_store: Arc<Mutex<LinkStore>>,
    /// The directory where all files are stored.
    pub output_directory: PathBuf,
}

impl Context {
    /// Reads all the links from either the provided file or from stdin.
    pub fn new(
        output_directory: &Option<String>,
        links_file: &Option<String>,
    ) -> std::io::Result<Self> {
        let link_store = LinkStore::new(links_file)?;
        Ok(Self {
            link_store: Arc::new(Mutex::new(link_store)),
            output_directory: output_directory
                .as_ref()
                .map(|v| Path::new(&v).into())
                .unwrap_or_else(|| Path::new("./").into()),
        })
    }

    pub async fn next(&self) -> Option<(PathBuf, Url)> {
        let mut store = self.link_store.lock().await;
        store.links.pop().map(|url| {
            let name = store.choose_name(&url);
            let output_path = self.output_directory.join(name);
            (output_path, url)
        })
    }
}

#[derive(Debug, Default)]
pub struct LinkStore {
    links: Vec<Url>,
    /// By default, like wget, we name the files the rest of the url after the final slash but this
    /// create collisions so we keep track of the names and append the file's count to stop that.
    file_name_count_map: HashMap<String, usize>,
}

impl LinkStore {
    pub fn new(links_file: &Option<String>) -> std::io::Result<Self> {
        let mut links = Vec::new();
        links_file
            .as_ref()
            .map(std::fs::read_to_string)
            .unwrap_or_else(|| {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                Ok(buf)
            })?
            .lines()
            .for_each(|line| match Url::parse(line) {
                Ok(link) => links.push(link),
                Err(_) => eprintln!("{} is not a valid url", line),
            });

        Ok(Self {
            links,
            file_name_count_map: HashMap::default(),
        })
    }

    fn choose_name(&mut self, link: &Url) -> String {
        let last = link
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|last| if last.is_empty() { None } else { Some(last) })
            .unwrap_or("index.html");

        let occurance = *self
            .file_name_count_map
            .entry(last.into())
            .and_modify(|x| *x += 1)
            .or_insert(0);

        if occurance > 0 {
            // TODO: Maybe preserve the file extension?
            format!("{}.{}", last, occurance)
        } else {
            last.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_with_extensions() {
        let url = Url::parse("https://example.com/test.png").unwrap();
        let mut store = LinkStore::default();

        assert_eq!(store.choose_name(&url), "test.png");
        assert_eq!(store.choose_name(&url), "test.png.1");
    }

    #[test]
    fn multiple_without_extensions() {
        let url = Url::parse("https://example.com/").unwrap();
        let mut store = LinkStore::default();

        assert_eq!(store.choose_name(&url), "index.html");
        assert_eq!(store.choose_name(&url), "index.html.1");
    }
}
