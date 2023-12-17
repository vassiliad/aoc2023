use anyhow::{Context, Result};
use clap::{arg, command, Parser};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

struct Maze {
    width: isize,
    height: isize,
    board: Vec<u8>,
}

impl Maze {
    fn get(&self, x: isize, y: isize) -> Option<u8> {
        if x >= 0 && y >= 0 && x < self.width && y < self.height {
            Some(self.board[(x + y * self.height) as usize])
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 4,
}

impl Direction {
    fn to_delta(&self) -> (isize, isize) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }

    fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct State {
    x: isize,
    y: isize,
    consecutive: u8,
    dir: Direction,
    heat_loss: u128,
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .heat_loss
            .cmp(&self.heat_loss)
            .then(
                self.x
                    .cmp(&other.x)
                    .then(self.y.cmp(&other.y))
                    .then(self.dir.cmp(&other.dir)),
            )
            .then(self.consecutive.cmp(&other.consecutive))
    }
}

fn parse_text(text: &str) -> Maze {
    let mut width = 0isize;
    let mut board = vec![];

    for line in text.lines() {
        let line = line.trim();

        if line.len() == 0 {
            continue;
        }

        width = line.len() as isize;
        board.extend(
            line.chars()
                .map(|c| c.to_digit(10).with_context(|| "Parsing number").unwrap() as u8),
        )
    }

    Maze {
        width: width,
        height: board.len() as isize / width,
        board,
    }
}

fn parse_path(path: &std::path::Path) -> Result<Maze> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input")?;
    Ok(parse_text(&contents))
}

fn solve(maze: &Maze) -> u128 {
    let end = (maze.width - 1, maze.height - 1);
    let heuristic = (end.0 + end.1) as u128;

    let mut pending = BinaryHeap::from([
        State {
            x: 0,
            y: 0,
            consecutive: 0,
            dir: Direction::South,
            heat_loss: 0,
        },
        State {
            x: 0,
            y: 0,
            consecutive: 0,
            dir: Direction::East,
            heat_loss: 0,
        },
    ]);

    let directions = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];

    let mut known_best = HashMap::new();

    while let Some(current) = pending.pop() {
        if current.x == end.0 && current.y == end.1 {
            return current.heat_loss;
        }

        for dir in &directions {
            let delta = dir.to_delta();

            if current.dir == dir.reverse() {
                continue;
            }

            let length = if current.dir == *dir {
                current.consecutive as isize + 1
            } else {
                1
            };
            let position = (current.x + delta.0, current.y + delta.1);

            if length > 3 {
                continue;
            }

            let mut heat_loss = current.heat_loss;

            if let Some(next) = maze.get(position.0, position.1) {
                heat_loss += next as u128;

                if let Some(known_heat_loss) = known_best.get(&(position, *dir, length)) {
                    if *known_heat_loss <= heat_loss {
                        continue;
                    }
                }

                known_best.insert((position, *dir, length).clone(), heat_loss);

                let next = State {
                    heat_loss,
                    consecutive: length as u8,
                    x: position.0,
                    y: position.1,
                    dir: *dir,
                };

                pending.push(next);
            }
        }
    }
    unreachable!()
}

#[test]
fn test_sample() {
    let sample = "2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533";

    let maze = parse_text(sample);

    assert_eq!(solve(&maze), 102);
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

    let maze = parse_path(&path)?;
    let solution = solve(&maze);

    println!("{solution}");

    Ok(())
}
