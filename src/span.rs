use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub line: u32,
    pub col: u32,
    pub length: u32,
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, col {}", self.line, self.col)
    }
}
