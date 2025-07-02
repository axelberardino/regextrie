//! regextrie main func
use regextrie::RegexTrie;

#[expect(
    clippy::too_many_lines,
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "..."
)]
fn main() {
    // Create a new RegexTrie.
    let mut trie = RegexTrie::new();

    // Insert the regex patterns. Note that insert can now fail if the regex is invalid.
    let patterns = vec![
        "something[0-9]+",
        "hello.*",
        "hello.*test",
        "hello[a-z]+test",
        "anotherpattern",
        ".*test", // A pattern with no literal prefix
    ];

    println!("Inserting patterns...");
    for pattern in patterns {
        if let Err(e) = trie.insert(pattern) {
            eprintln!("Failed to insert pattern '{pattern}': {e}");
        }
    }

    // Define an input string to test.
    let input = "helloabctest";
    println!("\n---------------------------------");
    println!("Input string: \"{input}\"");

    // Find and print the matches.
    let matches = trie.find_matches(input);

    println!("\nFound matching regex patterns:");
    if matches.is_empty() {
        println!("  No matches found.");
    } else {
        // Sort for consistent output order
        let mut sorted_matches = matches;
        sorted_matches.sort();
        for m in sorted_matches {
            println!("  - {m}");
        }
    }

    println!("\n---------------------------------");

    // Another example
    let input2 = "something12345";
    println!("\nInput string: \"{input2}\"");
    let matches2 = trie.find_matches(input2);
    println!("\nFound matching regex patterns:");
    if matches2.is_empty() {
        println!("  No matches found.");
    } else {
        for m in matches2 {
            println!("  - {m}");
        }
    }

    println!("\n---------------------------------");

    {
        // Create a new RegexTrie.
        let mut trie = RegexTrie::new();

        // Insert the regex patterns. Note that insert can now fail if the regex is invalid.
        let patterns = vec![
            ".*",
            ".*test",
            "test",
            "test.*",
            ".*test.*",
            ".*(test).*",
            ".*(test|toto).*",
        ];

        println!("Inserting patterns...");
        for pattern in patterns {
            if let Err(e) = trie.insert(pattern) {
                eprintln!("Failed to insert pattern '{pattern}': {e}");
            }
        }

        // Another example
        let input2 = "test";
        println!("\nInput string: \"{input2}\"");
        let matches2 = trie.find_matches(input2);
        println!("\nFound matching regex patterns:");
        if matches2.is_empty() {
            println!("  No matches found.");
        } else {
            for m in matches2 {
                println!("  - {m}");
            }
        }
    }

    {
        // Create a new RegexTrie.
        let mut trie = RegexTrie::new();

        // Insert the regex patterns. Note that insert can now fail if the regex is invalid.
        let patterns = vec![
            "https://google.com/.*",
            "https://google.com/user/.*",
            "https://google.com/user/.*/photos/*",
            "https://facebook.com",
        ];

        println!("Inserting patterns...");
        for pattern in patterns {
            if let Err(e) = trie.insert(pattern) {
                eprintln!("Failed to insert pattern '{pattern}': {e}");
            }
        }

        // Another example
        let input2 = "https://google.com/user/1234";
        println!("\nInput string: \"{input2}\"");
        let matches2 = trie.find_matches(input2);
        println!("\nFound matching regex patterns:");
        if matches2.is_empty() {
            println!("  No matches found.");
        } else {
            for m in matches2 {
                println!("  - {m}");
            }
        }
    }

    {
        // Create a new RegexTrie.
        let mut trie = RegexTrie::new();

        // Insert the regex patterns. Note that insert can now fail if the regex is invalid.
        let patterns = vec![".*", ".*ac", ".*bc", ".*abcd"];

        println!("Inserting patterns...");
        for pattern in patterns {
            if let Err(e) = trie.insert(pattern) {
                eprintln!("Failed to insert pattern '{pattern}': {e}");
            }
        }

        // dbg!(&trie);

        // Another example
        let input2 = "https://google.com/user/1234";
        println!("\nInput string: \"{input2}\"");
        let matches2 = trie.find_matches(input2);
        println!("\nFound matching regex patterns:");
        if matches2.is_empty() {
            println!("  No matches found.");
        } else {
            for m in matches2 {
                println!("  - {m}");
            }
        }
    }
}
