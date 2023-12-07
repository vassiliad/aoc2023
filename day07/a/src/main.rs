use anyhow::{Context, Result};
use clap::{arg, command, Parser};

/// First card, is "hand type", followed by the 5 cards in the hand
/// Hand Type is:
///     0: High Card
///     2: One Pair
///     22: Two pair
///     30: Three of a kind
///     32: Full house
///     40: Four of a kind
///     60: Five of a kind
#[derive(Debug)]
struct Cards {
    numbers: Vec<u8>,
}

#[derive(Debug)]
struct Hand {
    cards: Cards,
    bet: usize,
}

fn parse_cards(cards: &str) -> Result<Cards> {
    let mut histogram = vec![0; 15];

    let mut numbers: Vec<u8> = cards
        .chars()
        .map(|c| {
            let x = match c {
                '2'..='9' => c.to_digit(10).unwrap(),
                'T' => 10,
                'J' => 11,
                'Q' => 12,
                'K' => 13,
                'A' => 14,

                _ => unreachable!("Unexpected Card"),
            };

            let x = x as u8;

            *histogram.get_mut(x as usize).unwrap() += 1;

            x
        })
        .collect();

    // VV: Reverse sort histogram so that larger numbers end up first
    histogram.sort_by(|a: &u8, &b| b.cmp(a));
    let histogram = &histogram[..];
    let hand_type = if histogram[0] == 5 {
        50u8 // VV: 5 of a kind
    } else if histogram[0] == 4 {
        40 // VV: 4 of a kind
    } else if histogram[0] == 3 && histogram[1] == 2 {
        32 // VV: full house
    } else if histogram[0] == 3 {
        30 // VV: 3 of a kind
    } else if histogram[0] == 2 && histogram[1] == 2 {
        22 // VV: 2 pairs
    } else if histogram[0] == 2 {
        2 // VV: 1 pair
    } else {
        0 // VV: high card
    };
    numbers.insert(0, hand_type);

    Ok(Cards { numbers })
}

fn solve(hands: &mut Vec<Hand>) -> u128 {
    hands.sort_by(|a: &Hand, b| a.cards.numbers.cmp(&b.cards.numbers));

    hands
        .iter()
        .enumerate()
        .map(|(idx, hand)| hand.bet as u128 * (idx as u128 + 1))
        .sum()
}

fn parse_text(text: &str) -> Result<Vec<Hand>> {
    Ok(text
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| line.len() > 0)
        .map(|line| {
            let (cards, bet) = line
                .split_once(' ')
                .with_context(|| format!("Splitting line {line}"))
                .unwrap();
            let cards = parse_cards(cards).unwrap();
            let bet = bet
                .parse::<usize>()
                .with_context(|| "Parsing bet {bet}")
                .unwrap();

            Hand { cards, bet }
        })
        .collect())
}

fn parse_path(path: &std::path::Path) -> Result<Vec<Hand>> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input file")?;
    parse_text(&contents)
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";
    let mut cards = parse_text(sample)?;

    println!("Cards: {cards:#?}");

    let solution = solve(&mut cards);

    println!("Sorted Cards: {cards:#?}");

    assert_eq!(solution, 6440);

    Ok(())
}

#[derive(Parser)]
#[command(about)]
struct Args {
    #[arg(short, long, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(args.input);
    let mut cards = parse_path(&path)?;
    let solution = solve(&mut cards);

    println!("{solution}");

    Ok(())
}
