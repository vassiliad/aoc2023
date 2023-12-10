use anyhow::{Context, Result};
use std::fmt::write;
use std::{env, fmt};

use clap::{arg, command, Parser};

static STENCIL_NORTH_SOUTH: [u8;9] = [
    0, 1, 0,
    0, 1, 0,
    0, 1, 0
];

static STENCIL_EAST_WEST: [u8;9] = [
    0, 0, 0,
    3, 3, 3,
    0, 0, 0
];

// VV: Count the "southern" end of a North-facing "pipe" as a "wall" for the bucket-fill operation
static STENCIL_NORTH_EAST: [u8;9] = [
    0, 1, 0,
    0, 1, 3,
    0, 0, 0
];

// VV: Count the "southern" end of a North-facing "pipe" as a "wall" for the bucket-fill operation
static STENCIL_NORTH_WEST: [u8;9] = [
    0, 1, 0,
    3, 1, 0,
    0, 0, 0
];


// VV: For south-facing pipes do not count the "northern" end as a wall
static STENCIL_SOUTH_WEST: [u8;9] = [
    0, 0, 0,
    3, 3, 0,
    0, 1, 0
];

static STENCIL_SOUTH_EAST: [u8;9] = [
    0, 0, 0,
    0, 3, 3,
    0, 1, 0
];


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
    North = 1,
    South = 2,
    West = 4,
    East = 8,
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

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value {
            1 => Direction::North,
            2 => Direction::South,
            4 => Direction::West,
            8 => Direction::East,
            _ => unreachable!(),
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
    fn from_directions(dir1: &Direction, dir2: &Direction) -> Self {
        let agg = (*dir1 as u8) | (*dir2 as u8);

        match agg {
            3 => Pipe::NS,
            12 => Pipe::EW,
            9 => Pipe::NE,
            5 => Pipe::NW,
            6 => Pipe::SW,
            10 => Pipe::SE,
            _ => unreachable!(),
        }
    }

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
            '.' | 'I' | 'O' => Pipe::Empty,
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
            path.push((next, self.pipes[next].clone()));

            avoid_pos = start_pos;
            start_pos = next;
        }
    }
}

fn apply_stencil(blown: &mut Vec<u8>, width: usize, x: usize, y: usize, smaller: &[u8]) {
    for iy in 0..3usize {
        for  ix in 0..3usize {
            blown[x+ix + (y+iy)*width] = smaller[ix + iy*3]
        }
    }
}

fn discover_loop(maze: &mut Maze) {
    let neighbours = maze.neighbours(maze.start_pos);

    for (pos, direction) in neighbours {
        let neighbour = maze.pipes[pos];
        let reverse_dir = direction.reverse();

        if !neighbour.can_walk(&reverse_dir) {
            continue;
        }

        if let Some(l) = maze.walk_loop(pos, maze.start_pos) {
            let real_neighbours: Vec<(usize, Direction)> = maze
                .neighbours(maze.start_pos)
                .iter()
                .filter(|(neigh, direction)| {
                    let neighbour = maze.pipes[*neigh];
                    let reverse_dir = direction.reverse();
                    neighbour.can_walk(&reverse_dir)
                })
                .map(|nd| *nd)
                .collect();

            // VV: Clear the maze and calculate the type of the pipe at location S
            // VV: This is a delta from Part A
            maze.start = Pipe::from_directions(&real_neighbours[0].1, &real_neighbours[1].1);

            for idx in 0..maze.pipes.len() {
                maze.pipes[idx] = Pipe::Empty;
            }

            for (idx, pipe) in &l {
                maze.pipes[*idx] = *pipe;
            }

            maze.pipes[maze.start_pos] = maze.start;

            break;
        }
    }
}

fn solve(maze: &mut Maze) -> usize {
    // VV: Nearly identical to Part A. The difference is that the method also updates the
    // pipes so that:
    // 1. the Starting pipe is the actual shape of the pipe
    // 2. the maze only contains pipes that are part of the loop, everything else is empty space
    discover_loop(maze);

    // VV: Blow up the maze and make it 9 times as large (3x for the X axis and 3x for the Y axis)
    // Intuitively, instead of having 1 Cell with a pipe that has 2 shapes oriented in different
    // directions (-- and └) you have 2 pipes that connect to each other and are arranged in a
    // 3x3 space (see the top of this file for the stencil definitions).
    // This larger space now has much simpler shapes in it, we can just scan it from top to bottom
    // and left to right and count the number of "vertical" walls.
    // Because we blow it up by a factor of 3x3 we need to be careful about the "connection"
    // "pixels" of angled pipes. I'm counting those that are pointing North as a wall but not
    // those that are facing down.
    // This distinction wouldn't be necessary if I had used a factor of 4x4.
    let mut blown = vec![0;  maze.pipes.len() * 9];
    let iwidth = maze.width * 3;
    let iheight = maze.height * 3;


    for y in 0..maze.height {
        for x in 0..maze.width {
            let idx = y * maze.width + x;
            let pipe = maze.pipes[idx];

            let stencil = match pipe {
                Pipe::NS => &STENCIL_NORTH_SOUTH,
                Pipe::EW => &STENCIL_EAST_WEST,
                Pipe::NE => &STENCIL_NORTH_EAST,
                Pipe::NW => &STENCIL_NORTH_WEST,
                Pipe::SW => &STENCIL_SOUTH_WEST,
                Pipe::SE => &STENCIL_SOUTH_EAST,
                Pipe::Empty => &[2; 9],
                Pipe::Start => unreachable!()
            };

            apply_stencil(&mut blown, iwidth, x * 3, y * 3, stencil);
        }
    }

    // VV: Simple bucket-fill. An "empty space cell" which comes after an even number of walls
    // is OUTSIDE the loop.
    for y in 0..iheight {
        let mut walls = 0;

        for x in 0..iwidth {
            let idx = y * iwidth + x;
            if blown[idx] == 2 {
                if walls % 2 == 0 {
                    // VV: This empty space is NOT within the loop
                    blown[idx] = 0;
                }
            } else if blown[idx] == 1 {
                walls += 1;
            }

        }
    }
    blown.iter().filter(|x| **x == 2).count()/9
}

#[test]
fn test_sample0() -> Result<()> {
    let sample = ".....
.F-7.
.|.|.
.L-J.
.....";

    let mut maze = parse_text(sample);

    let solution = solve(&mut maze);

    println!("New maze:\n{maze}");

    assert_eq!(solution, 1);

    Ok(())
}

#[test]
fn test_sample1() -> Result<()> {
    let sample = "..F7.
.FJ|.
SJ.L7
|F--J
LJ...";

    let mut maze = parse_text(sample);

    let solution = solve(&mut maze);

    println!("New maze:\n{maze}");

    assert_eq!(solution, 1);

    Ok(())
}

#[test]
fn test_sample2() -> Result<()> {
    let sample = ".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...";

    let mut maze = parse_text(sample);

    let solution = solve(&mut maze);

    println!("New maze:\n{maze}");

    assert_eq!(solution, 8);

    Ok(())
}

#[test]
fn test_conversion() {
    assert!(matches!(
        Pipe::from_directions(&Direction::North, &Direction::South),
        Pipe::NS
    ));
    assert!(matches!(
        Pipe::from_directions(&Direction::North, &Direction::East),
        Pipe::NE
    ));
    assert!(matches!(
        Pipe::from_directions(&Direction::North, &Direction::West),
        Pipe::NW
    ));
    assert!(matches!(
        Pipe::from_directions(&Direction::South, &Direction::East),
        Pipe::SE
    ));
    assert!(matches!(
        Pipe::from_directions(&Direction::South, &Direction::West),
        Pipe::SW
    ));
    assert!(matches!(
        Pipe::from_directions(&Direction::East, &Direction::West),
        Pipe::EW
    ));
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
