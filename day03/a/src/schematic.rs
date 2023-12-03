use anyhow::{Context, Result};
use regex::Regex;

#[derive(Debug)]
pub struct Symbol {
    pub x: usize,
    pub y: usize,
    pub label: char,
}

#[derive(Debug)]
pub struct Part {
    pub number: u32,
    pub symbols: Vec<Symbol>,
}
#[derive(Debug)]
pub struct Schematic {
    pub plan: String,

    pub width: usize,
    pub height: usize,

    /// An array containing all numbers and the symbol that they are associated with
    pub parts: Vec<Part>,
}

impl Schematic {
    pub fn new(text: &str) -> Result<Self> {
        Self::parse_str(text)
    }

    pub fn parse_path(path: &std::path::Path) -> Result<Self> {
        let contents = std::fs::read_to_string(&path).with_context(|| "Could not read file")?;
        Self::parse_str(&contents)
    }

    pub fn parse_str(text: &str) -> Result<Self> {
        let text = text.replace('\r', "");
        let text = text.trim();
        let lines = text.split('\n');
        let mut width = 0;
        let height = text.lines().count();

        let mut parts = vec![];

        fn scan_lines_for_symbols(
            number: u32,
            sub: &str,
            start_x: usize,
            end_x: usize,
            parts: &mut Vec<Part>,
        ) {
            let mut part = Part {
                number,
                symbols: vec![],
            };

            let symbols = &mut part.symbols;

            let lines = sub.split('\n');
            for (y, line) in lines.into_iter().enumerate() {
                for (idx, label) in line[start_x..end_x].chars().enumerate() {
                    if !(('0' <= label && label <= '9') || label == '.' || label == '\n') {
                        symbols.push(Symbol {
                            x: idx + start_x,
                            y,
                            label,
                        })
                    }
                }
            }

            parts.push(part);
        }

        let re = Regex::new(r"\d+")?;
        let mut y = 0usize;
        for line in lines {
            let line = line.trim();
            if line.len() == 0 {
                continue;
            }

            width = line.len() + 1;

            let many_matches = re.captures_iter(line);

            for m in many_matches {
                let m = m.get(0).unwrap();
                let number = m.as_str().parse::<u32>()?;
                let start = m.start();
                let end = m.end();

                let start_x = if start > 0 { start - 1 } else { start };
                let end_x = if end < width - 1 { end + 1 } else { end };

                let start_y = if y > 0 { y - 1 } else { y };
                let end_y = if y < height - 2 { y + 1 } else { y };

                let end_char = ((end_y + 1) * width).min(text.len());
                let sub = &text[start_y * width..end_char].trim();
                scan_lines_for_symbols(number, sub, start_x, end_x, &mut parts);
            }

            y += 1;
        }

        Ok(Self {
            plan: text.to_string(),
            width,
            height,
            parts,
        })
    }
}
