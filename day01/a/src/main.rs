use anyhow::{Context, Result};
use clap::{arg, Parser};
use std::env::current_dir;
use std::io::BufRead;

/// Advent of code, day 01/a
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn read_buffer(reader: Box<dyn std::io::BufRead>) -> Result<u128> {
    reader.lines().into_iter().fold(Ok(0), |sum_so_far, line| {
        let line = line?;
        let line = line.trim_end();

        if line.len() < 1 {
            sum_so_far
        } else {
            let pair = line.chars().fold((None, None), |acc, c| {
                if let Some(digit) = c.to_digit(10) {
                    if acc.0.is_none() {
                        (Some(digit as u128), Some(digit as u128))
                    } else {
                        (acc.0, Some(digit as u128))
                    }
                } else {
                    acc
                }
            });

            let sum = sum_so_far?;
            let first = (pair.0).context("First digit")?;
            let second = (pair.1).context("Second digit")?;
            let pair = first * 10 + second;

            Ok(sum + pair)
        }
    })
}

fn read_str(text: String) -> Result<u128> {
    let cursor = std::io::Cursor::new(text);
    let reader = std::io::BufReader::new(cursor);
    read_buffer(Box::new(reader))
}

fn read_path(path: &std::path::Path) -> Result<u128> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    read_buffer(Box::new(reader))
}

#[test]
fn test_small() -> Result<()> {
    let number = read_str(
        "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet"
            .to_string(),
    );

    assert_eq!(number?, 142);

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let cwd = current_dir()?;

    let path_input = if args.input.is_absolute() {
        args.input
    } else {
        cwd.join(args.input)
    };

    let result = read_path(&path_input)?;
    println!("{result}");

    Ok(())
}
