use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};
use std::path::{Path, PathBuf};
use std::ptr::copy_nonoverlapping;

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

fn reflection_horizontal(maze: &Maze) -> (Option<usize>, Option<usize>) {
    let mut mistakes = vec![0; maze.height];
    let mut perfect = None;

    for mirror in 0..maze.height - 1 {


        for x in 0..maze.width {
            let start_up = mirror;
            let start_down = mirror + 1;

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
                    mistakes[mirror] += 1;
                }
            }
        }

        if mistakes[mirror] == 0 {
            assert!(perfect.is_none());
            perfect = Some(mirror);
        }

    }

    if let Some(one_smudge) =  mistakes.iter().position(|x| *x == 1){
        return (Some(one_smudge + 1), perfect)
    } else if let Some(no_smudge) = perfect {
        return (None, Some(no_smudge + 1))
    }

    (None, None)
}

fn reflection_vertical(maze: &Maze) -> (Option<usize>, Option<usize>) {
    let mut mistakes = vec![0; maze.width];
    let mut perfect = None;

    for mirror in 0..maze.width - 1 {
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
                    mistakes[mirror] += 1;
                }
            }
        }

        if mistakes[mirror] == 0 {
            assert!(perfect.is_none());
            perfect = Some(mirror);
        }
    }

    if let Some(one_smudge) =  mistakes.iter().position(|x| *x == 1){
        return (Some(one_smudge + 1), perfect)
    } else if let Some(no_smudge) = perfect {
        return (None, Some(no_smudge + 1))
    }

    (None, None)
}

fn solve(mazes: &Vec<Maze>) -> u128 {
    mazes
        .iter()
        .map(|maze| {
            // VV: The idea here is to generate 2 numbers for each vertical/horizontal mirror
            // the first number is the mirror position which we'd get if the mirror had exactly 1 smudge
            // the second number is the mirror position we'd get if the mirror had no smudges at all
            // The actual mirror position is that for which there's 1 smudge, if there's no such scenario
            // then the actual mirror position is whichever mirror position would be valid if the mirror had no smudges
            let vertical = reflection_vertical(maze);
            let horizontal = reflection_horizontal(maze);

            let x = if let Some(one_smudge) = vertical.0 {
                one_smudge
            } else if let Some(one_smudge) = horizontal.0 {
                one_smudge * 100
            } else if let Some(perfect) = vertical.1 {
                perfect
            } else if let Some(perfect) = horizontal.1 {
                perfect * 100
            } else {
                unreachable!()
            };

            x as u128
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

    assert_eq!(solution, 400)
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
