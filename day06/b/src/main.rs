use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};

#[derive(Debug)]
struct Race {
    time: f32,
    distance: f32,
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(short, long, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn parse_text(text: &str) -> Result<Vec<Race>> {
    let mut lines = text.lines();
    let times = lines
        .next()
        .with_context(|| "Getting line with Time information")?
        .strip_prefix("Time:")
        .with_context(|| "Stripping prefix Time:")?;
    let distances = lines
        .next()
        .with_context(|| "Getting line with Distance information")?
        .strip_prefix("Distance:")
        .with_context(|| "Stripping prefix Distance:")?;

    let times = times.replace(" ", "");
    let distances = distances.replace(" ", "");

    let times = times
        .split(' ')
        .map(|x| x.trim())
        .filter(|x| x.len() > 0)
        .map(|x| {
            x.parse::<f32>()
                .with_context(|| format!("Parsing time {x}"))
                .unwrap()
        });
    let distances = distances
        .split(' ')
        .map(|x| x.trim())
        .filter(|x| x.len() > 0)
        .map(|x| {
            x.parse::<f32>()
                .with_context(|| format!("Parsing distance {x}"))
                .unwrap()
        });

    Ok(times
        .zip(distances)
        .map(|(time, distance)| Race { time, distance })
        .collect())
}

fn parse_path(path: &std::path::Path) -> Result<Vec<Race>> {
    let contents = std::fs::read_to_string(&path).with_context(|| "Could not read file")?;

    parse_text(&contents)
}

fn solve(races: &Vec<Race>) -> u128 {
    // VV: quadratic inequality: -h^2 + t*h - d > 0
    // Calc quadratic roots: (-t +- sqrt(t^2 -4*d) )/(-2)

    races.iter().fold(1u128, |total, race| {
        let d = (race.time * race.time - 4. * race.distance).sqrt();
        let a = -(-race.time + d) / 2.;
        let b = -(-race.time - d) / 2.;

        let this = (b.ceil() - a.floor()) as u128 - 1;

        this * total
    })
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "Time:      7  15   30
Distance:  9  40  200";
    let races = parse_text(sample)?;
    assert_eq!(solve(&races), 71503);
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(args.input);
    let races = parse_path(&path)?;

    let solution = solve(&races);

    println!("{solution}");

    Ok(())
}
