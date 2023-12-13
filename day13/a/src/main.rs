use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};
use std::path::{Path, PathBuf};

#[derive(Debug, Copy, Clone, PartialEq)]
enum Tile {
    Ash = 0,
    Rock = 1,
}

#[derive(Debug)]
struct Maze {
    width: usize,
    height: usize,

    board: Vec<Tile>,
}

fn parse_text(text: &str) -> Result<Vec<Maze>> {
    let mut mazes = vec![];

    let mut board = vec![];
    let mut width = 0;

    for line in text.lines() {
        let line = line.trim();

        if line.len() == 0 && board.len() > 0 {
            mazes.push(Maze {
                width: width,
                height: board.len() / width,
                board: board.clone(),
            });

            board.clear();

            continue;
        }

        width = line.len();

        for c in line.chars() {
            match c {
                '.' => board.push(Tile::Ash),
                '#' => board.push(Tile::Rock),
                _ => bail!("Unexpected character {c}"),
            }
        }
    }

    if board.len() > 0 {
        mazes.push(Maze {
            width: width,
            height: board.len() / width,
            board: board,
        });
    }

    Ok(mazes)
}

fn parse_path(path: &Path) -> Result<Vec<Maze>> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input")?;
    parse_text(&contents)
}

fn reflection_horizontal(maze: &Maze) -> Option<usize> {
    for mirror in 0..maze.height - 1 {
        let mut is_reflection = true;

        for x in 0..maze.width {
            let start_up = mirror;
            let start_down = mirror + 1;
            if !is_reflection {
                break;
            }
            for dy in 0..=mirror {
                let down = start_down + dy;
                if down >= maze.height {
                    continue;
                }

                if dy > start_up {
                    continue;
                }

                let up = start_up - dy;

                if maze.board[x + up * maze.width] != maze.board[x + down * maze.width] {
                    is_reflection = false;
                    break;
                }
            }
        }

        if is_reflection {
            let y = mirror + 1;
            return Some(y);
        }
    }

    None
}

fn reflection_vertical(maze: &Maze) -> Option<usize> {
    for mirror in 0..maze.width - 1 {
        let mut is_reflection = true;

        for y in 0..maze.height {
            let start_left = mirror;
            let start_right = mirror + 1;

            for dx in 0..=mirror {
                let right = start_right + dx;
                if right >= maze.width {
                    continue;
                }

                if dx > start_left {
                    continue;
                }

                let left = start_left - dx;

                if maze.board[left + y * maze.width] != maze.board[right + y * maze.width] {
                    is_reflection = false;
                    break;
                }
            }
        }

        if is_reflection {
            let x = mirror + 1;
            return Some(x);
        }
    }

    None
}

fn solve(mazes: &Vec<Maze>) -> u128 {
    mazes
        .iter()
        .map(|maze| {
            let vertical = reflection_vertical(maze);
            let horizontal = reflection_horizontal(maze);

            if vertical.is_none() {
                (horizontal.unwrap() * 100) as u128
            } else if horizontal.is_none() {
                vertical.unwrap() as u128
            } else {
                unreachable!()
            }
        })
        .sum()
}

#[test]
fn test_sample() {
    let sample = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#";
    let mazes = parse_text(sample).unwrap();

    let solution = solve(&mazes);

    assert_eq!(solution, 405)
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(&args.input);

    let mazes = parse_path(&path)?;
    let solution = solve(&mazes);

    println!("{solution}");

    Ok(())
}
