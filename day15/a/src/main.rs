use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};

enum Character {
    Letter(char),
    Digit(u8),
    Equal,
    Minus,
}

type Word = Vec<Character>;

impl Character {
    fn ascii(&self) -> u128 {
        match self {
            Character::Letter(c) => *c as u128,
            Character::Digit(d) => '0' as u128 + *d as u128,
            Character::Equal => '=' as u128,
            Character::Minus => '-' as u128,
        }
    }
}

fn parse_text(text: &str) -> Result<Vec<Word>> {
    let mut words = vec![];

    for many in text.trim().split(',').map(|x| x.trim()) {
        if text.len() == 0 {
            bail!("Consecutive , are invalid");
        }

        let word: Word = many
            .chars()
            .map(|c| match c {
                'a'..='z' => Character::Letter(c),
                '0'..='9' => Character::Digit(c.to_digit(10).unwrap() as u8),
                '-' => Character::Minus,
                '=' => Character::Equal,
                _ => unimplemented!("Unexpected {c}"),
            })
            .collect();

        words.push(word);
    }

    Ok(words)
}

fn parse_path(path: &std::path::Path) -> Result<Vec<Word>> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input file")?;
    parse_text(&contents)
}

fn solve(words: &Vec<Word>) -> u128 {
    words
        .iter()
        .map(|word| {
            word.iter()
                .fold(0u128, |hash, curr| ((hash + curr.ascii()) * 17) % 256)
        })
        .sum()
}

#[test]
fn test_sample() {
    let sample = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    let words = parse_text(sample).unwrap();

    assert_eq!(solve(&words), 1320);
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(args.input);

    let words = parse_path(&path)?;
    let solution = solve(&words);

    println!("{solution}");

    Ok(())
}
