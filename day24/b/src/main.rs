use clap::{arg, command, Parser};
use z3::ast::Ast;
use z3::*;

type Number = i64;

#[derive(Debug)]
struct Vector {
    x: Number,
    y: Number,
    z: Number,
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
        let z = tokens
            .next()
            .expect("Get Z")
            .trim()
            .parse::<Number>()
            .expect("Parse Z");

        Self { x, y, z }
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

fn solve(shards: &Vec<Shard>) -> i64 {
    /*VV: It's rambling time.

     There's a line z = ax + by + c (rock) that starts at X, Y, Z (solution) and intersects with *all* shards.

     If we imagine that each Shard forms a line, then our line (rock) crosses all of the Shard lines.
     To find the exact points that form our line we need Some shard-lines and one rock line (solution).

     So we have:
     r: {x = r.pos.x + r.vel.x * t}, {y = r.pos.y + r.vel.y * t}, {z = r.pos.z + r.vel.z * t} (rock)
     i: {X = i.pos.X + i.vel.X * t}, {Y = i.pos.Y + i.vel.Y * t}, {Z = i.pos.Z + i.vel.Z * t} (n-th Shard)

     (capitals are constants)

     For 2 lines r and B and time O we can produce the following

         for r

             x = r.pos.x + r.vel.x *t
             y = r.pos.y + r.vel.y *t
             z = r.pos.z + r.vel.z *t

         for B:
             x = B.pos.X + B.vel.X *t
             y = B.pos.Y + B.vel.Y *t
             z = B.pos.Z + B.vel.Z *t

         B.pos.X + B.vel.X * t = r.pos.x + r.vel.x *t
         B.pos.Y + B.vel.Y * t = r.pos.y + r.vel.y *t
         B.pos.Z + B.vel.Z * t = r.pos.z + r.vel.z *t

         So for r and i at time t
             i.pos.X + i.vel.X * t = r.pos.x + r.vel.x *t    (1)
             i.pos.Y + i.vel.Y * t = r.pos.y + r.vel.y *t    (2)
             i.pos.Z + i.vel.Z * t = r.pos.z + r.vel.z *t    (3)

     We now have 3 equations and 7 unknowns. We need more info

         r and j intersection at time s:
             j.pos.X + j.vel.X * s = r.pos.x + r.vel.x *s    (4)
             j.pos.Y + j.vel.Y * s = r.pos.y + r.vel.y *s    (5)
             j.pos.Z + j.vel.Z * s = r.pos.z + r.vel.z *s    (6)

    We now have 6 equations and 8 unknowns. We need more info

        r and k intersection at time q:
             k.pos.X + k.vel.X * q = r.pos.x + r.vel.x *q    (7)
             k.pos.Y + k.vel.Y * q = r.pos.y + r.vel.y *q    (8)
             k.pos.Z + k.vel.Z * q = r.pos.z + r.vel.z *q    (9)

    We now have 9 equations and 9 unknowns. We can do this !

    Unknowns: t, s, q, r.pos.x, r.pos.y, r.pos.z, r.vel.x, r.vel.y, r.vel.z

    (1) - (4) => (i.pos.x-j.pos.x) + t * i.vel.x - s*j.vel.x = r.vel.x * (t-s)   (10) (r.vel.x, t, s)
    (1) - (7) => (i.pos.x-k.pos.x) + t * i.vel.x - q*k.vel.x = r.vel.x * (t-q)   (11) (r.vel.x, t, q)

    (2) - (5) => (i.pos.y-j.pos.y) + t * i.vel.y - s*j.vel.y = r.vel.y * (t-s)   (12) (r.vel.y, t, s)
    (2) - (8) => (i.pos.y-k.pos.y) + t * i.vel.y - q*k.vel.y = r.vel.y * (t-q)   (13) (r.vel.y, t, q)

    (3) - (6) => (i.pos.z-j.pos.z) + t * i.vel.z - s*j.vel.z = r.vel.z * (t-s)   (14) (r.vel.z, t, s)
    (3) - (9) => (i.pos.z-k.pos.z) + t * i.vel.z - q*k.vel.z = r.vel.z * (t-q)   (15) (r.vel.z, t, q)

    (10) => (t-s) = ((i.pos.x-j.pos.x) + t * i.vel.x - s*j.vel.x) / r.vel.x
    (12) => (t-s) = ((i.pos.y-j.pos.y) + t * i.vel.y - s*j.vel.y) / r.vel.y
    (14) => (t-s) = ((i.pos.z-j.pos.z) + t * i.vel.z - s*j.vel.z) / r.vel.z

    I give up, time to z3.
    */

    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    let solver = Solver::new(&ctx);

    let r_pos_x = ast::Int::new_const(&ctx, "r_pos_x");
    let r_pos_y = ast::Int::new_const(&ctx, "r_pos_y");
    let r_pos_z = ast::Int::new_const(&ctx, "r_pos_z");

    let r_vel_x = ast::Int::new_const(&ctx, "r_vel_x");
    let r_vel_y = ast::Int::new_const(&ctx, "r_vel_y");
    let r_vel_z = ast::Int::new_const(&ctx, "r_vel_z");

    let threshold = ast::Int::from_i64(&ctx, 0);

    for (idx, shard) in shards.iter().take(3).enumerate() {
        let i_pos_x = ast::Int::from_i64(&ctx, shard.pos.x);
        let i_pos_y = ast::Int::from_i64(&ctx, shard.pos.y);
        let i_pos_z = ast::Int::from_i64(&ctx, shard.pos.z);

        let i_vel_x = ast::Int::from_i64(&ctx, shard.vel.x);
        let i_vel_y = ast::Int::from_i64(&ctx, shard.vel.y);
        let i_vel_z = ast::Int::from_i64(&ctx, shard.vel.z);

        let i_t = ast::Int::new_const(&ctx, format!("t{idx}"));

        // VV: Intersections cannot happen in the past
        solver.assert(&i_t.gt(&threshold));

        let rock_pos_x_t = r_pos_x.clone() + r_vel_x.clone() * i_t.clone();
        let rock_pos_y_t = r_pos_y.clone() + r_vel_y.clone() * i_t.clone();
        let rock_pos_z_t = r_pos_z.clone() + r_vel_z.clone() * i_t.clone();

        let shard_pos_x_t = i_pos_x.clone() + i_vel_x.clone() * i_t.clone();
        let shard_pos_y_t = i_pos_y.clone() + i_vel_y.clone() * i_t.clone();
        let shard_pos_z_t = i_pos_z.clone() + i_vel_z.clone() * i_t.clone();

        solver.assert(&rock_pos_x_t._eq(&shard_pos_x_t));
        solver.assert(&rock_pos_y_t._eq(&shard_pos_y_t));
        solver.assert(&rock_pos_z_t._eq(&shard_pos_z_t));
    }

    let model = solver.get_model().expect("Solve the problem");

    let rock_x = model
        .eval(&r_pos_x, true)
        .expect("Rock position x")
        .as_i64()
        .expect("Extract rock position x from z3");
    let rock_y = model
        .eval(&r_pos_y, true)
        .expect("Rock position y")
        .as_i64()
        .expect("Extract rock position y from z3");
    let rock_z = model
        .eval(&r_pos_z, true)
        .expect("Rock position z")
        .as_i64()
        .expect("Extract rock position z from z3");

    rock_x + rock_y + rock_z
}

#[test]
fn test_sample() {
    let sample = "19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3";

    let shards = parse_text(&sample);

    let solution = solve(&shards);

    assert_eq!(solution, 47)
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

    let shards = parse_text(&contents);

    let solution = solve(&shards);

    println!("{solution}");
}
