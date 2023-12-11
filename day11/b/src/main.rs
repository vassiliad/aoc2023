use anyhow::{bail, Context, Result};
use std::fmt::Formatter;

use clap::{arg, command, Parser};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Position {
    x: i128,
    y: i128,
}

struct Space {
    width: i128,
    height: i128,
    galaxies: Vec<Position>,
}

impl std::fmt::Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(_idx) = self
                    .galaxies
                    .iter()
                    .position(|galaxy| galaxy == &Position { x, y })
                {
                    write!(f, "#").unwrap();
                } else {
                    write!(f, ".").unwrap();
                }
            }

            write!(f, "\n").unwrap();
        }

        write!(f, "\n")
    }
}

fn parse_text(text: &str, expand: i128) -> Result<Space> {
    if expand < 1 {
        bail!("expand must be positive")
    }

    let expand = (expand - 1).max(1);

    let mut width = 0i128;
    let mut height = 0i128;
    let mut galaxies = Vec::new();

    let mut empty_rows = vec![];
    let mut empty_columns = vec![];

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }

        if width == 0 {
            width = line.len() as i128;
            empty_columns = (0..width).collect();
        }

        let mut galaxies_in_line = 0;
        for (x, c) in line.chars().enumerate() {
            let x = x as i128;

            match c {
                '#' => {
                    galaxies_in_line += 1;
                    galaxies.push((x, height));

                    if let Some(idx) = empty_columns.iter().position(|column| *column == x) {
                        empty_columns.remove(idx);
                    }
                }
                '.' => {}
                _ => bail!("Unexpected Debris in space {c}"),
            }
        }

        if galaxies_in_line == 0 {
            empty_rows.push(height);
        }

        height += 1;
    }

    width += empty_columns.len() as i128;
    height += empty_rows.len() as i128;

    let mut expand_galaxy_coords = |x, y| {
        let delta_x = empty_columns.partition_point(|col_idx| col_idx < x) as i128;
        let delta_y = empty_rows.partition_point(|row_idx| row_idx < y) as i128;

        Position {
            x: x + delta_x * expand,
            y: y + delta_y * expand,
        }
    };

    let expanded_galaxies = galaxies.iter().map(|(x, y)| expand_galaxy_coords(x, y));
    let expanded_galaxies = Vec::from_iter(expanded_galaxies);

    Ok(Space {
        width,
        height,
        galaxies: expanded_galaxies,
    })
}

fn parse_path(path: &std::path::Path, expand: i128) -> Result<Space> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| "Reading input file")
        .unwrap();
    parse_text(&contents, expand)
}
fn distance(start: Position, end: Position) -> u128 {
    let dx = (start.x - end.x).abs();
    let dy = (start.y - end.y).abs();
    (dx + dy) as u128
}

fn solve(space: &Space) -> u128 {
    (0..space.galaxies.len())
        .map(|i| {
            let start = space.galaxies[i];

            (i + 1..space.galaxies.len())
                .map(|j| {
                    let end = space.galaxies[j];

                    distance(start, end)
                })
                .sum::<u128>()
        })
        .sum()
}

#[test]
fn test_sample_10() -> Result<()> {
    let sample = "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

    let space = parse_text(sample, 10)?;

    println!("{space}");

    let solution = solve(&space);

    assert_eq!(solution, 1030);

    Ok(())
}

#[test]
fn test_sample_100() -> Result<()> {
    let sample = "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

    let space = parse_text(sample, 100)?;

    println!("{space}");

    let solution = solve(&space);

    assert_eq!(solution, 8410);

    Ok(())
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,

    #[arg(long, short, default_value_t = 1000000)]
    expand: i128,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(args.input);
    let space = parse_path(&path, args.expand)?;

    let solution = solve(&space);
    println!("{solution}");

    Ok(())
}
