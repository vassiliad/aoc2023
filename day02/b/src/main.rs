use anyhow::{bail, Context, Result};
use clap::{arg, Parser};
use std::io::BufRead;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value = "input/mine")]
    input: std::path::PathBuf,
}

struct Round {
    blue: u32,
    red: u32,
    green: u32,
}

struct Game {
    id: u32,
    rounds: Vec<Round>,
}

fn parse_reader(mut reader: Box<dyn std::io::BufRead>) -> Result<Vec<Game>> {
    let mut ret = vec![];
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.len() == 0 {
            continue;
        }

        if !line.starts_with("Game ") {
            bail!("Line {line} does not start with \"Game \"");
        }
        let line = &line[5..];
        let mut parts = line.split(':');
        let id = parts.next().with_context(|| "Part of line containing ID")?;
        let id = id.parse::<u32>().with_context(|| "Parsing ID of game")?;
        let mut rounds = vec![];
        let remaining = parts
            .next()
            .with_context(|| "Part of line containing rounds")?
            .trim();

        for part in remaining.split("; ") {
            let mut round = Round {
                blue: 0,
                red: 0,
                green: 0,
            };

            for cubes in part.split(", ") {
                let mut number_colour = cubes.split(" ");
                let number = number_colour.next().with_context(|| "Number in round")?;
                let colour = number_colour.next().with_context(|| "Colour in round")?;
                let number = number
                    .parse::<u32>()
                    .with_context(|| "Invalid number of cubes")?;

                match colour {
                    "blue" => round.blue = number,
                    "red" => round.red = number,
                    "green" => round.green = number,
                    what => bail!("Invalid colour {what}"),
                }
            }

            rounds.push(round);
        }

        ret.push(Game { id, rounds })
    }

    Ok(ret)
}

fn parse_str(text: &str) -> Result<Vec<Game>> {
    let cursor = std::io::Cursor::new(text.to_string());
    let reader = std::io::BufReader::new(cursor);
    parse_reader(Box::new(reader))
}

fn parse_path(path: &std::path::Path) -> Result<Vec<Game>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Could not open input file {0}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    parse_reader(Box::new(reader))
}

fn find_min_cubes(games: &Vec<Game>) -> u128 {
    games.iter().fold(0u128, |acc, game| {
        let min_cubes = game.rounds.iter().fold(
            Round {
                red: 0,
                blue: 0,
                green: 0,
            },
            |acc, round| Round {
                red: acc.red.max(round.red),
                green: acc.green.max(round.green),
                blue: acc.blue.max(round.blue),
            },
        );
        (min_cubes.red * min_cubes.green * min_cubes.blue) as u128 + acc
    })
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";

    let games = parse_str(sample)?;

    let solution = find_min_cubes(&games);

    assert_eq!(solution, 2286);
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cwd = std::env::current_dir()?;
    let path = cwd.join(&args.input);

    let games = parse_path(&path)?;
    let solution = find_min_cubes(&games);

    println!("{solution}");

    Ok(())
}
