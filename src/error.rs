/// Special error for the batch serialization
#[derive(Debug, thiserror::Error, Clone)]
pub enum RegexTrieError {
    /// When no columns are found in the specs
    #[error(transparent)]
    RegexCompilationFailed(Box<regex_automata::dfa::dense::BuildError>),
}
