use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Tile {
    Empty = 0,
    Round = 1,
    Cube = 2,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Dish {
    width: isize,
    height: isize,
    board: Vec<Tile>,
}

fn parse_text(text: &str) -> Result<Dish> {
    let mut width = 0;
    let mut board = vec![];

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }

        width = line.len() as isize;
        for c in line.chars() {
            let t = match c {
                'O' => Tile::Round,
                '#' => Tile::Cube,
                '.' => Tile::Empty,
                _ => bail!("Unexpected character {c}"),
            };
            board.push(t);
        }
    }

    Ok(Dish {
        width,
        height: board.len() as isize / width,
        board,
    })
}

fn parse_path(path: &std::path::Path) -> Result<Dish> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input file")?;
    parse_text(&contents)
}

impl Dish {
    fn get_at(&self, x: isize, y: isize) -> Tile {
        if x >= 0 && y >= 0 && x < self.width && y < self.height {
            self.board[(x + y * self.width) as usize]
        } else {
            Tile::Cube
        }
    }

    fn score_from_scratch(&self) -> u128 {
        let mut score = 0;
        for y in 0..self.height {
            let round = self.board[(y * self.width) as usize..((y + 1) * self.width) as usize]
                .iter()
                .filter(|x| matches!(x, Tile::Round))
                .count() as u128;
            let row = round * (self.height as u128 - y as u128);
            score += row;
        }

        score
    }

    fn tilt_left(&mut self) -> bool {
        let mut moved = false;

        for y in 0..self.height {
            for x in 1..self.width {
                if matches!(self.board[(x + y * self.width) as usize], Tile::Round)
                    && matches!(self.board[(x - 1 + y * self.width) as usize], Tile::Empty)
                {
                    self.board[(x - 1 + y * self.width) as usize] = Tile::Round;
                    self.board[(x + y * self.width) as usize] = Tile::Empty;
                }
            }
        }

        moved
    }

    fn tilt_right(&mut self) -> bool {
        let mut moved = false;

        for y in 0..self.height {
            for x in (0..self.width - 1).rev() {
                if matches!(self.board[(x + y * self.width) as usize], Tile::Round)
                    && matches!(self.board[(x + 1 + y * self.width) as usize], Tile::Empty)
                {
                    self.board[(x + 1 + y * self.width) as usize] = Tile::Round;
                    self.board[(x + y * self.width) as usize] = Tile::Empty;
                }
            }
        }

        moved
    }

    fn tilt_up(&mut self) -> bool {
        let mut moved = false;

        for y in 1..self.height {
            for x in 0..self.width {
                if matches!(self.board[(x + y * self.width) as usize], Tile::Round)
                    && matches!(self.board[(x + (y - 1) * self.width) as usize], Tile::Empty)
                {
                    moved = true;
                    self.board[(x + (y - 1) * self.width) as usize] = Tile::Round;
                    self.board[(x + y * self.width) as usize] = Tile::Empty;
                }
            }
        }

        moved
    }

    fn tilt_down(&mut self) -> bool {
        let mut moved = false;

        for y in (0..self.height - 1).rev() {
            for x in 0..self.width {
                if matches!(self.board[(x + y * self.width) as usize], Tile::Round)
                    && matches!(self.board[(x + (y + 1) * self.width) as usize], Tile::Empty)
                {
                    moved = true;
                    self.board[(x + (y + 1) * self.width) as usize] = Tile::Round;
                    self.board[(x + y * self.width) as usize] = Tile::Empty;
                }
            }
        }

        moved
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Direction {
    Up = 0,
    Left = 1,
    Down = 3,
    Right = 4,
}

#[derive(Debug, Clone)]
struct State {
    dish: Dish,
    direction: Direction,
    score: u128,
}

fn solve(dish: &mut Dish) -> u128 {
    let mut pending = vec![
        State {
            dish: dish.clone(),
            direction: Direction::Up,
            score: dish.score_from_scratch(),
        },
        State {
            dish: dish.clone(),
            direction: Direction::Down,
            score: dish.score_from_scratch(),
        },
        State {
            dish: dish.clone(),
            direction: Direction::Left,
            score: dish.score_from_scratch(),
        },
        State {
            dish: dish.clone(),
            direction: Direction::Right,
            score: dish.score_from_scratch(),
        },
    ];

    let mut memo: HashMap<(Dish, Direction), u128> = HashMap::new();
    let mut best_score = 0;

    while pending.len() > 0 {
        let current = pending.pop().unwrap();

        if let Some(score) = memo.get(&(current.dish.clone(), current.direction)) {
            best_score = best_score.max(*score);
            continue;
        }

        let new_state = match current.direction {
            Direction::Left => {
                let mut dish = current.dish.clone();
                dish.tilt_left();

                State {
                    score: dish.score_from_scratch(),
                    dish,
                    direction: current.direction,
                }
            }
            Direction::Right => {
                let mut dish = current.dish.clone();
                dish.tilt_right();

                State {
                    score: dish.score_from_scratch(),
                    dish,
                    direction: current.direction,
                }
            }
            Direction::Down => {
                let mut dish = current.dish.clone();
                dish.tilt_down();

                State {
                    score: dish.score_from_scratch(),
                    dish,
                    direction: current.direction,
                }
            }
            Direction::Up => {
                let mut dish = current.dish.clone();
                dish.tilt_up();

                State {
                    score: dish.score_from_scratch(),
                    dish,
                    direction: current.direction,
                }
            }
        };

        best_score = best_score.max(new_state.score);
        memo.insert((current.dish.clone(), current.direction), new_state.score);
        pending.push(new_state);
    }

    best_score
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";

    let mut dish = parse_text(sample)?;
    let solution = solve(&mut dish);

    assert_eq!(solution, 136);

    Ok(())
}

#[test]
fn test_sample_score() -> Result<()> {
    let sample = "OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....";

    let dish = parse_text(sample)?;

    assert_eq!(dish.score_from_scratch(), 136);

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

    let mut dish = parse_path(&path)?;

    let solution = solve(&mut dish);

    println!("{solution}");

    Ok(())
}
