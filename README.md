# pbnx

A Rust program that scrapes the [People and Blogs](https://peopleandblogs.com/)
RSS feed, identifies cross-references between the interviews, and runs the
infamous [PageRank](https://en.wikipedia.org/wiki/PageRank) algorithm on the
result. This gives a coarse idea of which blogs are more or less popular.

## FAQ

**What do I do with this?** Clone, `cargo run > out.csv`, then do whatever you'd
like with the results.

**Isn't ranking blogs by popularity against the spirit of People and Blogs?**
Very much so. I'm sorry, Manu. To redeem myself, I will refrain from printing
the output of the program here. If you are curious, you'll have to clone it and
run it yourself. The purpose of this repo was to learn more about the PageRank
algorithm and exercise some classic Rust libraries: reqwests, ndarray.

**Where does [your blog](https://maxkapur.com) rank?** Tied with 60 others for
last place.
