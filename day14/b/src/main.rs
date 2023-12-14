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

const ITERATIONS: u128 = 1000000000;

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
    fn print(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                match self.board[(x + y * self.width) as usize] {
                    Tile::Round => print!("O"),
                    Tile::Cube => print!("#"),
                    Tile::Empty => print!("."),
                }
            }

            println!("");
        }

        println!("");
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

                    moved = true;
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

                    moved = true;
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
                    self.board[(x + (y - 1) * self.width) as usize] = Tile::Round;
                    self.board[(x + y * self.width) as usize] = Tile::Empty;

                    moved = true;
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
                    self.board[(x + (y + 1) * self.width) as usize] = Tile::Round;
                    self.board[(x + y * self.width) as usize] = Tile::Empty;

                    moved = true;
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
    Down = 2,
    Right = 3,
}

#[derive(Debug, Clone)]
struct State {
    dish: Dish,
    direction: Direction,
    score: u128,
}

fn kernel(
    dish: &mut Dish,
    iterations: u128,
    start_iter: u128,
    history: &mut HashMap<(Dish, usize), u128>,
    use_history: bool,
    skip_first: usize,
) -> u128 {
    let mut skip_first = skip_first;
    let mut use_history = use_history;

    for step in start_iter..iterations {
        for (idx, direction) in [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ]
        .iter()
        .enumerate()
        .skip(skip_first)
        {
            // println!("At step {step}@{idx} {}", dish.score_from_scratch());
            // dish.print();

            if use_history {
                // VV: Detect when the cycle begins and then just run the remaining iterations
                // There's probably a way to find the beginning of the exact iteration in the history ...
                if let Some(&cycle_start) = history.get(&(dish.clone(), idx)) {
                    let cycle_span = step - cycle_start;
                    let skipped_iterations = iterations - ((iterations - step) % cycle_span);

                    // println!("Cycle start at {cycle_start} @ {idx} when at iteration {step}  -> span {cycle_span}  -> i.e. {skipped_iterations}");
                    return kernel(dish, iterations, skipped_iterations, history, false, idx);
                }

                history.insert((dish.clone(), idx), step);
            }

            // VV: Move till there's no more moves left to do
            loop {
                let moved = match direction {
                    Direction::Up => dish.tilt_up(),
                    Direction::Left => dish.tilt_left(),
                    Direction::Down => dish.tilt_down(),
                    Direction::Right => dish.tilt_right(),
                };

                if !moved {
                    break;
                }
            }
        }

        skip_first = 0;
    }
    // println!("At step {iterations}@0 {}", dish.score_from_scratch());
    // dish.print();
    dish.score_from_scratch()
}

fn solve(dish: &mut Dish, iterations: u128) -> u128 {
    // VV: Keys are Dish plus the index of the direction right before it got applied
    let mut history: HashMap<(Dish, usize), u128> = HashMap::new();

    kernel(dish, iterations, 0, &mut history, true, 0)
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
    let solution = solve(&mut dish, ITERATIONS);

    assert_eq!(solution, 64);

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

    #[arg(long, default_value_t=ITERATIONS)]
    iterations: u128,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(&args.input);

    let mut dish = parse_path(&path)?;

    let solution = solve(&mut dish, ITERATIONS);

    println!("{solution}");

    Ok(())
}
