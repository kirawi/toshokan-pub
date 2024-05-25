pub struct Csv {
    /// A vector of rows, with each row being a vector of the column values
    pub entries: Vec<Vec<String>>,
}

impl Csv {
    pub fn from(s: &str) -> Self {
        let mut entries = Vec::new();
        for line in s.lines().map(|l| l.trim()) {
            let cols: Vec<_> = line.split(',').map(|s| s.to_string()).collect();
            entries.push(cols);
        }
        Self { entries }
    }
}
