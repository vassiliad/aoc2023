use clap::{arg, command, Parser};

type Number = i128;

#[derive(Debug)]
struct Vector {
    x: Number,
    y: Number,
    // z: Number,
}

#[derive(Debug)]
struct Shard {
    pos: Vector,
    vel: Vector,
}

impl Vector {
    fn from_str(text: &str) -> Self {
        let mut tokens = text.trim().split(',');

        let x = tokens
            .next()
            .expect("Get X")
            .trim()
            .parse::<Number>()
            .expect("Parse X");
        let y = tokens
            .next()
            .expect("Get Y")
            .trim()
            .parse::<Number>()
            .expect("Parse Y");
        // let z = tokens
        //     .next()
        //     .expect("Get Z")
        //     .trim()
        //     .parse::<Number>()
        //     .expect("Parse Z");

        Self { x, y /*, z */ }
    }
}

impl Shard {
    fn from_str(text: &str) -> Self {
        let (pos, vel) = text.split_once("@").expect("Split position and velocity");
        let pos = Vector::from_str(pos);
        let vel = Vector::from_str(vel);

        Self { pos, vel }
    }

    /// Computes y = ax + b and returns (a, b)
    fn to_line(&self) -> (f64, f64) {
        let a = self.vel.y as f64 / self.vel.x as f64;
        let b = self.pos.y as f64 - a * self.pos.x as f64;

        (a, b)
    }

    fn intersection_point(first: &Self, other: &Self) -> Option<(Number, Number)> {
        let (a1, b1) = first.to_line();
        let (a2, b2) = other.to_line();

        /* VV: Find intersection point between:
                y = a1*x + b1
                y = a2*x + b2

           a1 * x + b1 = a2 * x + b2 => (a1-a2)*x = b2-b1 => x = (b2-b1)/(a1-a2)
           y = a1 * x + b1

           Filter out intersections that took place in the past.

           Returns integers just to make filtering out points outside the min/max region easier.
        */
        let delta = a1 - a2;

        if delta != 0.0 {
            let x = (b2 - b1) / delta;
            let y = a1 * x + b1;
            if (x - first.pos.x as f64) / first.vel.x as f64 >= 0.0
                && (x - other.pos.x as f64) / other.vel.x as f64 >= 0.0
            {
                Some((x as Number, y as Number))
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn parse_text(text: &str) -> Vec<Shard> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.len() > 0 {
                Some(Shard::from_str(line))
            } else {
                None
            }
        })
        .collect()
}

fn solve(shards: &Vec<Shard>, min: Number, max: Number) -> usize {
    (0..shards.len())
        .map(|i| {
            let first = &shards[i];

            (i + 1..shards.len())
                .filter(move |j| {
                    let other = &shards[*j];
                    if let Some(point) = Shard::intersection_point(first, other) {
                        point.0 >= min && point.0 <= max && point.1 >= min && point.1 <= max
                    } else {
                        false
                    }
                })
                .count()
        })
        .sum()
}

#[test]
fn test_intersect_0() {
    let first = Shard::from_str("19, 13, 30 @ -2, 1, -2");
    let other = Shard::from_str("18, 19, 22 @ -1, -1, -2");

    let point = Shard::intersection_point(&first, &other).unwrap();

    println!("Intersection: {point:?}");

    assert_eq!(point, (14, 15));
}

#[test]
fn test_sample() {
    let sample = "19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3";

    let shards = parse_text(&sample);

    let solution = solve(&shards, 7, 27);

    assert_eq!(solution, 2)
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
    #[arg(long, default_value_t = 200000000000000)]
    min: Number,
    #[arg(long, default_value_t = 400000000000000)]
    max: Number,
}

fn main() {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(&args.input);

    let contents = std::fs::read_to_string(&path).expect("Reading input file");

    let shards = parse_text(&contents);

    let solution = solve(&shards, args.min, args.max);

    println!("{solution}");
}
