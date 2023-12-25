use clap::{arg, command, Parser};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Debug, Clone)]
struct Node {
    name: String,
    neighbours: HashSet<String>,
}

type Network = HashMap<String, Node>;

fn parse_text(text: &str) -> Network {
    let mut nodes = Network::new();

    let lines = text.lines().filter_map(|line| {
        let line = line.trim();
        if line.len() > 0 {
            Some(line)
        } else {
            None
        }
    });
    for line in lines {
        let (name, others) = line.split_once(":").expect("Partition node: <neighbours>");
        let name = name.trim();
        let neighbours = others.trim().split(' ').filter_map(|word| {
            let word = word.trim();
            if word.len() > 0 {
                Some(word)
            } else {
                None
            }
        });

        let current = name.to_string();

        nodes.entry(name.to_string()).or_insert(Node {
            name: name.to_string(),
            neighbours: HashSet::new(),
        });

        for name in neighbours {
            {
                let neighbour = nodes.entry(name.to_string()).or_insert(Node {
                    name: name.to_string(),
                    neighbours: HashSet::new(),
                });
                neighbour.neighbours.insert(current.clone());
            }
            {
                nodes.entry(current.clone()).and_modify(|e| {
                    e.neighbours.insert(name.to_string());
                });
            }
        }
    }

    nodes
}

fn dot(nodes: &Network) {
    println!("graph G {{");

    for (name, node) in nodes {
        for n_name in &node.neighbours {
            if name > n_name {
                println!("{name} -- {n_name}");
            }
        }
    }

    println!("}}");
}

#[derive(Clone, Eq, PartialEq)]
struct State<'s> {
    cost: usize,
    position: &'s String,
    path: Vec<&'s String>,
}

impl Ord for State<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
            .then_with(|| self.path.cmp(&other.path))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for State<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn walk_between_nodes(nodes: &Network, start: &String, end: &String) -> Vec<String> {
    let mut pending = BinaryHeap::from([State {
        position: start,
        cost: 0,
        path: vec![start],
    }]);

    let mut distances = HashMap::new();

    while let Some(State {
        position,
        cost,
        path,
    }) = pending.pop()
    {
        if position == end {
            return path.iter().map(|x| x.to_string()).collect();
        }

        if let Some(best) = distances.get(position) {
            if cost > *best {
                continue;
            }
        }
        let cost = cost + 1;

        for next in nodes.get(position).unwrap().neighbours.iter() {
            if let Some(best) = distances.get(next) {
                if *best <= cost {
                    continue;
                }
            }

            let mut path = path.clone();
            path.push(next);

            distances.insert(next.clone(), cost);
            pending.push(State {
                position: next,
                cost,
                path,
            });
        }
    }

    unreachable!("Cannot walk from {start} to {end}");
}

fn count_subgraph_populations(nodes: &Network) -> (usize, usize) {
    let mut visited = HashSet::new();
    let start = nodes.iter().nth(0).unwrap().0;

    let mut pending = vec![start];

    while let Some(node) = pending.pop() {
        for next in nodes.get(node).unwrap().neighbours.iter() {
            if !visited.contains(next) {
                visited.insert(next);

                pending.push(next);
            }
        }
    }

    (visited.len(), nodes.len() - visited.len())
}

fn find_most_used_edge(nodes: &mut Network) -> (String, String) {
    let mut ret: HashMap<(String, String), usize> = HashMap::new();

    // VV: This gets quite slow when you have a large number of Nodes
    // It doesn't exploit the fact that a path provides information
    // about multiple start,end. There should be a way to avoid calculating the distance
    // between 2 nodes multiple times but lunch is ready and I already have my solution :)
    for (_idx, (start, _)) in nodes.iter().enumerate() {
        for (end, _) in nodes.iter() {
            if start < end {
                let path = walk_between_nodes(nodes, start, end);

                for i in 1..path.len() {
                    let start = &path[i - 1];
                    let end = &path[i];

                    let (start, end) = if start < end {
                        (start, end)
                    } else {
                        (end, start)
                    };

                    ret.entry((start.clone(), end.clone()))
                        .and_modify(|e| *e += 1)
                        .or_insert(0);
                }
            }
        }
    }

    let mut max: Option<((String, String), usize)> = None;

    for ((start, end), times_used) in ret {
        max = if let Some((_edge, best)) = &max {
            if *best < times_used {
                Some(((start, end), times_used))
            } else {
                max
            }
        } else {
            Some(((start, end), times_used))
        };
    }

    max.unwrap().0
}

/// if you walk the paths between all nodes you will find yourself crossing the edges that
/// separate the graph most frequently. Find the min distance between all nodes of the graph
/// 3 times. Each time remove the edge that was most frequently used.
fn solve(nodes: &mut Network) -> usize {
    for _ in 0..3 {
        let edge = find_most_used_edge(nodes);
        println!("Removed {edge:?}");

        for (_, node) in nodes.iter_mut() {
            if edge.0 == node.name || edge.1 == node.name {
                node.neighbours.retain(|x| *x != edge.0 && *x != edge.1);
            }
        }
    }

    let (graph1, graph2) = count_subgraph_populations(nodes);

    graph1 * graph2
}

#[test]
fn test_sample() {
    let sample = "jqt: rhn xhk nvd
rsh: frs pzl lsr
xhk: hfx
cmg: qnr nvd lhk bvb
rhn: xhk bvb hfx
bvb: xhk hfx
pzl: lsr hfx nvd
qnr: nvd
ntq: jqt hfx bvb xhk
nvd: lhk
lsr: lhk
rzs: qnr cmg lsr rsh
frs: qnr lhk lsr";

    let mut nodes = parse_text(sample);

    assert_eq!(solve(&mut nodes), 54)
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

    let contents = std::fs::read_to_string(&path).expect("Read input file");
    let mut nodes = parse_text(&contents);

    let solution = solve(&mut nodes);

    println!("{solution}")
}
