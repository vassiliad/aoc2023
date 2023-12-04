use std::env::current_dir;

use anyhow::Result;
use clap::{arg, command, Parser};

#[derive(Parser)]
#[command(about)]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn solve(cards: &Vec<b::card::Card>) -> u128 {
    fn calc_common(card: &b::card::Card) -> usize {
        card.mine
            .iter()
            .filter(|number| card.winning.contains(*number))
            .count()
    }

    let mut total_cards = vec![1; cards.len()];

    cards.iter().enumerate().fold(0u128, |acc, (idx, card)| {
        let common = calc_common(card);
        for i in idx + 1..(idx + 1 + common) {
            total_cards[i] += total_cards[idx];
        }

        acc + total_cards[idx]
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

    let cards = b::card::parse_str(sample)?;
    let solution = solve(&cards);

    assert_eq!(solution, 30);

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = current_dir()?.join(args.input);

    let cards = b::card::parse_path(&path)?;
    let solution = solve(&cards);

    println!("{solution}");
    Ok(())
}
