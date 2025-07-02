use pretty_assertions::assert_eq;

use crate::RegexTrie;

/// Test set
const TEST_SET: &str = include_str!("../assets/small_set.txt");

/// Test a basic regex works
#[test]
fn test_basic_patterns() {
    let patterns = vec![
        "https://www.google.com/.*".to_string(),
        "https://www.google.com/.*/toto/.*".to_string(),
    ];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    let result = tree.find_matches("https://www.google.com/test/toto/");
    assert_eq_no_sort(patterns, result);
}

/// Test a character class and negativ character class regex works
#[test]
fn test_character_class() {
    let patterns = vec!["test[0-9]+".to_string(), "test[^a-z]*".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    let result = tree.find_matches("test123");
    assert_eq_no_sort(patterns, result);

    let result = tree.find_matches("testa123");
    assert!(result.is_empty(), "should be empty");
}

/// Test a disjunction regex works
#[test]
fn test_disjunction() {
    let patterns = vec!["test(abc|def)".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(patterns.clone(), tree.find_matches("testabc"));
    assert_eq_no_sort(patterns, tree.find_matches("testdef"));
    assert_eq_no_sort(Vec::<String>::default(), tree.find_matches("testxyz"));
}

/// Test a quantifiers regex works
#[test]
fn test_quantifiers() {
    let patterns = vec![
        "test[0-9]?".to_string(),
        "test[0-9]+".to_string(),
        "test[0-9]{2,4}".to_string(),
    ];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(vec![patterns[0].clone()], tree.find_matches("test"));
    assert_eq_no_sort(
        vec![patterns[0].clone(), patterns[1].clone()],
        tree.find_matches("test5"),
    );
    assert_eq_no_sort(
        vec![patterns[1].clone(), patterns[2].clone()],
        tree.find_matches("test55"),
    );
}

/// Test a non matching input
#[test]
fn test_no_regex_match() {
    let patterns = vec!["test".to_string(), "test.*".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(patterns, tree.find_matches("test"));
}

/// Ensure basic escaping works
#[test]
fn test_basic_escaped_characters() {
    let patterns = vec!["\\[".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(vec![patterns[0].clone()], tree.find_matches("["));
}

/// Ensure many basic escaping works
#[test]
fn test_escaped_characters() {
    let patterns = vec!["test\\[bracket\\]".to_string(), "\\.\\*toto".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(
        vec![patterns[0].clone()],
        tree.find_matches("test[bracket]"),
    );
}

/// Test that priority is made by pattern length by default
#[test]
fn test_shortest_match_priority() {
    let patterns = vec!["a.*".to_string(), "a[0-9]+b.*".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    // Ensure everything match
    assert_eq_no_sort(patterns.clone(), tree.find_matches("a123bbb"));

    // Match best
    let result = tree.find_best_match("a123bbb");
    assert_eq!(Some(patterns[0].clone()), result, "should match");
}

/// Test that non-regex plain match have the priority by default
#[test]
fn test_plain_match_priority_over_regex() {
    let patterns = vec!["a.*".to_string(), "a123bbb".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    // Ensure everything match
    assert_eq_no_sort(patterns.clone(), tree.find_matches("a123bbb"));

    // Match best
    let result = tree.find_best_match("a123bbb");
    assert_eq!(
        Some(patterns[1].clone()),
        result,
        "should not match the shortest, but the plain match"
    );
}

/// Test that non-regex plain match have the priority
#[test]
fn test_custom_match_priority() {
    let patterns = vec!["a.*".to_string(), "a123bbb".to_string()];
    let tree = RegexTrie::from_with_scorer(
        &patterns,
        // Custom scorer which only take size
        Box::new(|pattern: &str, _| pattern.len()),
    )
    .expect("can't init regex trie");

    // Ensure everything match
    assert_eq_no_sort(patterns.clone(), tree.find_matches("a123bbb"));

    // Match best
    let result = tree.find_best_match("a123bbb");
    assert_eq!(
        Some(patterns[0].clone()),
        result,
        "should not match the shortest, but the plain match"
    );
}

/// Test custom comparator, not base on raw size, but on path fragment
#[test]
fn test_custom_url_priority() {
    let patterns = vec![
        "https://www.google.com/[a-zA-Z0-9/-]+".to_string(),
        "https://www.google.com/.*/test/.*".to_string(),
    ];
    let tree = RegexTrie::from_with_scorer(
        &patterns,
        // Custom scorer with url path segments count
        Box::new(|pattern: &str, _| {
            url::Url::parse(pattern).map_or_else(
                |_err| pattern.len(),
                |val| {
                    val.path_segments()
                        .map_or_else(|| pattern.len(), |segments| segments.into_iter().count())
                },
            )
        }),
    )
    .expect("can't init regex trie");

    // Ensure everything match
    assert_eq_no_sort(
        patterns.clone(),
        tree.find_matches("https://www.google.com/foo/test/bar"),
    );

    // Match best
    let result = tree.find_best_match("https://www.google.com/foo/test/bar");
    assert_eq!(
        Some(patterns[0].clone()),
        result,
        "should not match the shortest, but the plain match"
    );
}

/// Test real like assets works
#[expect(clippy::print_stdout, reason = "debug timing")]
#[test]
fn test_assets() {
    let patterns = TEST_SET
        .split('\n')
        .filter_map(|item| {
            if item.is_empty() {
                None
            } else {
                Some(item.to_string())
            }
        })
        .collect::<Vec<String>>();

    let start = std::time::Instant::now();
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");
    println!("===> loaded {} in {:?}", patterns.len(), start.elapsed());

    // Match best
    let result = tree.find_best_match(
        "https://www.google.com/b4a/test/mqgzumi/another/yh936/again/kk839gym/abc123",
    );
    assert_eq!(
        Some(
            "https://www\\.google\\.com/b4a/.*/mqgzumi/.*/yh936/.*/kk839gym/[a-z0-9]+".to_string()
        ),
        result,
        "should not match the shortest, but the plain match"
    );
}

/// Test failing a group insert, failed everything
#[test]
fn test_basic_error_handlings() {
    let patterns = vec!["https://www.google.com/[".to_string()];
    let tree = RegexTrie::from(&patterns);
    assert!(tree.is_err(), "should have failed");
}

/// Test that a failed insert, doesn't prevent another successful one to work.
#[test]
fn test_error_handlings() {
    let patterns = vec![
        "https://www\\.google\\.com/[".to_string(),
        "https://www\\.google\\.com/.*".to_string(),
    ];
    let mut tree = RegexTrie::new();
    for pattern in &patterns {
        let _ = tree.insert(pattern);
    }

    // Match best
    let result = tree.find_best_match("https://www.google.com/test");
    assert_eq!(
        Some(patterns[1].clone()),
        result,
        "should not match the shortest, but the plain match"
    );
}

/// Test inserting regex works
#[test]
fn test_insert() {
    let matching = "https://www.google.com/.*/toto/.*".to_string();
    let patterns = vec![
        matching.clone(),
        "https://www.google.com/dontmatch/.*".to_string(),
        "https://www.yahoo.com/.*".to_string(),
        "https://www.linkedin.com/.*".to_string(),
        "https://www.amazon.com/.*".to_string(),
    ];

    let mut tree = RegexTrie::new();
    for pattern in &patterns {
        tree.insert(pattern).expect("should have worked");
    }

    let result = tree.find_matches("https://www.google.com/test/toto/");
    assert_eq_no_sort(vec![matching], result);
}

/// Test inserting many regex works
#[test]
fn test_many_insert() {
    let matching = "https://www.google.com/.*/toto/.*".to_string();
    let patterns = vec![
        matching.clone(),
        "https://www.google.com/dontmatch/.*".to_string(),
        "https://www.yahoo.com/.*".to_string(),
        "https://www.linkedin.com/.*".to_string(),
        "https://www.amazon.com/.*".to_string(),
    ];

    let mut tree = RegexTrie::new();
    tree.insert_many(&patterns).expect("should have worked");

    let result = tree.find_matches("https://www.google.com/test/toto/");
    assert_eq_no_sort(vec![matching], result);
}

/// Helper which assert two vector are equal ignoring any ordering
#[track_caller]
fn assert_eq_no_sort<T>(mut lhs: Vec<T>, mut rhs: Vec<T>)
where
    T: std::cmp::Ord + std::fmt::Debug,
{
    lhs.sort();
    rhs.sort();
    assert_eq!(lhs, rhs, "should be equal");
}
