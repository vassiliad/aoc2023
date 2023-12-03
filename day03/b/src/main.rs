use std::collections::HashMap;
use anyhow::Result;
use clap::{arg, command, Parser};

#[derive(Clone, Parser)]
#[command(version, about)]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn solve(engine: &b::schematic::Schematic) -> u128 {
    let mut gears: HashMap<(usize, usize), Vec<u32>> = HashMap::new();

    engine.parts.iter().map(|part| {
        for symbol in &part.symbols {
            if symbol.label == '*' {
                if let Some(gear) = gears.get_mut(&(symbol.x, symbol.y)) {
                    gear.push(part.number);
                } else {
                    gears.insert((symbol.x, symbol.y), vec![part.number]);
                }
            }
        }
    }).count();

    gears.values().fold(0, |acc, numbers| {
        if numbers.len() == 2 {
            acc + (numbers.get(0).unwrap() * numbers.get(1).unwrap()) as u128
        } else {
            acc
        }
    })
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";
    let engine = b::schematic::Schematic::parse_str(sample)?;
    let solution = solve(&engine);

    assert_eq!(solution, 467835);

    Ok(())
}

fn main() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let args = Args::parse();

    let path = cwd.join(args.input);
    let engine = b::schematic::Schematic::parse_path(&path)?;

    let solution = solve(&engine);

    println!("{solution}");

    Ok(())
}
