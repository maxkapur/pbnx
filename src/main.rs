use feedparser_rs::{Entry, parse};
use ndarray_linalg::Eig;
use platform_dirs::AppDirs;
use reqwest::{self, Error as ReqwestError, Url};
use scraper::{Html, Selector};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, create_dir_all, read_to_string, write},
    ops::Div,
};

use ndarray::*;

const FEED_URL: &str = "https://manuelmoreale.com/feed/peopleandblogs";
const DAMPING_FACTOR: f64 = 0.15;
const CACHE_LIFETIME_SEC: u64 = 3600 * 24 * 7;

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

    // Canonically order URLs alphabetically
    let mut idx2url: Vec<String> = xrefs.keys().cloned().collect();
    idx2url.sort();
    let idx2url = idx2url;

    let url2idx: HashMap<String, usize> = idx2url
        .iter()
        .enumerate()
        .map(|(i, url)| (url.clone(), i))
        .collect();

    let n = xrefs.len();

    let xrefs_idx: Vec<HashSet<usize>> = idx2url
        .iter()
        .map(|src_url| {
            xrefs[&src_url.clone()]
                .iter()
                .map(|dest_url| url2idx[dest_url])
                .collect()
        })
        .collect();

    let d = DAMPING_FACTOR / (n as f64);
    let nlinks: Vec<usize> = xrefs_idx.iter().map(|s| s.len()).collect();
    let c: Vec<f64> = nlinks
        .iter()
        .map(|&m| (1.0 - DAMPING_FACTOR) / (m as f64))
        .collect();
    let one_over_n = 1.0 / (n as f64);

    let markov_array: Array2<f64> = Array2::from_shape_fn((n, n), |(j, i)| {
        if xrefs_idx[i].is_empty() {
            // Then c[i] is +inf and column sum will be less than 1. Just assign
            // equal probability to every transition
            one_over_n
        } else if xrefs_idx[i].contains(&j) {
            d + c[i]
        } else {
            d
        }
    });

    let col_sum = markov_array.sum_axis(Axis(0));
    assert!(col_sum.iter().all(|&x| (x - 1.0).abs() < 1e-8));

    let (eig, vecs) = markov_array.eig().unwrap();

    // index of eigenvalue closest to Complex(1, 0)
    let idx = eig
        .iter()
        .enumerate()
        .max_by(|(_, x), (_, y)| {
            let dx = (x.re - 1.0).hypot(x.im);
            let dy = (y.re - 1.0).hypot(y.im);
            dx.partial_cmp(&dy).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap();

    // extract the stationary distribution
    let stationary_dist = vecs.column(idx).div(vecs.column(idx).sum());
    // assert!(stationary_dist.iter().all(|c| c.im < 1e-8));
    // assert!(stationary_dist.iter().all(|c| 0.0 <= c.re && c.re <= 1.0));

    let stationary_dist = stationary_dist.map(|c| c.re);

    let mut with_keys: Vec<(String, f64)> = stationary_dist
        .iter()
        .enumerate()
        .map(|(i, &x)| (idx2url[i].clone(), x))
        .collect();
    with_keys
        .sort_unstable_by(|(_, x), (_, y)| x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal));

    dbg!(with_keys);
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
