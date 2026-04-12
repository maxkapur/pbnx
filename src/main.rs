use feedparser_rs::{Entry, parse};
use html_parser::Dom;
use platform_dirs::AppDirs;
use reqwest::{self, Error as ReqwestError};
use std::fs::{read_to_string, write};

const FEED_URL: &str = "https://manuelmoreale.com/feed/peopleandblogs";

fn main() {
    let feed_contents = get_feed_contents().unwrap();
    let feed = parse(feed_contents.as_bytes()).unwrap();
    let doms = feed.entries.iter().filter_map(extract_dom);
    for dom in doms {
        for p in dom.children {
            let Some(element) = p.element() else {
                continue;
            };
            if element.name != "p" {
                continue;
            };
            // NOTE: p is a Node, which is an enum of Element, Text, or Comment;
            // can't have Some(text) here since we already established that we
            // are dealing with an Element. Instead, there will be a Text instance
            // among the children which is what we'll actually work with
            let Some(text) = p.text() else { continue };
            println!("{}", text);
        }
    }
}

fn get_feed_cache_path() -> std::path::PathBuf {
    let app_dirs = AppDirs::new(Some("pnbx"), true).unwrap();
    app_dirs.config_dir.join("config-file")
}

fn get_feed_contents() -> Result<String, ReqwestError> {
    let feed_cache_path = get_feed_cache_path();

    if let Ok(feed_data) = read_to_string(feed_cache_path.clone()) {
        return Ok(feed_data);
    }
    let body = reqwest::blocking::get(FEED_URL).unwrap().text().unwrap();
    write(feed_cache_path, body.clone()).unwrap();
    Ok(body)
}

fn extract_dom(feed_entry: &Entry) -> Option<Dom> {
    feed_entry
        .summary_detail
        .as_ref()
        .and_then(|d| Dom::parse(&d.value).ok())
}
