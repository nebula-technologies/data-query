pub(crate) const BLOCK_SEPARATOR: &str = "\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}";

pub(crate) const BLOCK_EQUAL: &str = "\u{0}\u{0}\u{0}\u{0}\u{7}\u{0}\u{0}\u{0}";
pub(crate) const BLOCK_PROCEED: &str = "\u{0}\u{0}\u{0}\u{0}\u{0}\u{7}\u{0}\u{0}";

/// Lexer constant

pub(crate) const LEX_IDENTIFIER: char = '.';
pub(crate) const LEX_PIPE: char = '|';

pub(crate) const LEX_CAPSULE_START: char = '(';
pub(crate) const LEX_CAPSULE_END: char = ')';

pub(crate) const LEX_GENERIC_START: char = '[';
pub(crate) const LEX_GENERIC_SEPARATOR: char = ',';
pub(crate) const LEX_GENERIC_SLICE: char = '-';
pub(crate) const LEX_GENERIC_END: char = ']';
