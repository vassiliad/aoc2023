use anyhow::{Context, Result};
use clap::{arg, Parser};
use std::env::current_dir;
use std::io::BufRead;

/// Advent of code, day 01/b
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn read_buffer(reader: Box<dyn std::io::BufRead>) -> Result<u128> {
    let digits = vec![
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];

    reader.lines().into_iter().fold(Ok(0), |sum_so_far, line| {
        let line = line?;
        let line = line.trim_end();
        if line.len() < 1 {
            sum_so_far
        } else {
            let mut pair = (None, None);
            let mut idx = 0;

            while idx < line.len() {
                let c = line.chars().nth(idx).context("Getting a character")?;
                let mut digit = None;
                idx += 1;

                if let Some(d) = c.to_digit(10) {
                    digit = Some(d);
                } else {
                    for (i, word) in digits.iter().enumerate() {
                        if line[(idx-1)..].starts_with(*word) {
                            digit = Some((i as u32 ) + 1);
                            // VV: Puzzle is fine with digits sharing letters
                            // e.g. eightwothree is equivalent to 823
                            // idx += word.len() -1;
                            break
                        }
                    }
                }

                if let Some(digit) = digit {
                    if pair.0.is_none() {
                        pair =  (Some(digit as u128), Some(digit as u128));
                    } else {
                        pair = (pair.0, Some(digit as u128));
                    }
                }
            }

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
    let file = std::fs::File::open(path).with_context(|| "Could not find input file")?;
    let reader = std::io::BufReader::new(file);
    read_buffer(Box::new(reader))
}

#[test]
fn test_small() -> Result<()> {
    let number = read_str(
        "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen"
            .to_string(),
    );

    assert_eq!(number?, 281);

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
