#[derive(Debug, Clone, thiserror::Error)]
pub enum ParsingError {
    #[error("bracket at {0} is not closed")]
    UnclosedBracket(usize),
    #[error("bracket at {0} doesn't have a corresponding opening bracket")]
    UnopenedBracket(usize),
}
