use std::env::current_dir;

use anyhow::Result;
use clap::{arg, command, Parser};

#[derive(Parser)]
#[command(about)]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn solve(cards: &Vec<a::card::Card>) -> u128 {
    cards.iter().fold(0u128, |acc, card| {
        let common = card
            .mine
            .iter()
            .filter(|number| card.winning.contains(*number))
            .count();

        let score = if common > 0 { 1 << (common - 1) } else { 0 };
        acc + score
    })
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";

    let cards = a::card::parse_str(sample)?;
    let solution = solve(&cards);

    assert_eq!(solution, 13);

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = current_dir()?.join(args.input);

    let cards = a::card::parse_path(&path)?;
    let solution = solve(&cards);

    println!("{solution}");
    Ok(())
}
