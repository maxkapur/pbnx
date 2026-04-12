use feedparser_rs::{Entry, parse};
use platform_dirs::AppDirs;
use reqwest::{self, Error as ReqwestError};
use scraper::{Html, Selector};
use std::{
    collections::HashMap,
    fs::{read_to_string, write},
};

const FEED_URL: &str = "https://manuelmoreale.com/feed/peopleandblogs";

fn main() {
    let feed_contents = get_feed_contents().unwrap();
    let feed = parse(feed_contents.as_bytes()).unwrap();
    let documents: HashMap<String, Html> = feed
        .entries
        .iter()
        .filter_map(extract_document)
        .filter_map(|document| extract_blog_url(&document).map(|url| (url, document)))
        .collect();

    dbg!(documents);
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

fn extract_document(feed_entry: &Entry) -> Option<Html> {
    feed_entry
        .summary_detail
        .as_ref()
        .and_then(|d| Some(Html::parse_fragment(&d.value)))
}

fn extract_blog_url(document: &Html) -> Option<String> {
    let p_selector = Selector::parse("p").unwrap();
    let p = document.select(&p_selector).next()?;
    if !p.inner_html().contains("whose blog can be found at") {
        return None;
    };
    let a_selector = Selector::parse("a").unwrap();
    let a = p.select(&a_selector).next()?;
    let raw_url = a.value().attr("href")?;
    Some(canonicalize(&raw_url))
}

fn canonicalize(url: &str) -> String {
    let url = url.replace("www.", "");
    let url = url.strip_prefix("http://").unwrap_or(&url);
    let url = url.strip_prefix("https://").unwrap_or(&url);
    let url = url.strip_suffix("/").unwrap_or(&url);
    url.to_string()
}
