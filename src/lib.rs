//! regex-trie

/// Regex trie
mod regex_trie;
pub use regex_trie::RegexTrie;

/// Error for regex trie
mod error;
pub use error::RegexTrieError;

/// Test for pattern parser
#[cfg(test)]
mod regex_trie_test;
