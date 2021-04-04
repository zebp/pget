use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
};

use reqwest::Url;

#[derive(Debug, Clone)]
pub struct Context {
    links: Vec<Url>,
    /// By default, like wget, we name the files the rest of the url after the final slash but this
    /// create collisions so we keep track of the names and append the file's count to stop that.
    file_name_count_map: HashMap<String, usize>,
    /// The directory where all files are stored.
    output_directory: PathBuf,
}

impl Context {
    /// Reads all the links from either the provided file or from stdin.
    pub fn new(
        output_directory: &Option<String>,
        links_file: &Option<String>,
    ) -> std::io::Result<Self> {
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
            output_directory: output_directory
                .as_ref()
                .map(|v| Path::new(&v).into())
                .unwrap_or_else(|| Path::new("./").into()),
            file_name_count_map: HashMap::default(),
        })
    }

    pub fn next(&mut self) -> Option<(String, Url)> {
        self.links.pop()
            .map(|url| (self.choose_name(&url), url))
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
    use std::{collections::HashMap, path::Path};

    #[test]
    fn multiple_with_extensions() {
        let url = Url::parse("https://example.com/test.png").unwrap();
        let mut ctx = Context {
            links: Vec::new(),
            file_name_count_map: HashMap::default(),
            output_directory: Path::new("./").into(),
        };

        assert_eq!(ctx.choose_name(&url), "test.png");
        assert_eq!(ctx.choose_name(&url), "test.png.1");
    }

    #[test]
    fn multiple_without_extensions() {
        let url = Url::parse("https://example.com/").unwrap();
        let mut ctx = Context {
            links: Vec::new(),
            file_name_count_map: HashMap::default(),
            output_directory: Path::new("./").into(),
        };

        assert_eq!(ctx.choose_name(&url), "index.html");
        assert_eq!(ctx.choose_name(&url), "index.html.1");
    }
}
