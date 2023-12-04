use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Card {
    /// The Card ID
    pub id: u32,
    /// The winning numbers
    pub winning: Vec<u32>,
    /// Numbers on the card
    pub mine: Vec<u32>,
}

impl Card {
    pub fn from_str(text: &str) -> Result<Self> {
        let text = text.trim();
        if !text.starts_with("Card ") {
            bail!("text does not start with \"Card \"")
        }

        let (id, rest) = text
            .split_once(':')
            .with_context(|| "Splitting line at first ':'")?;

        let id = &id[5..];

        let id = id
            .trim()
            .parse::<u32>()
            .with_context(|| "Parsing the Card id")?;

        let (winning, mine) = rest
            .split_once("|")
            .with_context(|| "Extracting the winning and mine numbers")?;

        fn extract_numbers(text: &str) -> Vec<u32> {
            text.trim()
                .split(' ')
                .filter(|word| word.len() > 0)
                .map(|x| x.trim().parse::<u32>().expect("Parsing winning cards"))
                .collect()
        }

        let winning = extract_numbers(winning);
        let mine = extract_numbers(mine);

        Ok(Self { id, winning, mine })
    }
}

pub fn parse_str(text: &str) -> Result<Vec<Card>> {
    Ok(text
        .lines()
        .map(|line| {
            println!("Line is {line:?}");
            Card::from_str(line).unwrap()
        })
        .collect())
}

pub fn parse_path(path: &std::path::Path) -> Result<Vec<Card>> {
    let file = std::fs::File::open(path).with_context(|| "Unable to open file")?;
    let mut reader = BufReader::new(file);

    Ok(reader
        .lines()
        .map(|line| Card::from_str(&line.unwrap()).unwrap())
        .collect())
}
