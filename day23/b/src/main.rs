use clap::{arg, command, Parser};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};

#[derive(Copy, Clone, Debug)]
enum Tile {
    Path,
    Forest,
    // Right,
    // Left,
    // Up,
    // Down,
    Portal(usize, usize),
}

struct Maze {
    width: usize,
    height: usize,
    board: Vec<Tile>,
}

#[derive(Debug)]
struct Neighbour {
    id: usize,
    distance: usize,
}

#[derive(Debug)]
struct Node {
    position: usize,
    neighbours: Vec<Neighbour>,
}

impl Maze {
    fn print(&self, pos: usize, visited: &HashSet<usize>) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = x + y * self.width;
                let tile = &self.board[idx];
                let visited = visited.contains(&idx);

                if idx == pos {
                    print!("@")
                } else if !visited && matches!(tile, Tile::Forest) {
                    print!("#")
                } else if !visited && matches!(tile, Tile::Path) {
                    print!(".")
                } else if let &Tile::Portal(side_a, side_b) = tile {
                    let diff = (side_a as isize - side_b as isize).abs() as usize;

                    if diff < self.width {
                        if side_a == idx {
                            if visited {
                                print!("A");
                            } else {
                                print!(">");
                            }
                        } else {
                            if visited {
                                print!("B");
                            } else {
                                print!("<");
                            }
                        }
                    } else {
                        if side_a == idx {
                            if visited {
                                print!("c");
                            } else {
                                print!("v");
                            }
                        } else {
                            if visited {
                                print!("D");
                            } else {
                                print!("^");
                            }
                        }
                    }
                } else if visited {
                    print!("O");
                } else {
                    unreachable!()
                }
            }
            println!("");
        }
    }

    fn print_with_nodes(&self, nodes: &Vec<Node>) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = x + y * self.width;
                let tile = &self.board[idx];

                let is_node = nodes.iter().position(|n| n.position == idx);

                if let Some(pos) = is_node {
                    print!("{pos}")
                } else if matches!(tile, Tile::Forest) {
                    print!("#")
                } else if matches!(tile, Tile::Path) {
                    print!(".")
                } else {
                    unreachable!()
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
                '.' | '>' | '<' | 'v' | '^' => Tile::Path,
                '#' => Tile::Forest,
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

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    pos: usize,
    distance: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance
            .cmp(&self.distance)
            .then_with(|| self.pos.cmp(&other.pos))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct State2 {
    pos: usize,
    path: Vec<usize>,
    distance: usize,
}

fn shortest_path(start: usize, end: usize, maze: &Maze) -> Option<usize> {
    let mut pending = BinaryHeap::from([State {
        pos: start,
        distance: 0,
    }]);

    let mut best = vec![usize::MAX; maze.board.len()];
    best[start] = 0;

    let deltas: [isize; 4] = [1, -1, maze.width as isize, -(maze.width as isize)];

    while let Some(State { pos, distance }) = pending.pop() {
        // println!("At {pos} with {distance}");

        if pos == end {
            return Some(distance);
        }

        if distance > best[pos] {
            // println!("  Skip A");
            continue;
        }

        if pos != start {
            let neighbours = deltas
                .iter()
                .map(|d| (pos as isize + d) as usize)
                .filter(|pos| *pos < maze.board.len())
                .filter(|idx| matches!(maze.board[*idx], Tile::Path))
                .count();
            if neighbours > 2 {
                // VV: This is a junction that we don't care about, we cannot cross it as we want to
                // reach the End without going through any junctions

                // println!("  Skip {neighbours:?} B");
                continue;
            }
        }

        let distance = distance + 1;

        for d in &deltas {
            let pos = pos as isize + d;
            if pos < 0 || pos as usize >= maze.board.len() {
                continue;
            }

            let pos = pos as usize;
            if matches!(maze.board[pos], Tile::Forest) {
                continue;
            }

            if distance < best[pos] {
                pending.push(State { pos, distance });
                best[pos] = distance;
            }
        }
    }

    None
}

fn walk_nodes(nodes: &Vec<Node>) -> usize {
    // VV: 1st node is start, 2nd is the End
    let end = 1;
    let mut pending = vec![State2 {
        pos: 0,
        path: vec![0],
        distance: 0,
    }];

    let mut max_score = 0;

    while let Some(State2 {
        pos,
        path,
        distance,
    }) = pending.pop()
    {
        let node = &nodes[pos];

        if pos == end {
            if distance > max_score {
                println!("With score: {distance}: {path:?}");
                max_score = distance;
            }

            max_score = max_score.max(distance);
            continue;
        }

        for neighbour in node.neighbours.iter() {
            if path.iter().position(|idx| *idx == neighbour.id).is_some() {
                continue;
            }

            let mut path = path.clone();

            path.push(neighbour.id);

            pending.push(State2 {
                pos: neighbour.id,
                path,
                distance: distance + neighbour.distance,
            })
        }
    }

    max_score
}

fn solve(maze: &mut Maze) -> usize {
    let nodes = from_maze(&maze);
    walk_nodes(&nodes)
}

fn from_maze(maze: &Maze) -> Vec<Node> {
    // VV: All junctions (neighbours > 2) are Nodes. There are 2 special nodes too,
    // the starting and ending points.
    // The idea here is to find all junctions and wire together those that can reach each other
    // directly.

    let end = maze.width - 2 + (maze.height - 1) * maze.width;
    let mut nodes = vec![
        Node {
            position: 1,
            neighbours: vec![],
        },
        Node {
            position: end,
            neighbours: vec![],
        },
    ];
    let mut positions_to_nodes: BTreeMap<usize, usize> = BTreeMap::from([(1, 0), (end, 1)]);

    let deltas: [isize; 4] = [1, -1, maze.width as isize, -(maze.width as isize)];

    for y in 1..maze.height - 1 {
        for x in 1..maze.width - 1 {
            let idx = x + y * maze.width;

            if matches!(maze.board[idx], Tile::Forest) {
                continue;
            }

            let neighbours = deltas
                .iter()
                .map(|d| (idx as isize + d) as usize)
                .filter(|idx| *idx < maze.width * maze.height)
                .filter(|pos| matches!(maze.board[*pos], Tile::Path))
                .count();

            if neighbours > 2 {
                positions_to_nodes.insert(nodes.len(), idx);
                nodes.push(Node {
                    position: idx,
                    neighbours: vec![],
                });
            }
        }
    }

    // println!("{nodes:#?}");

    let num_nodes = nodes.len();

    for i in 0..num_nodes {
        for j in i + 1..num_nodes {
            let start = nodes[i].position;
            let end = nodes[j].position;
            let distance = shortest_path(start, end, &maze);

            // println!("Direct distance {start} to {end} is {distance:?}");

            if let Some(distance) = distance {
                nodes[i].neighbours.push(Neighbour { id: j, distance });
                nodes[j].neighbours.push(Neighbour { id: i, distance });
            }

            // unreachable!()
        }
    }

    // println!("{nodes:#?}");

    nodes
}

#[test]
fn test_sample() {
    let sample = "
#.#####################
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

    let mut maze = parse_text(sample);

    let solution = solve(&mut maze);

    assert_eq!(solution, 154);
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

    let mut maze = parse_text(&contents);

    let solution = solve(&mut maze);

    println!("{solution}");
}
