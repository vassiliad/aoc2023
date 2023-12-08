use anyhow::{Context, Result};
use clap::{arg, command, Parser};
use std::collections::HashMap;

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

fn solve(board: &Board) -> u128 {
    let mut steps = 0u128;

    let mut curr = "AAA";
    let dest = "ZZZ";

    while curr != dest {
        let dir = &board.directions[(steps % board.directions.len() as u128) as usize];
        steps += 1;

        let options = board
            .maze
            .get(curr)
            .with_context(|| "Getting possible next nodes")
            .unwrap();

        curr = match dir {
            Direction::Left => &options.0,
            Direction::Right => &options.1,
        }
        .as_ref();
    }

    steps
}

#[test]
fn test_sample_0() -> Result<()> {
    let sample = "RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)";
    let board = parse_text(sample)?;

    println!("Board: {board:#?}");

    let solution = solve(&board);

    assert_eq!(solution, 2);

    Ok(())
}

#[test]
fn test_sample_1() -> Result<()> {
    let sample = "LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)";
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
