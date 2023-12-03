use anyhow::Result;
use clap::{arg, command, Parser};

#[derive(Clone, Parser)]
#[command(version, about)]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn solve(engine: &a::schematic::Schematic) -> u128 {
    engine.parts.iter().fold(0, |sum, part| {
        if part.symbols.len() > 0 {
            sum + part.number as u128
        } else {
            sum
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
    let engine = a::schematic::Schematic::parse_str(sample)?;
    let solution = solve(&engine);

    assert_eq!(solution, 4361);

    Ok(())
}

fn main() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let args = Args::parse();

    let path = cwd.join(args.input);
    let engine = a::schematic::Schematic::parse_path(&path)?;

    let solution = solve(&engine);

    println!("{solution}");

    Ok(())
}
