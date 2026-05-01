use feedparser_rs::{Entry, parse};
use platform_dirs::AppDirs;
use reqwest::{self, Error as ReqwestError, Url};
use scraper::{Html, Selector};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, create_dir_all, read_to_string, write},
};

use crate::constants::{CACHE_LIFETIME_SEC, FEED_URL};

/// Parse the P&B feed and collect all cross references between interviews.
pub fn people_and_blogs_xrefs() -> HashMap<String, HashSet<String>> {
    let feed_contents = get_feed_contents().unwrap();
    let feed = parse(feed_contents.as_bytes()).unwrap();
    let documents: HashMap<String, Html> = feed
        .entries
        .iter()
        .filter_map(extract_document)
        .filter_map(|document| extract_blog_url(&document).map(|url| (url, document)))
        .collect();

    documents
        .iter()
        .map(|(my_url, document)| {
            let a_selector = Selector::parse("a").unwrap();
            // Use a HashSet to capture only unique references
            let xrefs: HashSet<String> = document
                .select(&a_selector)
                .filter_map(|a| {
                    // Extract references to blogs other than the interviewee's
                    let ref_url = a.value().attr("href").map(extract_domain)??;
                    if documents.contains_key(&ref_url) && &ref_url != my_url {
                        return Some(ref_url);
                    }
                    None
                })
                .collect();
            (my_url.clone(), xrefs)
        })
        .collect()
}

fn get_feed_cache_path() -> std::path::PathBuf {
    AppDirs::new(Some("pnbx"), true)
        .unwrap()
        .config_dir
        .join("config-file")
}

fn is_fresh(feed_cache_path: &std::path::Path) -> bool {
    let fresh = || -> Option<bool> {
        let elapsed = fs::metadata(feed_cache_path)
            .ok()?
            .modified()
            .ok()?
            .elapsed()
            .ok()?;
        Some(elapsed.as_secs() <= CACHE_LIFETIME_SEC)
    };
    fresh().unwrap_or(false)
}

/// Get the contents of the P&B feed, either by reading from disk cache or HTTP
/// request if fresh cache doesn't exist.
fn get_feed_contents() -> Result<String, ReqwestError> {
    let feed_cache_path = get_feed_cache_path();
    if is_fresh(feed_cache_path.as_path())
        && let Ok(feed_data) = read_to_string(feed_cache_path.clone())
    {
        return Ok(feed_data);
    }
    let body = reqwest::blocking::get(FEED_URL)?.text()?;
    create_dir_all(feed_cache_path.parent().unwrap()).unwrap();
    write(feed_cache_path, body.clone()).unwrap();
    Ok(body)
}

fn extract_document(feed_entry: &Entry) -> Option<Html> {
    feed_entry
        .summary_detail
        .as_ref()
        .map(|d| Html::parse_fragment(&d.value))
}

/// Extract the interviewee's blog URL from their interview.
fn extract_blog_url(document: &Html) -> Option<String> {
    let p_selector = Selector::parse("p").unwrap();
    let p = document.select(&p_selector).next()?;
    if !p.inner_html().contains("whose blog can be found at") {
        return None;
    };
    let a_selector = Selector::parse("a").unwrap();
    let a = p.select(&a_selector).next()?;
    let raw_url = a.value().attr("href")?;
    extract_domain(raw_url)
}

fn extract_domain(raw_url: &str) -> Option<String> {
    Some(
        Url::parse(raw_url)
            .ok()?
            .domain()?
            .to_string()
            // Clean up
            .replace("www.", "")
            .to_lowercase(),
    )
}
