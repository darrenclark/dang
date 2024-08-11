#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

impl LineCol {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    pub fn unknown() -> Self {
        Self::new(0, 0)
    }
}

impl From<(usize, usize)> for LineCol {
    fn from((line, col): (usize, usize)) -> Self {
        Self::new(line, col)
    }
}
