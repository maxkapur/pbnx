mod constants;
mod feed;
mod pagerank;

fn main() {
    let xrefs = feed::collect_xrefs();
    let pageranks = pagerank::pagerank(xrefs);
    println!("rank,url,probability");
    pageranks
        .iter()
        .for_each(|(rank, url, probability)| println!("{},{},{}", rank, url, probability));
}
