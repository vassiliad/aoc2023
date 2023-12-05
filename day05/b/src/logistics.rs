use anyhow::{bail, Context, Result};
use std::io::BufRead;

/// Maps a value from range [src, src+len) to range [dest, dest+len)
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Mapper {
    pub dest: usize,
    pub src: usize,
    pub len: usize,
    // pub accessed: bool,
}

pub type Rule = Vec<Mapper>;

#[derive(Debug)]
pub struct Book {
    pub seeds: Vec<(usize, usize)>,
    pub rules: Vec<Rule>,
}

impl Mapper {
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        let numbers: Vec<usize> = s
            .split(' ')
            .filter(|x| x.len() > 0)
            .map(|x| {
                x.parse::<usize>()
                    .with_context(|| format!("Parsing Mapper from {s}"))
                    .unwrap()
            })
            .collect();

        if numbers.len() != 3 {
            bail!("Line {s} does not contain 3 numbers - cannot build a Mapper out of it");
        }

        Ok(Self {
            dest: *numbers.get(0).unwrap(),
            src: *numbers.get(1).unwrap(),
            len: *numbers.get(2).unwrap(),
            // accessed: false,
        })
    }

    pub fn src_to_dest(&self, src: usize) -> Option<usize> {
        if self.src <= src && self.src + self.len > src {
            Some(self.dest + (src - self.src))
        } else {
            None
        }
    }
}

impl Book {
    pub fn parse_reader(reader: Box<dyn BufRead>) -> Result<Self> {
        let mut reader = reader.lines();

        let mut rules = vec![];
        let mut seeds: Option<Vec<usize>> = None;

        // VV: First process the line starting with "seeds: "
        for line in reader.by_ref() {
            let line = line?;
            let line = line.trim();

            if line.len() == 0 {
                continue;
            }

            if !line.starts_with("seeds:") {
                bail!("First line was supposed to start with \"seeds:\" but found {line} instead");
            }
            let (_, line) = line.split_once(":").unwrap();
            seeds = Some(
                line.split(" ")
                    .filter(|x| x.len() > 0)
                    .map(|x| {
                        x.parse::<usize>()
                            .with_context(|| "Unable to parse seed")
                            .unwrap()
                    })
                    .collect(),
            );
            break;
        }

        let seeds = seeds.unwrap();

        let mut seeds: Vec<(usize, usize)> = seeds
            .chunks(2)
            .map(|x| (*x.get(0).unwrap(), *x.get(1).unwrap()))
            .collect();

        seeds.sort_by(|a: &(usize, usize), b| a.0.cmp(&b.0));

        // VV: Next process all the X-to-Y rules
        let mut current_collection = vec![];

        for line in reader.by_ref() {
            let line = line?;
            let line = line.trim();

            if line.len() == 0 {
                continue;
            }

            if line.ends_with("map:") {
                if current_collection.len() > 0 {
                    current_collection.sort_by(|a: &Mapper, b| a.src.cmp(&b.src));
                    rules.push(current_collection.clone());
                    current_collection.clear();
                }
                continue;
            }

            let mapper = Mapper::from_str(line)?;

            if mapper.len > 0 {
                current_collection.push(mapper);
            }
        }

        if current_collection.len() > 0 {
            current_collection.sort_by(|a: &Mapper, b| a.src.cmp(&b.src));
            rules.push(current_collection.clone());
            current_collection.clear();
        }

        Ok(Self { rules, seeds })
    }

    pub fn parse_text(text: &str) -> Result<Self> {
        let cursor = std::io::Cursor::new(text.to_string());
        let reader = std::io::BufReader::new(cursor);

        Self::parse_reader(Box::new(reader))
    }

    pub fn parse_path(path: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(&path).with_context(|| "Cannot open input file")?;
        let reader = std::io::BufReader::new(file);

        Self::parse_reader(Box::new(reader))
    }

    pub fn introduce_seed_layer(&mut self) {
        let seed_layer: Vec<Mapper> = self
            .seeds
            .iter()
            .map(|&(start, len)| Mapper {
                dest: start,
                src: start,
                len,
                // accessed: false,
            })
            .collect();
        self.rules.insert(0, seed_layer);
        self.seeds.clear();
    }
}
