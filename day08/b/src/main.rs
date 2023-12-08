use anyhow::{Context, Result};
use clap::{arg, command, Parser};
use itertools::iproduct;
use num::integer::lcm;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
enum Direction {
    Left,
    Right,
}

#[derive(Debug)]
struct Board {
    maze: HashMap<String, (String, String)>,
    directions: Vec<Direction>,
}

fn parse_text(text: &str) -> Result<Board> {
    let mut lines = text.lines();

    let directions: Vec<Direction> = lines
        .next()
        .with_context(|| "Extracting directions")
        .unwrap()
        .trim()
        .chars()
        .map(|c| match c {
            'L' => Direction::Left,
            'R' => Direction::Right,
            c => unreachable!("Unexpected character {c}"),
        })
        .collect();
    let mut maze = HashMap::new();

    for line in lines.map(|x| x.trim()).filter(|x| x.len() > 0) {
        let (curr, next) = line
            .split_once("=")
            .with_context(|| "Splitting a Maze line on =")
            .unwrap();
        let curr = curr.trim();
        let (left, right) = next
            .trim()
            .split_once(",")
            .with_context(|| "Splitting next nodes")
            .unwrap();
        let left = left.trim();
        let right = right.trim();

        let left = left
            .strip_prefix('(')
            .with_context(|| "Stripping prefix in Left")
            .unwrap();

        let right = right
            .strip_suffix(')')
            .with_context(|| "Stripping suffix in Right")
            .unwrap();
        maze.insert(curr.to_string(), (left.to_string(), right.to_string()));
    }

    Ok(Board { directions, maze })
}

fn parse_path(path: &std::path::Path) -> Result<Board> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| "Reading file")
        .unwrap();

    parse_text(&contents)
}

fn walk_to_node(board: &Board, curr: &str, dest: &str) -> Option<u128> {
    let mut steps = 0u128;
    let mut curr = curr;

    let mut explored: HashSet<(String, usize)> = HashSet::new();

    while curr != dest {
        let dir_index = (steps % board.directions.len() as u128) as usize;
        let dir = &board.directions[dir_index];
        steps += 1;

        let options = board
            .maze
            .get(curr)
            .with_context(|| "Getting possible next nodes")
            .unwrap();

        let next = (curr.to_string(), dir_index);

        if let Some(_) = explored.get(&next) {
            // VV: Node is unreachable
            return None;
        }

        explored.insert(next);

        curr = match dir {
            Direction::Left => &options.0,
            Direction::Right => &options.1,
        }
        .as_ref();
    }

    Some(steps)
}

// VV: Produced this starting from https://stackoverflow.com/a/65780800
fn product(vectors: Vec<Vec<u128>>) -> Vec<Vec<u128>> {
    let mut result: Vec<Vec<u128>> = vec![vec![]];

    for vector in vectors {
        result = iproduct!(result.iter(), vector.iter())
            .map(|(v, x)| {
                let mut v1 = v.clone();
                v1.push(*x);
                v1
            })
            .collect();
    }

    result
}

fn solve(board: &Board) -> u128 {
    let mut nodes_start = vec![];
    let mut nodes_end = vec![];

    for (node, _) in board.maze.iter() {
        if node.ends_with('Z') {
            nodes_end.push(node.clone());
        } else if node.ends_with('A') {
            nodes_start.push(node.clone());
        }
    }

    // VV: A vector of vectors. Each outer vector represents one A-Node. Each inner vector is the
    // steps to walk from said A-Node to the reachable Z-Nodes;
    let mut book = vec![];

    // VV: Visit all reachable Z-Nodes from all A-Nodes and record the distance (in number of steps)
    for curr in nodes_start.iter() {
        let mut distances = vec![];
        for dest in nodes_end.iter() {
            if let Some(steps) = walk_to_node(board, curr, dest) {
                distances.push(steps);
            }
        }

        book.push(distances);
    }

    // VV: The answer is min(lcm([a[i][z for all reachable z from a[i]] for all i])
    product(book)
        .iter()
        .map(|all_steps| {
            all_steps
                .iter()
                .fold(1u128, |curr_lcm, a_z_steps| lcm(curr_lcm, *a_z_steps))
        })
        .min()
        .with_context(|| "Computing least-common-multiple of steps from A to Z nodes")
        .unwrap()
}

#[test]
fn test_sample_0() -> Result<()> {
    let sample = "LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)";
    let board = parse_text(sample)?;

    println!("Board: {board:#?}");

    let solution = solve(&board);

    assert_eq!(solution, 6);

    Ok(())
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(short, long, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(args.input);

    let board = parse_path(&path)?;

    let solution = solve(&board);

    println!("{solution}");

    Ok(())
}
