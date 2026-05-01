use ndarray_linalg::Eig;
use std::{
    collections::{HashMap, HashSet},
    ops::DivAssign,
};

use crate::constants::DAMPING_FACTOR;

use ndarray::*;

/// Run the PageRank algorithm on the given map of cross references.
pub fn pagerank(xrefs: HashMap<String, HashSet<String>>) -> Vec<(usize, String, f64)> {
    let (xrefs_idx, idx2url) = indexify(&xrefs);
    let markov_array = construct_markov_array(xrefs_idx, DAMPING_FACTOR);
    let stationary_dist = compute_stationary_distribution(markov_array);
    let mut with_urls: Vec<(String, f64)> = stationary_dist
        .iter()
        .enumerate()
        .map(|(i, &x)| (idx2url[i].clone(), x))
        .collect();
    with_urls
        .sort_unstable_by(|(_, x), (_, y)| y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal));
    let pageranks: Vec<(usize, String, f64)> = with_urls
        .iter()
        .enumerate()
        .map(|(rank, (url, probability))| (rank + 1, url.clone(), *probability))
        .collect();
    pageranks
}

/// Transform `String -> Vec<String>` mapping into a `usize -> HashSet<usize>` mapping
/// to ease array operations. Also provide a Vec<String> to associate each URL with its
/// numerical index.
fn indexify(xrefs: &HashMap<String, HashSet<String>>) -> (Vec<HashSet<usize>>, Vec<String>) {
    let idx2url: Vec<String> = xrefs.keys().cloned().collect();
    let url2idx: HashMap<String, usize> = idx2url
        .iter()
        .enumerate()
        .map(|(i, url)| (url.clone(), i))
        .collect();

    let xrefs_idx: Vec<HashSet<usize>> = idx2url
        .iter()
        .map(|src_url| {
            xrefs[&src_url.clone()]
                .iter()
                .map(|dest_url| url2idx[dest_url])
                .collect()
        })
        .collect();
    (xrefs_idx, idx2url)
}

/// Construct a Markov array where the `(j, i)` entry gives the probability of
/// traveling from page `i` to `j`. The `damping_factor` determines the
/// probability of a random jump across the graph.
fn construct_markov_array(
    xrefs_idx: Vec<HashSet<usize>>,
    damping_factor: f64,
) -> ArrayBase<OwnedRepr<f64>, Dim<[usize; 2]>, f64> {
    let n = xrefs_idx.len();
    let d = damping_factor / (n as f64);
    let nlinks: Vec<usize> = xrefs_idx.iter().map(|s| s.len()).collect();
    let c: Vec<f64> = nlinks
        .iter()
        .map(|&m| (1.0 - damping_factor) / (m as f64))
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
    markov_array
}

/// Compute the stationary distribution of the Markov matrix.
fn compute_stationary_distribution(
    markov_array: ArrayBase<OwnedRepr<f64>, Dim<[usize; 2]>, f64>,
) -> ArrayBase<OwnedRepr<f64>, Dim<[usize; 1]>, f64> {
    let (eig, vecs) = markov_array.eig().unwrap();

    // Get index of eigenvalue closest to Complex(1, 0)
    let (idx, should_be_one) = eig
        .iter()
        .enumerate()
        .min_by(|(_, x), (_, y)| {
            let dx = (x.re - 1.0).hypot(x.im);
            let dy = (y.re - 1.0).hypot(y.im);
            dx.partial_cmp(&dy).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();
    assert!((should_be_one.re - 1.0).hypot(should_be_one.im) < 1e-8);

    // Get the real part of the corresponding vector and assert stationarity
    // properties
    let complex_dist = vecs.column(idx);
    assert!(complex_dist.iter().all(|c| c.im < 1e-8));

    let mut real_dist = complex_dist.map(|c| c.re);
    real_dist.div_assign(real_dist.sum());
    assert!(real_dist.iter().all(|&c| 0.0 < c && c <= 1.0));

    let should_be_zeros = markov_array.dot(&real_dist) - &real_dist;
    assert!(should_be_zeros.iter().all(|x| x.abs() < 1e-8));
    real_dist
}
