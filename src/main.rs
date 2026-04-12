use platform_dirs::AppDirs;
use reqwest::{self, Error as ReqwestError};
use std::fs::{read_to_string, write};

const FEED_URL: &str = "https://manuelmoreale.com/feed/peopleandblogs";

fn main() {
    println!("Hello, world!");
    dbg!(get_feed_cache_path());
    let feed_contents = get_feed_contents().unwrap();
    dbg!(feed_contents);
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

fn get_feed_cache_path() -> std::path::PathBuf {
    let app_dirs = AppDirs::new(Some("pnbx"), true).unwrap();
    app_dirs.config_dir.join("config-file")
}
