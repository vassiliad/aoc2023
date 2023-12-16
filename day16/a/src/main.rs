use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};

const NO_BEAMS: u8 = 0;

enum Beam {
    None = 0,
    North = 1,
    South = 2,
    West = 4,
    East = 8,
}

type Bunch = u8;

#[derive(Copy, Clone)]
enum Cell {
    Wall,
    Empty(Bunch),
    // VV: An Eastward moving Beam that hits this mirror moves North (i.e. /), a Westward one moves South
    //     A Southward moving beam moves West, a Northward one moves East
    MirrorNorth(Bunch),
    // VV: A Westward moving Beam that hits this mirror moves North (i.e. \), am Eastward one moves South
    //     A Southward moving beam moves East, a Northward one moves West
    MirrorSouth(Bunch),
    // VV: A Horizontal splitter splits a beam moving East/West into 2 beams: one moving North and the other South
    SplitterHorizontal(Bunch),
    // VV: A Vertical splitter splits a beam moving North/South into 2 beams: one moving East and the other West
    SplitterVertical(Bunch),
}

trait MergeBeams {
    fn merge(a: Bunch, b: Bunch) -> Bunch;
}

impl MergeBeams for Bunch {
    fn merge(a: Bunch, b: Bunch) -> Bunch {
        a | b
    }
}

struct Cave {
    width: isize,
    height: isize,
    board: Vec<Cell>,
}

fn parse_text(text: &str) -> Result<Cave> {
    let mut width = 0;
    let mut board = vec![];

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }

        width = line.len();

        for c in line.chars() {
            let cell = match c {
                '.' => Cell::Empty(NO_BEAMS),
                '|' => Cell::SplitterVertical(NO_BEAMS),
                '-' => Cell::SplitterHorizontal(NO_BEAMS),
                '/' => Cell::MirrorNorth(NO_BEAMS),
                '\\' => Cell::MirrorSouth(NO_BEAMS),
                _ => bail!("Invalid character {c}"),
            };

            board.push(cell);
        }
    }

    Ok(Cave {
        width: width as isize,
        height: (board.len() / width) as isize,
        board,
    })
}

fn parse_path(path: &std::path::Path) -> Result<Cave> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading mine file")?;
    parse_text(&contents)
}

trait BunchOfBeams {
    fn count_beams(&self) -> u128;
}

impl BunchOfBeams for Bunch {
    fn count_beams(&self) -> u128 {
        self.count_ones() as u128
    }
}

impl Cave {
    fn print(&self) {
        for y in 0..self.height {
            let mut line_map = "".to_string();
            let mut line_charged = "".to_string();

            for x in 0..self.width {
                let curr = &self.board[(x + y * self.width) as usize];
                let character = match curr {
                    Cell::Wall => unreachable!("What even is a wall?"),
                    Cell::Empty(_) => ".",
                    Cell::MirrorNorth(_) => "/",
                    Cell::MirrorSouth(_) => "\\",
                    Cell::SplitterHorizontal(_) => "-",
                    Cell::SplitterVertical(_) => "|",
                };
                line_map = format!("{line_map}{character}");

                let bunch = extract_beams(&curr).unwrap().count_beams();
                line_charged = format!("{line_charged}{}", if bunch == 0 { "." } else { "#" });
            }

            println!("{line_map} {line_charged}");
        }
        println!("");
    }
}

impl Beam {
    fn to_u8(&self) -> u8 {
        match self {
            Beam::None => 0,
            Beam::North => 1,
            Beam::South => 2,
            Beam::West => 4,
            Beam::East => 8,
        }
    }

    fn next_location(&self, x: isize, y: isize, cave: &Cave) -> Option<(isize, isize)> {
        match self {
            Beam::North => {
                if y == 0 {
                    None
                } else {
                    Some((x, y - 1))
                }
            }
            Beam::South => {
                if y == cave.height - 1 {
                    None
                } else {
                    Some((x, y + 1))
                }
            }
            Beam::West => {
                if x == 0 {
                    None
                } else {
                    Some((x - 1, y))
                }
            }
            Beam::East => {
                if x == cave.width - 1 {
                    None
                } else {
                    Some((x + 1, y))
                }
            }
            _ => unreachable!(),
        }
    }

    fn transform(&self, cell: &Cell) -> Option<Bunch> {
        match cell {
            Cell::Wall => None,
            Cell::Empty(_) => Some(self.to_u8()),
            // VV: An Eastward moving Beam that hits this mirror moves North (i.e. /), a Westward one moves South
            //     A Southward moving beam moves West, a Northward one moves East
            Cell::MirrorNorth(_) => match self {
                Beam::East => Some(Beam::North.to_u8()),
                Beam::West => Some(Beam::South.to_u8()),
                Beam::South => Some(Beam::West.to_u8()),
                Beam::North => Some(Beam::East.to_u8()),
                _ => Some(self.to_u8()),
            },
            // VV: A Westward moving Beam that hits this mirror moves North (i.e. \), am Eastward one moves South
            //     A Southward moving beam moves East, a Northward one moves West
            Cell::MirrorSouth(_) => match self {
                Beam::West => Some(Beam::North.to_u8()),
                Beam::East => Some(Beam::South.to_u8()),
                Beam::South => Some(Beam::East.to_u8()),
                Beam::North => Some(Beam::West.to_u8()),
                _ => Some(self.to_u8()),
            },
            // VV: A Horizontal splitter splits a beam moving North/South into 2 beams: one moving East and the other West
            Cell::SplitterHorizontal(_) => match self {
                Beam::North | Beam::South => Some(Beam::West.to_u8() | Beam::East.to_u8()),
                _ => Some(self.to_u8()),
            },
            // VV: A Vertical splitter splits a beam moving East/West into 2 beams: one moving North and the other South
            Cell::SplitterVertical(_) => match self {
                Beam::East | Beam::West => Some(Beam::North.to_u8() | Beam::South.to_u8()),
                _ => Some(self.to_u8()),
            },
        }
    }
}

fn extract_beams(curr: &Cell) -> Option<Bunch> {
    match curr {
        Cell::Wall => None,
        Cell::Empty(bunch)
        | Cell::MirrorNorth(bunch)
        | Cell::MirrorSouth(bunch)
        | Cell::SplitterHorizontal(bunch)
        | Cell::SplitterVertical(bunch) => Some(*bunch),
    }
}

fn update_bunch_in_cell(curr: &mut Cell, new_bunch: Bunch) {
    match curr {
        Cell::Wall => unreachable!("Cannot update a wall"),
        Cell::Empty(bunch) => *bunch = new_bunch,
        Cell::MirrorNorth(bunch) => *bunch = new_bunch,
        Cell::MirrorSouth(bunch) => *bunch = new_bunch,
        Cell::SplitterHorizontal(bunch) => *bunch = new_bunch,
        Cell::SplitterVertical(bunch) => *bunch = new_bunch,
    }
}

fn propagate_bunch(
    bunch: &Bunch,
    cave: &mut Cave,
    x: isize,
    y: isize,
    only_through_empty: bool,
) -> bool {
    let mut changed = false;
    let curr_cell = cave.board[(x + y * cave.width) as usize];

    for beam in [Beam::North, Beam::South, Beam::West, Beam::East] {
        if beam.to_u8() & bunch == 0 {
            continue;
        }
        if let Some((bx, by)) = beam.next_location(x, y, cave) {
            let next_cell = &mut cave.board[(bx + by * cave.width) as usize];
            let mut next_beams = extract_beams(&next_cell).unwrap();

            if !only_through_empty | matches!(curr_cell, Cell::Empty(_)) {
                if beam.to_u8() & next_beams == 0 {
                    next_beams |= beam.to_u8();

                    update_bunch_in_cell(next_cell, next_beams);
                    changed = true;
                }
            }
        }
    }
    changed
}

fn solve(cave: &mut Cave) -> u128 {
    let first = &mut cave.board[0];

    *first = match first {
        Cell::Wall => unreachable!("Top-left cell cannot be a wall"),
        Cell::Empty(0) => Cell::Empty(Beam::East.to_u8()),
        Cell::SplitterVertical(0) => Cell::SplitterVertical(Beam::East.to_u8()),
        Cell::SplitterHorizontal(0) => Cell::SplitterHorizontal(Beam::East.to_u8()),
        Cell::MirrorNorth(0) => Cell::MirrorNorth(Beam::East.to_u8()),
        Cell::MirrorSouth(0) => Cell::MirrorSouth(Beam::East.to_u8()),
        _ => unreachable!("Invalid mine"),
    };

    let mut changed = true;

    while changed {
        changed = false;

        for y in 0..cave.height {
            for x in 0..cave.width {
                let curr_cell = cave.board[(x + y * cave.width) as usize].clone();

                let curr_bunch = extract_beams(&curr_cell).unwrap();

                // VV: Propagate the beams in the current cell to their next destination
                changed |= propagate_bunch(&curr_bunch, cave, x, y, true);
                let mut next_bunch = Bunch::default();
                for beam in [Beam::North, Beam::South, Beam::West, Beam::East] {
                    if beam.to_u8() & curr_bunch == 0 {
                        continue;
                    }
                    // VV: Calculate the next positions for the beam in the current bunch
                    if let Some(next_bunch_for_beam) = beam.transform(&curr_cell) {
                        next_bunch |= next_bunch_for_beam
                    }
                }

                // VV: Propagate the beams in the current cell to their next destination
                changed |= propagate_bunch(&next_bunch, cave, x, y, false);
            }
        }
    }

    cave.board
        .iter()
        .map(|x| (extract_beams(x).or(Some(0)).unwrap().count_beams() > 0) as u128)
        .sum()
}

#[test]
fn test_sample() {
    let sample = ".|...\\....
|.-.\\.....
.....|-...
........|.
..........
.........\\
..../.\\\\..
.-.-/..|..
.|....-|.\\
..//.|....";

    let mut cave = parse_text(sample).unwrap();
    assert_eq!(solve(&mut cave), 46)
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

    let mut cave = parse_path(&path)?;

    let solution = solve(&mut cave);

    println!("{solution}");

    Ok(())
}
