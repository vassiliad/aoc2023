use anyhow::{Context, Result};
use clap::{arg, command, Parser};

fn parse_text(text: &str) -> Vec<Vec<i128>> {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| line.len() > 0)
        .map(|line| {
            line.split(' ')
                .map(|word| {
                    word.parse::<i128>()
                        .with_context(|| format!("Parsing number from {word}"))
                        .unwrap()
                })
                .collect()
        })
        .collect()
}

fn parse_path(path: &std::path::Path) -> Result<Vec<Vec<i128>>> {
    let contents = std::fs::read_to_string(&path).with_context(|| "Reading input file")?;
    Ok(parse_text(&contents))
}

fn solve(values: &Vec<Vec<i128>>) -> i128 {
    values
        .iter()
        .map(|values| {
            let mut input = values.clone();
            let mut output: Vec<i128> = vec![];
            let mut aggregate = vec![*input.last().unwrap()];

            loop {
                output.extend((1..input.len()).map(|idx| input[idx] - input[idx - 1]));
                aggregate.push(
                    *output
                        .last()
                        .with_context(|| "Extracting last value in output vector")
                        .unwrap(),
                );

                if output.iter().all(|num| *num == 0) {
                    break;
                }

                std::mem::swap(&mut input, &mut output);
                output.clear();
            }

            aggregate.iter().sum::<i128>()
        })
        .sum()
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";

    let values = parse_text(sample);

    // println!("Values: {values:?}");

    let solution = solve(&values);

    assert_eq!(solution, 114);

    Ok(())
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(&args.input);

    let numbers = parse_path(&path)?;

    let solution = solve(&numbers);

    println!("{solution}");

    Ok(())
}
