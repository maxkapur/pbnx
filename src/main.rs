mod constants;
mod feed;
mod pagerank;

fn main() {
    let xrefs = feed::people_and_blogs_xrefs();
    let pageranks = pagerank::pagerank(xrefs);

    // Format PageRank results as a CSV
    println!("rank,url,probability");
    pageranks
        .iter()
        .for_each(|(rank, url, probability)| println!("{},{},{}", rank, url, probability));
}
