use pretty_assertions::assert_eq;

use crate::RegexTrie;

/// Test set
const TEST_SET: &str = include_str!("../assets/small_set.txt");

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

#[test]
fn test_character_class() {
    let patterns = vec!["test[0-9]+".to_string(), "test[^a-z]*".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    let result = tree.find_matches("test123");
    assert_eq_no_sort(patterns, result);

    let result = tree.find_matches("testa123");
    assert!(result.is_empty(), "should be empty");
}

#[test]
fn test_disjunction() {
    let patterns = vec!["test(abc|def)".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(patterns.clone(), tree.find_matches("testabc"));
    assert_eq_no_sort(patterns, tree.find_matches("testdef"));
    assert_eq_no_sort(Vec::<String>::default(), tree.find_matches("testxyz"));
}

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

#[test]
fn test_no_regex_match() {
    let patterns = vec!["test".to_string(), "test.*".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(patterns, tree.find_matches("test"));
}

#[test]
fn test_basic_escaped_characters() {
    let patterns = vec!["\\[".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(vec![patterns[0].clone()], tree.find_matches("["));
}

#[test]
fn test_escaped_characters() {
    let patterns = vec!["test\\[bracket\\]".to_string(), "\\.\\*toto".to_string()];
    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");

    assert_eq_no_sort(
        vec![patterns[0].clone()],
        tree.find_matches("test[bracket]"),
    );
}

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

    let tree = RegexTrie::from(&patterns).expect("can't init regex trie");
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
