# RegexTrie

**RegexTrie** is a high-performance Rust library designed for matching a large number of regular expressions against a given input string. It uses a trie data structure to store and organize regex patterns, enabling significantly faster matching compared to naive iterative approaches. In some cases, `RegexTrie` can be over **400 times faster**.

This library is ideal for applications that need to check an input string against a large and dynamic set of regex patterns, such as:

*   **URL routing and filtering:** Quickly matching incoming request URLs against a list of registered routes.
*   **Content filtering and moderation:** Scanning text for a large set of keywords or patterns.
*   **Security threat detection:** Identifying malicious patterns in network traffic or log files.

## Key Features

*   **High Performance:** Utilizes a trie-based indexing strategy to dramatically reduce the number of regex comparisons needed, leading to significant speedups.
*   **Ergonomic API:** A simple and intuitive API for creating, populating, and querying the `RegexTrie`.
*   **Customizable Matching Logic:** Supports custom scoring functions to define what constitutes the "best" match when multiple patterns are found.
*   **Parallelized Regex Compilation:** Leverages Rayon to compile regex patterns in parallel, speeding up the initial setup time.
*   **Error Handling:** Provides clear error handling for invalid regex patterns.

## How It Works

`RegexTrie` builds a trie from the literal prefixes of the regex patterns. When you search for matches, it traverses the trie based on the input string. This quickly narrows down the set of potential regex candidates. Only the patterns that could possibly match the input are then evaluated, avoiding the need to test every single regex.

For example, if you have the patterns `https://google.com/.*` and `https://yahoo.com/.*`, the trie will have a common path for `https://`. When you search for `https://google.com/page`, `RegexTrie` will only evaluate the first pattern, completely skipping the second one.

## Installation

Add `RegexTrie` to your `Cargo.toml`:

```toml
[dependencies]
regextrie = "0.1.0" # Replace with the latest version
```

## Usage

### Basic Example: Finding All Matches

```rust
use regextrie::RegexTrie;

fn main() {
    // Create a new RegexTrie.
    let mut trie = RegexTrie::new();

    // Insert regex patterns.
    trie.insert("hello.*").unwrap();
    trie.insert("hello[a-z]+test").unwrap();
    trie.insert("anotherpattern").unwrap();

    // Find all matches for a given input.
    let input = "helloabctest";
    let matches = trie.find_matches(input);

    println!("Found matching patterns for '{}': {:?}", input, matches);
    // Output: Found matching patterns for 'helloabctest': ["helloabctest", "hello.*"]
}
```

### Finding the Best Match

By default, `RegexTrie` considers the shortest pattern to be the "best" match.

```rust
use regextrie::RegexTrie;

fn main() {
    let patterns = vec![
        "a.*".to_string(),
        "a[0-9]+b.*".to_string(),
    ];
    let trie = RegexTrie::from(&patterns).unwrap();

    let best_match = trie.find_best_match("a123bbb");
    assert_eq!(best_match, Some("a.*".to_string()));
}
```

### Custom Scoring for Best Match

You can provide a custom scoring function to `RegexTrie` to define your own logic for what makes a match "best". The match with the lowest score will be chosen.

```rust
use regextrie::RegexTrie;
use url::Url;

fn main() {
    let patterns = vec![
        "https://www.google.com/[a-zA-Z0-9/-]+".to_string(),
        "https://www.google.com/.*/test/.*".to_string(),
    ];

    // Custom scorer that prioritizes patterns with more URL path segments.
    let trie = RegexTrie::from_with_scorer(
        &patterns,
        Box::new(|pattern: &str, _| {
            Url::parse(pattern)
                .map_or(pattern.len(), |url| {
                    url.path_segments().map_or(pattern.len(), |s| s.count())
                })
        }),
    ).unwrap();

    let best_match = trie.find_best_match("https://www.google.com/foo/test/bar");
    assert_eq!(best_match, Some("https://www.google.com/.*/test/.*".to_string()));
}
```

## Benchmarks

`RegexTrie` shows significant performance improvements over a naive implementation that iterates through a list of regexes.

Here are some benchmark results:

| Benchmark                 | Naive Implementation | RegexTrie      | Speedup |
| ------------------------- | -------------------- | -------------- | ------- |
| Small Set (10 patterns)   | ~1.5 µs              | ~0.5 µs        | ~3x     |
| Medium Set (1k patterns)  | ~150 µs              | ~0.5 µs        | ~300x   |
| Large Set (10k patterns)  | ~1.5 ms              | ~0.5 µs        | ~3000x  |
| Real-world URL patterns   | ~2.5 ms              | ~6 µs          | ~416x   |

*Benchmarks were run on a standard laptop. Your results may vary.*

More in [bench section](benches/readme.md)

## API Reference

The main components of the `RegexTrie` API are:

*   `RegexTrie::new()`: Creates a new, empty `RegexTrie`.
*   `RegexTrie::from(patterns: &[String])`: Creates a new `RegexTrie` from a list of patterns.
*   `RegexTrie::from_with_scorer(patterns: &[String], scorer: ScorerFuncType)`: Creates a new `RegexTrie` with a custom scorer.
*   `insert(&mut self, pattern: &str)`: Inserts a new regex pattern.
*   `insert_many(&mut self, patterns: &[String])`: Inserts multiple patterns at once.
*   `find_matches(&self, input: &str) -> Vec<String>`: Finds all patterns that match the input.
*   `find_best_match(&self, input: &str) -> Option<String>`: Finds the best matching pattern based on the scoring logic.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
