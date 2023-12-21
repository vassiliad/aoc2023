use clap::{arg, command, Parser};
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

fn solve(board: &Board, start: &(isize, isize), max_steps: u128) -> usize {
    let mut shortest_steps = HashMap::from([(start.clone(), 0)]);
    let mut pending = BinaryHeap::from([State {
        pos: start.clone(),
        steps: 0,
    }]);

    let deltas = [(-1, 0), (0, 1), (0, -1), (1, 0)];

    let mut can_visit = HashSet::from([start.clone()]);

    while let Some(State { pos, steps }) = pending.pop() {
        let steps = steps + 1;

        for d in deltas {
            let pos = (pos.0 + d.0, pos.1 + d.1);

            if pos.0 < 0
                || pos.1 < 0
                || pos.0 >= board.width
                || pos.1 >= board.height
                || matches!(
                    board.maze[(pos.0 + pos.1 * board.width) as usize],
                    Tile::Rock
                )
            {
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

    for y in 0..board.height as usize {
        for x in 0..board.width as usize {
            if can_visit.contains(&(x as isize, y as isize)) {
                print!("O")
            } else {
                if matches!(board.maze[x + y * board.width as usize], Tile::Plot) {
                    print!(".")
                } else {
                    print!("#")
                }
            }
        }

        println!("")
    }

    can_visit.len()
}

#[test]
fn test_sample() {
    let sample = "...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........";

    let (board, start) = parse_text(sample);
    assert_eq!(solve(&board, &start, 6), 16);
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,

    #[arg(long, short, default_value_t = 64)]
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
