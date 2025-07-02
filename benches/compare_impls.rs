//! benches
#![allow(missing_docs, reason = "inner doc in external macro lib")]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use regex::Regex;
use regextrie::RegexTrie;
use std::error::Error;
use std::hint::black_box;

/// Naive implementation
struct Naive {
    /// Stores the original pattern string and its compiled DFA, with an
    /// optional score
    compiled_patterns: Vec<(String, Regex, usize)>,
}

impl Naive {
    /// From a collection
    pub fn from(patterns: &[String]) -> Result<Self, Box<dyn Error>> {
        let mut compiled_patterns = Vec::with_capacity(patterns.len());

        for pattern in patterns {
            let dfa = Regex::new(pattern)?;
            compiled_patterns.push((pattern.clone(), dfa, pattern.len()));
        }

        Ok(Self { compiled_patterns })
    }

    /// Find best match naively
    pub fn find_best_match(&self, query: &str) -> Option<String> {
        let mut best_match = None;
        for (pattern, regex, score) in &self.compiled_patterns {
            if regex.is_match(query) {
                match best_match {
                    Some((_, best_score)) => {
                        if score < best_score {
                            best_match = Some((pattern, score));
                        }
                    }
                    None => best_match = Some((pattern, score)),
                }
            }
        }

        best_match.map(|(pattern, _)| pattern.clone())
    }
}

/// Bench all implementation
fn bench_best_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("best_match");

    // Measure behaviour at different scales.
    for &size in &[10_usize, 1_000, 10_000] {
        // Build a deterministic corpus *once* per size so setup cost isn’t timed.
        let corpus: Vec<String> = (0..size).map(|i| format!("test{i}")).collect();
        let query = "test42";

        // Own a private copy for each algorithm so they can mutate freely.
        let naive = Naive::from(&corpus).expect("can't init Naive");
        // let bin = BinarySearchIndex::from(corpus.clone());
        let trie = RegexTrie::from(&corpus).expect("can't init RegexTrie");

        group.bench_with_input(
            BenchmarkId::new("Naive", size),
            &naive,
            |bencher, matcher| bencher.iter(|| black_box(matcher.find_best_match(query))),
        );

        // group.bench_with_input(BenchmarkId::new("Binary", size), &bin, |b, bidx| {
        //     b.iter(|| black_box(bidx.find_best_match(query)))
        // });

        group.bench_with_input(
            BenchmarkId::new("RegexTrie", size),
            &trie,
            |bencher, matcher| bencher.iter(|| black_box(matcher.find_best_match(query))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_best_match);
criterion_main!(benches);
