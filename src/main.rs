use reqwest;
use rss::Channel;
use std::{env};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use urlencoding::encode;

async fn fetch_rss_feed(category_id: i32, query: &str) -> Result<String, Box<dyn Error>> {
    const HOST: &str = "http://truenas:9117";
    const ROUTE: &str = "/api/v2.0/indexers/torrentleech/results/torznab/api";
    const API_KEY: &str = "ccxkrk47v1cy9qar2jf598n0ahrkm3a2";

    let encoded_query = encode(query);
    let full_url = format!("{}{}?apikey={}&t=search&cat={}&q={}", HOST, ROUTE, API_KEY, category_id, encoded_query);

    let resp = reqwest::get(&full_url).await?.text().await?;
    Ok(resp)
}

async fn process_rss_feed(category_id: i32, query: &str) -> Result<(), Box<dyn Error>> {
    let content = fetch_rss_feed(category_id, query).await?;
    let channel = Channel::read_from(content.as_bytes())?;

    for item in channel.items() {
        println!("Title: {}", item.title().unwrap_or("No title"));
        println!("Link: {}", item.link().unwrap_or("No link"));
        println!();
    }

    let mut sorted_items = channel.items;
    sorted_items.sort_by(|a,b| b.pub_date.cmp(&a.pub_date));

    let item = sorted_items.first().unwrap();
    let title: &str = item.title.as_ref().unwrap();
    let link: &str = item.link.as_ref().unwrap();

    //println!("Downloading '{}' via {}", title, link);
    let output_file = friendly_name(title) + ".torrent";
    download_file(link, &output_file).await
}

async fn download_file(url: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(name)?;

    let response = reqwest::get(url).await?;
    let file_bytes = response.bytes().await; //.bytes().await?;

    let file_result = file.write(file_bytes.unwrap().iter().as_ref());

    println!("File write result: {}", file_result.unwrap());

    Ok(())
}

fn friendly_name(filename: &str) -> String {
    return filename
        .replace(" ", ".")
        .replace("'", "");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: rss_reader <CATEGORY_ID> <QUERY>");
        std::process::exit(1);
    }

    let category_id: i32 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("CATEGORY_ID must be an integer.");
        std::process::exit(1);
    });
    let query = &args[2];

    process_rss_feed(category_id, query).await
}