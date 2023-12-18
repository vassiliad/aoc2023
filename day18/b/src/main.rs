use anyhow::{Context, Result};
use clap::{arg, command, Parser};

#[derive(Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug)]
struct Movement {
    direction: Direction,
    length: i128,
}

impl Direction {
    fn delta(&self) -> (i128, i128) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

fn parse_text(text: &str) -> Result<Vec<Movement>> {
    let mut ret = vec![];

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }

        let parts = line
            .rsplit_once(' ')
            .with_context(|| "Extract colour")
            .unwrap();

        let colour = parts
            .1
            .strip_prefix("(#")
            .with_context(|| "Strip colour prefix")?;
        let colour = colour
            .strip_suffix(")")
            .with_context(|| "Strip colour suffix")?;

        let colour = i128::from_str_radix(colour, 16).with_context(|| "Parse colour")?;

        let direction = colour & 3;
        let length = colour >> 4;

        let direction = match direction {
            0 => Direction::East,
            1 => Direction::South,
            2 => Direction::West,
            3 => Direction::North,
            _ => unreachable!(),
        };

        ret.push(Movement { length, direction })
    }

    Ok(ret)
}

fn parse_path(path: &std::path::Path) -> Result<Vec<Movement>> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input")?;
    parse_text(&contents)
}

fn solve(movements: &Vec<Movement>) -> u128 {
    // VV: This is a huge Polygon with integer coordinates. Use the Pick's theorem to count the number of vertices.
    // Theorem needs number of interior points (i) (via the Shoelace Formula) and boundary points (b)
    // Area = interiorPoints + (boundaryPoints/2) - 1

    // VV: digger points to the beginning of the next movement
    let mut digger = (0i128, 0i128);

    let polygon: Vec<_> = movements
        .iter()
        .map(|m| {
            let delta = m.direction.delta();
            digger = (digger.0 + delta.0 * m.length, digger.1 + delta.1 * m.length);
            digger
        })
        .collect();

    if polygon.last().unwrap() != &(0, 0) {
        unreachable!("Not a closed loop!")
    }

    let mut area = 0;

    for idx in 0..polygon.len() - 2 {
        area += -polygon[idx].1 * polygon[idx + 1].0 + polygon[idx].0 * polygon[idx + 1].1;
    }

    area += -polygon[polygon.len() - 1].1 * polygon[0].0 * polygon[0].1;
    area = area / 2;

    // VV: The boundary points are perimeter + 4 -- Unfortunately, I don't know why :)
    // I reached that conclusion by calculating the area for the sample and then looking at Pick's theorem.
    // Since the big-boy example is hard to debug, I used this approach for the small sample (i.e. before updating
    // the logic to extract the proper movements).
    // Setting the boundary points to the perimeter of the Polygon doesn't yield the correct value, however it's close.
    // So then, I assumed that the Area is
    //    area = interiorPoints + something
    // and I solved "something = perimeter * a + b" using the area, interiorPoints and something that I get for
    // the small sample (that of part1). This produced: boundaryPoints / 2 - 1 = perimeter * a + b
    // One nice pair of values for the small sample are
    //     a = 0.5 and b = 1
    // I plugged these factors in (i.e. boundaryPoints = perimeter + 4) then updated the parsing logic and
    // verified that the boundaryPoints I compute work for the sample of part B. That convinced me that I stumbled
    // upon the right answer.
    let perimeter = movements.iter().map(|x| x.length).sum::<i128>();
    let boundary_points = perimeter + 4;

    (area + boundary_points / 2 - 1) as u128
}

#[test]
fn test_sample() {
    let sample = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)";
    let movements = parse_text(sample).unwrap();

    let solution = solve(&movements);

    // assert_eq!(solution, 62)
    assert_eq!(solution, 952408144115)
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
    let movements = parse_path(&path)?;

    let solution = solve(&movements);

    println!("{solution}");

    Ok(())
}
