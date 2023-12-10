use anyhow::{Context, Result};
use std::env::consts::FAMILY;
use std::fmt::write;
use std::mem::discriminant;
use std::process::id;
use std::task::ready;
use std::{env, fmt};

use clap::{arg, command, Parser};

#[derive(Debug, Clone, Copy)]
enum Pipe {
    NS, // VV: North-South
    EW, // VV: East-West
    NE, // VV: North-East
    NW, // VV: North-West
    SW, // VV: South-West
    SE, // VV: South-East
    Empty,
    Start,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    fn reverse(&self) -> Self {
        match self {
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
        }
    }
}

#[derive(Debug)]
struct Maze {
    pipes: Vec<Pipe>,
    start_pos: usize,
    width: usize,
    height: usize,
    start: Pipe,
}

impl std::fmt::Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pipe::Empty => write!(f, "."),
            Pipe::Start => write!(f, "0"),
            Pipe::NS => write!(f, "│"),
            Pipe::EW => write!(f, "─"),
            Pipe::NE => write!(f, "└"),
            Pipe::NW => write!(f, "┘"),
            Pipe::SW => write!(f, "┐"),
            Pipe::SE => write!(f, "┌"),
        }
    }
}

impl std::fmt::Display for Maze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if idx != self.start_pos {
                    write!(f, "{}", self.pipes[idx]).unwrap()
                } else {
                    write!(f, "{}", self.start).unwrap()
                }
            }

            write!(f, "\n").unwrap()
        }
        write!(f, "\n")
    }
}

impl Pipe {
    fn can_walk(&self, direction: &Direction) -> bool {
        match self {
            Pipe::Empty => false,
            Pipe::Start => true,
            Pipe::NS => match direction {
                Direction::North | Direction::South => true,
                _ => false,
            },
            Pipe::EW => match direction {
                Direction::East | Direction::West => true,
                _ => false,
            },
            Pipe::NE => match direction {
                Direction::North | Direction::East => true,
                _ => false,
            },
            Pipe::NW => match direction {
                Direction::North | Direction::West => true,
                _ => false,
            },
            Pipe::SW => match direction {
                Direction::South | Direction::West => true,
                _ => false,
            },
            Pipe::SE => match direction {
                Direction::South | Direction::East => true,
                _ => false,
            },
        }
    }
}

fn parse_text(text: &str) -> Maze {
    let mut start_pos = 0usize;

    let (first, _rest) = text
        .split_once('\n')
        .with_context(|| "Identifying width")
        .unwrap();
    let width = first.trim().len();
    let pipes: Vec<Pipe> = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .enumerate()
        .map(|(idx, c)| match c {
            '|' => Pipe::NS,
            '-' => Pipe::EW,
            'L' => Pipe::NE,
            'J' => Pipe::NW,
            '7' => Pipe::SW,
            'F' => Pipe::SE,
            '.' => Pipe::Empty,
            'S' => {
                start_pos = idx;

                Pipe::Start
            }
            c => unimplemented!("Unexpected character {c}"),
        })
        .collect();

    let height = pipes.len() / width;

    Maze {
        pipes,
        width,
        height,
        start_pos,

        // VV: We don't really know what the Starting pipe looks like because the presumed animal
        // is in the way. Bad "animal" !
        start: Pipe::Start,
    }
}

fn parse_path(path: &std::path::Path) -> Result<Maze> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input file")?;

    Ok(parse_text(&contents))
}

impl Maze {
    fn neighbours(&self, pos: usize) -> Vec<(usize, Direction)> {
        let pos = pos as isize;

        let pos_x = pos % self.width as isize;
        let pos_y = pos / self.width as isize;

        [
            (-1isize, 0isize, Direction::West),
            (1, 0, Direction::East),
            (0, 1, Direction::South),
            (0, -1, Direction::North),
        ]
        .iter()
        .map(|(dx, dy, direction)| {
            let x = pos_x + dx;
            let y = pos_y + dy;

            if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
                None
            } else {
                let cur = &self.pipes[pos as usize];
                let idx = (y * self.width as isize) + x;

                assert_ne!(pos, idx);

                let neighbour = &self.pipes[idx as usize];
                if let Pipe::Empty = neighbour {
                    None
                } else if cur.can_walk(direction) {
                    Some((idx as usize, *direction))
                } else {
                    None
                }
            }
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect()
    }

    /// Walks a loop starting from a Pipe and heading towards a direction.
    /// If there is a loop starting there, it returns a list of all tiles (their location, and Pipe type)
    /// If there is no loop starting there, it returns None
    fn walk_loop(&self, start_pos: usize, avoid_pos: usize) -> Option<Vec<(usize, Pipe)>> {
        let myself = (&self.pipes[start_pos]).clone();

        let mut path = vec![(start_pos, myself)];

        let mut start_pos = start_pos;
        let mut avoid_pos = avoid_pos;

        loop {
            let neighbours: Vec<(usize, Direction)> = self
                .neighbours(start_pos)
                .iter()
                .filter(|(next, direction)| {
                    if *next != avoid_pos {
                        let reverse_dir = direction.reverse();
                        self.pipes[*next].can_walk(&reverse_dir)
                    } else {
                        false
                    }
                })
                .map(|x| *x)
                .collect();

            if neighbours.len() == 0 {
                // VV: My neighbours do not connect back to me, therefore I'm not in a loop
                return None;
            }

            let (next, _direction) = neighbours[0];

            if next == self.start_pos {
                // VV: Found the beginning of the loop
                return Some(path);
            }

            if next == avoid_pos {
                // VV: There's  no more room to explore
                return None;
            }
            path.push((next, self.pipes[next]));

            avoid_pos = start_pos;
            start_pos = next;
        }
    }
}

fn solve(maze: &mut Maze) -> usize {
    let neighbours = maze.neighbours(maze.start_pos);

    for (pos, direction) in neighbours {
        let neighbour = maze.pipes[pos];
        let reverse_dir = direction.reverse();

        if !neighbour.can_walk(&reverse_dir) {
            continue;
        }

        if let Some(l) = maze.walk_loop(pos, maze.start_pos) {
            maze.pipes.resize(maze.pipes.len(), Pipe::Empty);
            for (idx, pipe) in &l {
                maze.pipes[*idx] = *pipe;
            }

            return (l.len() + 1) / 2;
        }
    }

    unreachable!()
}

#[test]
fn test_sample0() -> Result<()> {
    let sample = ".....
.S-7.
.|.|.
.L-J.
.....";
    let mut maze = parse_text(sample);

    println!("Maze\n{maze}");

    let solution = solve(&mut maze);

    assert_eq!(solution, 4);

    Ok(())
}

#[test]
fn test_sample1() -> Result<()> {
    let sample = "7-F7-
.FJ|7
SJLL7
|F--J
LJ.LJ";

    let mut maze = parse_text(sample);

    let solution = solve(&mut maze);

    assert_eq!(solution, 8);

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
    let path = std::env::current_dir().unwrap().join(args.input);

    let mut maze = parse_path(&path)?;

    let solution = solve(&mut maze);

    println!("{solution}");

    Ok(())
}
