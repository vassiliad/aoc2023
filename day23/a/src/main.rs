use clap::{arg, command, Parser};
use std::collections::HashSet;

enum Tile {
    Path,
    Forest,
    Right,
    Left,
    Up,
    Down,
}

struct Maze {
    width: usize,
    height: usize,
    board: Vec<Tile>,
}

impl Maze {
    fn teleport(&self, pos: usize) -> Option<usize> {
        let tile = &self.board[pos];

        match tile {
            Tile::Left => Some(pos - 1),
            Tile::Right => Some(pos + 1),
            Tile::Down => Some(pos + self.width),
            Tile::Up => Some(pos - self.width),
            _ => None,
        }
    }

    fn print(&self, pos: usize, visited: &HashSet<usize>) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = x + y * self.width;
                let tile = &self.board[idx];
                if visited.contains(&idx) {
                    print!("O");
                } else if idx == pos {
                    print!("@")
                } else if matches!(tile, Tile::Forest) {
                    print!("#")
                } else if matches!(tile, Tile::Path) {
                    print!(".")
                } else if matches!(tile, Tile::Left) {
                    print!("<")
                } else if matches!(tile, Tile::Right) {
                    print!(">")
                } else if matches!(tile, Tile::Up) {
                    print!("^")
                } else if matches!(tile, Tile::Down) {
                    print!("v")
                }
            }
            println!("");
        }
    }
}

fn parse_text(text: &str) -> Maze {
    let mut width = 0;
    let mut board = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }

        width = line.len();

        for c in line.chars() {
            let tile = match c {
                '.' => Tile::Path,
                '#' => Tile::Forest,
                '>' => Tile::Right,
                '<' => Tile::Left,
                '^' => Tile::Up,
                'v' => Tile::Down,
                _ => panic!("Invalid character {c}"),
            };

            board.push(tile);
        }
    }

    Maze {
        width,
        height: board.len() / width,
        board,
    }
}

struct State {
    pos: usize,
    visited: HashSet<usize>,
    portals: usize,
}

fn solve(maze: &Maze) -> usize {
    let mut pending = vec![State {
        // VV: Start directly below S so that we don't have to deal with checking whether neighbour cells are
        // a valid position on the board (the board has a border, boarder, hehe)
        pos: (1 + maze.width),
        // VV: Record that we've already visited S and the point just below it
        visited: HashSet::from([1, (1 + maze.width)]),
        portals: 0,
    }];

    let mut max_score = 0;
    let end = maze.width - 2 + (maze.height - 1) * maze.width;

    let deltas: [isize; 4] = [1, -1, maze.width as isize, -(maze.width as isize)];

    while let Some(State {
        pos,
        visited,
        portals,
    }) = pending.pop()
    {
        for d in &deltas {
            let mut portals = portals;
            let pos = (pos as isize + d) as usize;
            // VV: We might step in a portal (slope) so need to check the destination position instead of
            // that of the portal
            let pos = if let Some(pos) = maze.teleport(pos) {
                portals += 1;
                pos
            } else {
                pos
            };

            if pos == end {
                max_score = max_score.max(visited.len() + portals);
                continue;
            }

            if matches!(maze.board[pos], Tile::Forest) {
                continue;
            }

            if visited.contains(&pos) {
                continue;
            }
            let mut visited = visited.clone();
            visited.insert(pos);

            pending.push(State {
                pos,
                visited,
                portals,
            })
        }
    }

    max_score
}

#[test]
fn test_sample() {
    let sample = "#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#";

    let maze = parse_text(sample);

    let solution = solve(&maze);

    assert_eq!(solution, 94);
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(&args.input);
    let contents = std::fs::read_to_string(&path).expect("Read file input");

    let maze = parse_text(&contents);

    let solution = solve(&maze);

    println!("{solution}");
}
