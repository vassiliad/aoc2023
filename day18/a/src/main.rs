use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

#[derive(Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug)]
struct Movement {
    direction: Direction,
    length: u8,
    colour: u32,
}

impl Direction {
    fn delta(&self) -> (isize, isize) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

fn parse_text(text: &str) -> Result<Vec<Movement>> {
    let mut ret = vec![];

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }

        let mut parts = line.split(' ');
        let direction = parts.next().with_context(|| "Extract direction")?;
        let direction = match direction {
            "R" => Direction::East,
            "L" => Direction::West,
            "U" => Direction::North,
            "D" => Direction::South,
            _ => bail!("Invalid Direction in line {line}"),
        };

        let length = parts.next().with_context(|| "Extract length")?;
        let length = length.parse::<u8>().with_context(|| "Parse length")?;

        let colour = parts.next().with_context(|| "Extract colour")?;
        let colour = colour
            .strip_prefix("(#")
            .with_context(|| "Strip colour prefix")?;
        let colour = colour
            .strip_suffix(")")
            .with_context(|| "Strip colour suffix")?;
        let colour = u32::from_str_radix(colour, 16).with_context(|| "Parse colour")?;

        if parts.next().is_some() {
            bail!("Unexpected text past colour in {line}");
        }

        ret.push(Movement {
            length,
            direction,
            colour,
        })
    }

    Ok(ret)
}

fn parse_path(path: &std::path::Path) -> Result<Vec<Movement>> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input")?;
    parse_text(&contents)
}

fn flood_fill(
    border: &HashMap<(isize, isize), usize>,
    filled_in: &mut HashSet<(isize, isize)>,
    start: (isize, isize),
) -> u128 {
    let mut pending = vec![start];

    let deltas = [
        Direction::South.delta(),
        Direction::North.delta(),
        Direction::West.delta(),
        Direction::East.delta(),
    ];

    while pending.len() > 0 {
        let pos = pending.pop().unwrap();
        filled_in.insert(pos.clone());

        for d in &deltas {
            let new_pos = (pos.0 + d.0, pos.1 + d.1);
            if border.contains_key(&new_pos) {
                continue;
            }
            if filled_in.contains(&new_pos) {
                continue;
            }
            pending.push(new_pos);
        }
    }

    (filled_in.len() + border.len()) as u128
}

fn solve(movements: &Vec<Movement>) -> u128 {
    // VV: First, trace the border of the trench

    // VV: Contains just the points that make up the edges of the trench
    // Keys are (x, y) values (which can be negative) and values are indices to the movements vector
    let mut border = HashMap::new();
    let mut span_horiz = (0isize, 0isize);
    let mut span_vert = (0isize, 0isize);

    // VV: Current position for the head of the digger
    let mut digger = (0isize, 0isize);
    for (idx, m) in movements.iter().enumerate() {
        let delta = m.direction.delta();

        // VV: Record each individual border-pixel
        for _ in 0..m.length as isize {
            border.insert(digger.clone(), idx);
            digger = (digger.0 + delta.0, digger.1 + delta.1);
        }

        span_horiz.0 = span_horiz.0.min(digger.0);
        span_horiz.1 = span_horiz.1.max(digger.0);

        span_vert.0 = span_vert.0.min(digger.1);
        span_vert.1 = span_vert.1.max(digger.1);
    }

    span_horiz.1 += 1;
    span_vert.1 += 1;

    // VV: To flood fill, find one point inside the trench
    let mut filled_in = HashSet::new();

    let first = &movements[0];
    let second = &movements[1];

    // VV: A point inside the trench is the one at the corner of the first 2 Movements
    // i.e. the one right before the 1st movement's end and moved 1 towards the 2nd direction
    let first_end = first.direction.delta();
    let first_end = (
        first_end.0 * (first.length - 1) as isize,
        first_end.1 * (first.length - 1) as isize,
    );
    let second_delta = second.direction.delta();
    let inside = (first_end.0 + second_delta.0, first_end.1 + second_delta.1);

    flood_fill(&border, &mut filled_in, inside)
}

#[test]
fn test_sample() {
    let sample = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)";
    let movements = parse_text(sample).unwrap();

    let solution = solve(&movements);

    assert_eq!(solution, 62)
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
    let movements = parse_path(&path)?;

    let solution = solve(&movements);

    println!("{solution}");

    Ok(())
}
