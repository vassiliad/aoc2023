use clap::{arg, command, value_parser, Parser};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

enum Tile {
    Plot,
    Rock,
}

struct Board {
    width: isize,
    height: isize,
    maze: Vec<Tile>,
}

fn parse_text(text: &str) -> (Board, (isize, isize)) {
    let mut maze = Vec::new();
    let mut start = None;
    let mut width = 0;

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }
        width = line.len() as isize;

        for (x, c) in line.chars().enumerate() {
            let tile = match c {
                '.' => Tile::Plot,
                '#' => Tile::Rock,
                'S' => {
                    if start.is_none() {
                        start = Some((x as isize, (maze.len() / width as usize) as isize))
                    } else {
                        panic!("Multiple starting points")
                    }
                    Tile::Plot
                }
                _ => {
                    panic!("Unexpected character {c}")
                }
            };

            maze.push(tile);
        }
    }

    (
        Board {
            width,
            height: (maze.len() / width as usize) as isize,
            maze,
        },
        start.unwrap(),
    )
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    pos: (isize, isize),
    steps: u128,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.steps.cmp(&self.steps).then(self.pos.cmp(&other.pos))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn walk(board: &Board, start: &(isize, isize), max_steps: u128) -> u128 {
    let mut shortest_steps = HashMap::from([(start.clone(), 0)]);
    let mut pending = BinaryHeap::from([State {
        pos: start.clone(),
        steps: 0,
    }]);

    let deltas = [(-1, 0), (0, 1), (0, -1), (1, 0)];

    // VV: Basically, imagine stopping after step X on your walk and then retracing your steps.
    // This is equivalent to walking on a straight line but only counting every other step (e.g. you sacrifice one
    // tile so that you move forward). Then which tile you sacrifice depends on whether the steps are even or odd.
    // If they're odd you sacrifice the 1st tile (the one you started from), else you sacrifice the one right after
    // the start. You alternate between sacrificing a tile and marking a tile.
    let mut can_visit = if max_steps % 2 == 0 {
        HashSet::from([start.clone()])
    } else {
        HashSet::new()
    };

    while let Some(State { pos, steps }) = pending.pop() {
        let steps = steps + 1;

        for d in deltas {
            let pos = (pos.0 + d.0, pos.1 + d.1);

            let wrapped = (
                ((pos.0 % board.width) + board.width) % board.width,
                ((pos.1 % board.height) + board.height) % board.height,
            );

            if matches!(
                board.maze[(wrapped.0 + wrapped.1 * board.width) as usize],
                Tile::Rock
            ) {
                continue;
            }

            if let Some(known_steps) = shortest_steps.get(&pos) {
                if *known_steps <= steps {
                    continue;
                }
            }

            shortest_steps.insert(pos.clone(), steps);

            if steps % 2 == max_steps % 2 {
                can_visit.insert(pos);
            }

            if steps < max_steps {
                pending.push(State { pos, steps });
            }
        }
    }

    can_visit.len() as u128
}

fn solve(board: &Board, start: &(isize, isize), steps: u128) -> u128 {
    let remainder = steps % board.width as u128;
    let values_x = vec![
        remainder,
        board.width as u128 + remainder,
        (board.width * 2) as u128 + remainder,
    ];
    let values_y: Vec<u128> = values_x
        .iter()
        .map(|steps| walk(&board, &start, *steps))
        .collect();

    // VV: Looks like a quadratic thing, quacks like a quadratic thing. Must be a duck.
    let a = (values_y[2] + values_y[0] - 2 * values_y[1]) / 2;
    let b = values_y[1] - values_y[0] - a;
    let c = values_y[0];

    let x = steps / board.width as u128;

    x * x * a + b * x + c
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,

    #[arg(long, short, default_value_t = 26501365)]
    steps: u128,
}

fn main() {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(&args.input);
    let contents = std::fs::read_to_string(&path).expect("Reading input file");
    let (board, start) = parse_text(&contents);

    let solution = solve(&board, &start, args.steps);

    println!("{solution}");
}
