use feedparser_rs::{Entry, parse};
use platform_dirs::AppDirs;
use reqwest::{self, Error as ReqwestError, Url};
use scraper::{Html, Selector};
use std::{
    collections::{HashMap, HashSet},
    fs::{read_to_string, write},
};

use ndarray::*;

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

    // Track which interviews reference other interviews
    let xrefs: HashMap<String, Vec<String>> = documents
        .iter()
        .map(|(my_url, document)| {
            let a_selector = Selector::parse("a").unwrap();
            // Use a HashSet to capture only unique references
            let refs: HashSet<String> = document
                .select(&a_selector)
                .filter_map(|a| {
                    let ref_url = a.value().attr("href").map(extract_domain)??;
                    if documents.contains_key(&ref_url) && &ref_url != my_url {
                        return Some(ref_url);
                    }
                    None
                })
                .collect();
            (my_url.clone(), refs.into_iter().collect())
        })
        .collect();

    let idx2url: Vec<String> = xrefs.keys().cloned().collect();
    let url2idx: HashMap<String, usize> = idx2url
        .iter()
        .enumerate()
        .map(|(i, url)| (url.clone(), i))
        .collect();

    let n = xrefs.len();
    let mut array: Array2<f64> = Array2::zeros((n, n));

    xrefs.iter().for_each(|(src_url, dest_urls)| {
        let &j = url2idx.get(src_url).unwrap();

        dest_urls.iter().for_each(|dest_url| {
            let &i = url2idx.get(dest_url).unwrap();
            array[[i, j]] = 1.0;
        })
    });

    let sum = array.sum_axis(Axis(0));

    dbg!(array);
    dbg!(sum);
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
        .map(|d| Html::parse_fragment(&d.value))
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
