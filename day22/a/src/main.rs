use clap::{arg, command, Parser};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{write, Formatter};

type Number = u16;

#[derive(Debug, Copy, Clone, PartialEq)]
struct Vector {
    x: Number,
    y: Number,
    z: Number,
}

impl Vector {
    fn from_str(text: &str) -> Self {
        let mut parts = text.split(',');
        let x = parts
            .next()
            .expect("Extract x")
            .trim()
            .parse()
            .expect("Parse X");
        let y = parts
            .next()
            .expect("Extract x")
            .trim()
            .parse()
            .expect("Parse X");
        let z = parts
            .next()
            .expect("Extract x")
            .trim()
            .parse()
            .expect("Parse X");

        Self { x, y, z }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Brick {
    start: Vector,
    end: Vector,
    name: usize,
}

impl PartialOrd for Brick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Brick {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.z.cmp(&other.start.z).then(
            self.start
                .y
                .cmp(&other.start.y)
                .then(self.start.x.cmp(&other.start.x)),
        )
    }
}

impl Eq for Brick {}

impl std::fmt::Display for Brick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}) {},{},{}~{},{},{}",
            self.name, self.start.x, self.start.y, self.start.z, self.end.x, self.end.y, self.end.z
        )
    }
}

impl Brick {
    fn overlaps_xy(&self, other: &Self) -> bool {
        (self.start.x <= other.end.x && self.end.x >= other.start.x)
            && (self.start.y <= other.end.y && self.end.y >= other.start.y)
    }
}

fn parse_text(text: &str) -> Vec<Brick> {
    let mut bricks: Vec<Brick> = text
        .lines()
        .map(|x| x.trim())
        .filter(|x| x.len() > 0)
        .map(|line| {
            let (start, end) = line.split_once('~').expect("Splitting at ~");

            let mut start = Vector::from_str(start);
            let mut end = Vector::from_str(end);

            // VV: Just wanna make sure that I don't have to worry about bricks that are "facing" the wrong way
            if end.x < start.x || end.y < start.y || end.z < start.z {
                panic!("Oh no  {line}");
            }

            Brick {
                start,
                end,
                name: 0,
            }
        })
        .collect();

    for (idx, brick) in bricks.iter_mut().enumerate() {
        brick.name = idx + 1;
    }

    // VV: Place bricks closer to the ground towards the start of the array
    bricks.sort();

    bricks
}

fn simulate(this: &Brick, bricks: &mut Vec<Brick>) {
    // VV: A brick falls to just above the brick that's overlapping with it on X,Y plane and is at a lower Z
    let mut this = this.clone();
    let mut new_z = 1;

    for bottom in bricks.iter() {
        if this.overlaps_xy(bottom) {
            new_z = new_z.max(bottom.end.z + 1);
        }
    }

    let delta = this.start.z - new_z;

    this.start.z -= delta;
    this.end.z -= delta;

    bricks.push(this);
}

fn calc_required(bricks: &Vec<Brick>) -> HashSet<usize> {
    // VV: Keys are Top brick, values are collection of bricks that are supporting the top brick
    let mut rests_on: HashMap<usize, Vec<usize>> = HashMap::new();

    for (top_idx, top) in bricks.iter().enumerate().rev() {
        for bottom in bricks.iter().rev().skip(bricks.len() - top_idx - 1) {
            if bottom.end.z + 1 == top.start.z && bottom.overlaps_xy(top) {
                if let Some(supporting) = rests_on.get_mut(&top.name) {
                    supporting.push(bottom.name);
                } else {
                    rests_on.insert(top.name, vec![bottom.name]);
                }
            }
        }
    }

    HashSet::from_iter(
        rests_on
            .iter()
            .filter(|(_top, bottom)| bottom.len() == 1)
            .map(|(_top, bottom)| bottom[0]),
    )
}

fn solve(bricks: &mut Vec<Brick>) -> u128 {
    // VV: Sort Bricks by their lowest Z point (i.e. start.z)
    let mut simulated: Vec<Brick> = vec![bricks[0].clone()];

    for brick in bricks.iter().skip(1) {
        simulate(brick, &mut simulated);
    }

    simulated.sort();

    let required = calc_required(&simulated);

    (simulated.len() - required.len()) as u128
}

#[test]
fn test_sample() {
    let sample = "1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9";

    let mut bricks = parse_text(sample);

    for brick in bricks.iter().rev() {
        println!("{brick}");
    }

    assert_eq!(solve(&mut bricks), 5)
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
    let contents = std::fs::read_to_string(&path).expect("Reading input file");

    let mut bricks = parse_text(&contents);

    let solution = solve(&mut bricks);

    println!("{solution}");
}
