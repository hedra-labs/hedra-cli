//! What a failed copy out left in the destination.

/// What a failed copy out left in the destination, by its path, escaped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Left {
    /// A target this copy created, or a file it replaced, holding part of
    /// the copy.
    Partial(String),
    /// A directory that was there before the copy, which part of the copy
    /// was written into.
    Into(String),
}

impl Left {
    /// Says what was left, as a clause to follow a failure's message.
    pub fn clause(&self) -> String {
        match self {
            Self::Partial(p) => format!("a partial copy was left at {p}"),
            Self::Into(p) => {
                format!("part of the copy was written into {p}, which already existed")
            }
        }
    }

    /// Says what was left, as a sentence of its own.
    pub fn sentence(&self) -> String {
        let clause = self.clause();
        let mut chars = clause.chars();
        let first = chars.next().map(|c| c.to_ascii_uppercase());
        format!(
            "{}{}.",
            first.map(String::from).unwrap_or_default(),
            chars.as_str()
        )
    }
}
