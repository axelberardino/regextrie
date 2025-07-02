use regex_automata::dfa::regex::Regex;
use std::collections::{HashMap, HashSet};
use std::error::Error;

/// Special character in a regex
const SPECIALS: &str = ".?*+()[]{}";
/// Type for the scorer function
/// 1st arg is the pattern
/// 2nd arg is if it's a regex or a plain match
type ScorerFuncType = Box<dyn Fn(&str, bool) -> usize>;

/// Represents a node in the Regex Trie.
/// Each node has a map of children for subsequent characters. It stores the
/// indices of patterns that have this node's path as their literal prefix.
#[derive(Debug, Default)]
struct TrieNode {
    /// List of all children
    children: HashMap<char, TrieNode>,
    /// On which compiled pattern it should point
    pattern_indices: Vec<usize>,
    /// Indicate this node also count has a prefix without any regex
    contains_non_regex_prefix: bool,
    /// If this node is an escaped node
    is_escaped: bool,
}

/// The `RegexTrie` structure.
/// It holds the root of the trie and a vector of pre-compiled regex patterns
/// (DFAs).
pub struct RegexTrie {
    /// Head of the trie tree
    root: TrieNode,
    /// Stores the original pattern string and its compiled DFA, with an
    /// optional score
    compiled_patterns: Vec<(String, Regex, usize)>,
    /// Scorer function
    scorer: ScorerFuncType,
}

impl Default for RegexTrie {
    fn default() -> Self {
        Self::new_with_custom_scorer(Box::new(|pattern: &str, is_regex| {
            if is_regex {
                pattern.len()
            } else {
                // 0 score means it take priority over any regex
                0
            }
        }))
    }
}

impl std::fmt::Debug for RegexTrie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexTrie")
            .field("root", &self.root)
            .field("compiled_patterns", &self.compiled_patterns)
            .finish()
    }
}

impl RegexTrie {
    /// Creates a new, empty `RegexTrie` with default scorer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new, empty `RegexTrie` with a custom scorer.
    #[must_use]
    pub fn new_with_custom_scorer(scorer: ScorerFuncType) -> Self {
        Self {
            root: TrieNode::default(),
            compiled_patterns: Vec::default(),
            scorer,
        }
    }

    /// Creates a new `RegexTrie` from a set of patterns.
    ///
    /// ## Errors
    ///
    /// If any of the regex pattern can't be compiled
    pub fn from(patterns: &[String]) -> Result<Self, Box<dyn Error>> {
        let mut trie = Self::new();
        for pattern in patterns {
            trie.insert(pattern)?;
        }

        Ok(trie)
    }

    /// Creates a new `RegexTrie` from a set of patterns.
    ///
    /// ## Errors
    ///
    /// If any of the regex pattern can't be compiled
    pub fn from_with_scorer(
        patterns: &[String],
        scorer: ScorerFuncType,
    ) -> Result<Self, Box<dyn Error>> {
        let mut trie = Self::new_with_custom_scorer(scorer);
        for pattern in patterns {
            trie.insert(pattern)?;
        }

        Ok(trie)
    }

    /// Compiles a regex pattern and inserts it into the trie.
    /// The trie is built using the literal prefix of the pattern. The
    /// compilation is done once, upon insertion.
    ///
    /// ## Errors
    ///
    /// If the regex pattern can't be compiled
    pub fn insert(&mut self, pattern: &str) -> Result<(), Box<dyn Error>> {
        // Traverse the trie using the literal prefix of the pattern.
        let mut current_node = &mut self.root;
        let mut previous_char = None;
        let mut is_regex = false;
        let mut chars = pattern.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' && matches!(chars.peek(), Some(next) if SPECIALS.contains(*next)) {
                // Unpop the escape character
                previous_char = Some(ch);
                continue;
            }

            // Stop at the first non escaped regex meta-character.
            let mut is_escaped = false;
            if SPECIALS.contains(ch) {
                // Escaped means we should represent the pattern as escaped
                if previous_char == Some('\\') {
                    is_escaped = true;
                } else {
                    // This is a regex, we can stop
                    is_regex = true;
                    break;
                }
            }

            current_node = current_node.children.entry(ch).or_default();
            current_node.is_escaped = is_escaped;
            previous_char = Some(ch);
        }

        if is_regex {
            // Compile the pattern into a DFA. Return an error on failure.
            let dfa = Regex::new(pattern)?;
            let pattern_index = self.compiled_patterns.len();
            let score = (self.scorer)(pattern, true);
            self.compiled_patterns
                .push((pattern.to_string(), dfa, score));

            // Store the index of the compiled pattern at the node corresponding
            // to the end of its literal prefix.
            current_node.pattern_indices.push(pattern_index);
        } else {
            // Special value to indicate it's not a regex but a complete string
            current_node.contains_non_regex_prefix = true;
        }

        Ok(())
    }

    /// Finds all regex patterns in the trie that fully match the given input
    /// string.
    ///
    /// This method works in two stages:
    ///  **Candidate Selection:** It traverses the trie using the characters of
    ///  the `input` string. At each node it visits, it collects the indices of
    ///  any patterns that have that node's path as a literal prefix. This
    ///  creates a small set of candidate patterns.
    ///
    /// **DFA Matching:** It iterates through the candidate patterns. For each
    /// one, it retrieves its pre-compiled DFA and runs a match against the
    /// entire input string. This is very fast as the DFA is already built.
    #[must_use]
    pub fn find_matches(&self, input: &str) -> Vec<String> {
        let mut candidate_indices = HashSet::new();
        let mut current_node = &self.root;

        // Always include patterns with no literal prefix (e.g., ".*"), which
        // are stored at the root.
        for &index in &current_node.pattern_indices {
            candidate_indices.insert(index);
        }

        // Traverse the trie based on the input string to find more candidates.
        let mut input_match_entirely = true;
        let mut escaped_pattern = String::with_capacity(input.len());
        for ch in input.chars() {
            if let Some(node) = current_node.children.get(&ch) {
                if node.is_escaped {
                    escaped_pattern.push('\\');
                }
                escaped_pattern.push(ch);

                current_node = node;
                // Collect all patterns whose literal prefix matches what we've seen so far.
                for &index in &current_node.pattern_indices {
                    candidate_indices.insert(index);
                }
            } else {
                // No further path in the trie, so no more candidates can be found this way.
                input_match_entirely = false;
                break;
            }
        }

        let mut matching_patterns = Vec::new();

        // If we match the input exactly, it means there's no regex involved
        // here. We can directly return it.
        if input_match_entirely && current_node.contains_non_regex_prefix {
            matching_patterns.push(escaped_pattern);
        }

        // DFA Matching
        let input_bytes = input.as_bytes();

        for index in candidate_indices {
            let (pattern_str, dfa, _) = &self.compiled_patterns[index];

            if let Some(m) = dfa.find(input_bytes) {
                if m.start() == 0 && m.end() == input_bytes.len() {
                    matching_patterns.push(pattern_str.clone());
                }
            }
        }

        matching_patterns
    }

    /// Same as finding all the matches, but only keep the "best" match.
    /// See `scorer_func` in the init. By default, take the shortest pattern.
    ///
    /// See `find_matches` for explanation.
    #[must_use]
    pub fn find_best_match(&self, input: &str) -> Option<String> {
        let mut candidate_indices = HashSet::new();
        let mut current_node = &self.root;

        // Always include patterns with no literal prefix (e.g., ".*"), which
        // are stored at the root.
        for &index in &current_node.pattern_indices {
            candidate_indices.insert(index);
        }

        // Traverse the trie based on the input string to find more candidates.
        let mut input_match_entirely = true;
        let mut escaped_pattern = String::with_capacity(input.len());
        for ch in input.chars() {
            if let Some(node) = current_node.children.get(&ch) {
                if node.is_escaped {
                    escaped_pattern.push('\\');
                }
                escaped_pattern.push(ch);

                current_node = node;
                // Collect all patterns whose literal prefix matches what we've seen so far.
                for &index in &current_node.pattern_indices {
                    candidate_indices.insert(index);
                }
            } else {
                // No further path in the trie, so no more candidates can be found this way.
                input_match_entirely = false;
                break;
            }
        }

        let mut best_match = None;

        // If we match the input exactly, it means there's no regex involved
        // here. We can directly return it.
        if input_match_entirely && current_node.contains_non_regex_prefix {
            let score = (self.scorer)(&escaped_pattern, false);
            best_match = Some((escaped_pattern, score));
        }

        // DFA Matching
        let input_bytes = input.as_bytes();

        for index in candidate_indices {
            let (pattern_str, dfa, score) = &self.compiled_patterns[index];

            if let Some(m) = dfa.find(input_bytes) {
                if m.start() == 0 && m.end() == input_bytes.len() {
                    match &best_match {
                        Some((_, best_score)) => {
                            if score < best_score {
                                best_match = Some((pattern_str.clone(), *score));
                            }
                        }

                        None => best_match = Some((pattern_str.clone(), *score)),
                    }
                }
            }
        }

        best_match.map(|(pattern, _)| pattern)
    }
}
