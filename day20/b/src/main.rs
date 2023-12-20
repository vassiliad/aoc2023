use clap::{arg, command, Parser};
use num::integer::lcm;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Broadcast,
    FlipFlop(bool),
    Conjunction(Vec<bool>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Module<'s> {
    kind: Kind,
    name: &'s str,
    input: Vec<&'s str>,
    output: Vec<&'s str>,
}

type ModuleMap<'s> = BTreeMap<&'s str, Module<'s>>;

// VV: Keys are names of modules receiving a pulse, values are (Producer module, Pulse)
type ModuleInputs<'s> = BTreeMap<&'s str, VecDeque<(&'s str, bool)>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
struct Network<'s> {
    modules: ModuleMap<'s>,
    // inputs: ModuleInputs<'s>,
}

fn parse_text(text: &str) -> Network {
    let mut modules = ModuleMap::new();
    let mut inputs = ModuleInputs::new();
    let mut consumes: BTreeMap<&str, HashSet<&str>> = BTreeMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }

        let (name, output) = line.split_once("->").unwrap();
        let name = name.trim();
        let output = output.trim();

        let (kind, name) = if let Some(flip_flop) = name.strip_prefix("%") {
            (Kind::FlipFlop(false), flip_flop)
        } else if let Some(conjunction) = name.strip_prefix("&") {
            (Kind::Conjunction(vec![]), conjunction)
        } else if name == "broadcaster" {
            (Kind::Broadcast, name)
        } else {
            unreachable!("Unknown kind for module {name}")
        };

        let output: Vec<&str> = output.split(',').map(|x| x.trim()).collect();

        inputs.insert(name, VecDeque::new());

        for x in output.iter() {
            if !inputs.contains_key(x) {
                inputs.insert(x, VecDeque::new());
            }

            if let Some(consumer) = consumes.get_mut(x) {
                consumer.insert(name);
            } else {
                consumes.insert(x, HashSet::from([name]));
            }
        }

        let module = Module {
            name,
            output,
            kind,
            input: Vec::new(),
        };

        modules.insert(name, module);
    }

    for (consumer, inputs) in consumes.iter() {
        if let Some(module) = modules.get_mut(consumer) {
            for input in inputs.iter() {
                module.input.push(*input);
            }
        }
    }

    for (_, module) in modules.iter_mut() {
        if let Kind::Conjunction(memory) = &mut module.kind {
            *memory = vec![false; module.input.len()];
        }
    }

    Network { modules }
}

impl<'s> Network<'s> {
    fn pulse(
        &mut self,
        receiver: &'s str,
        pulse: bool,
        sender: &'_ str,
    ) -> VecDeque<(&'s str, &'s str, bool)> {
        let module = self.modules.get_mut(receiver).unwrap();

        let pulse = match &mut module.kind {
            Kind::Broadcast => Some(false),
            Kind::FlipFlop(last) => {
                if pulse {
                    None
                } else {
                    let flipped = !*last;
                    module.kind = Kind::FlipFlop(flipped);
                    Some(flipped)
                }
            }
            Kind::Conjunction(memory) => {
                let producer_idx = module.input.iter().position(|x| *x == sender).unwrap();
                memory[producer_idx] = pulse;

                if let Some(_) = memory.iter().position(|x| *x == false) {
                    Some(true)
                } else {
                    Some(false)
                }
            }
        };

        let mut pulses = VecDeque::new();

        if let Some(pulse) = pulse {
            // VV: Current module is sending pulses to other receivers
            let sender = receiver;

            for receiver in module.output.iter() {
                pulses.push_back((sender, *receiver, pulse));
            }
        }

        pulses
    }

    fn full_name(&self, node: &'s str) -> String {
        if let Some(module) = self.modules.get(node) {
            match module.kind {
                Kind::Broadcast => node.to_string(),
                Kind::Conjunction(_) => format!("\"&{node}\""),
                Kind::FlipFlop(_) => format!("\"\\%{node}\""),
            }
        } else {
            node.to_string()
        }
    }
}

fn calculate_cycle<'s>(network: &'s mut Network<'s>, start: &'s str, end: &'s str) -> (u128, u128) {
    let mut states: HashMap<Network, u128> = HashMap::new();

    states.insert(network.clone(), 0);

    for step in 1.. {
        let mut pulses: VecDeque<(&'s str, &'s str, bool)> =
            VecDeque::from([("broadcaster", start, false)]);

        while let Some((sender, receiver, pulse)) = pulses.pop_front() {
            if pulse && sender == end {
                // VV: In my input I get a Pulse right at the end of a cycle
                // println!("Step: {step}");
            }

            if network.modules.contains_key(receiver) {
                let new_pulses = network.pulse(receiver, pulse, sender);
                pulses.extend(new_pulses);
            }
        }

        if let Some(cycle_start) = states.get(&network) {
            return (*cycle_start, step);
        }

        states.insert(network.clone(), step);
    }

    unreachable!()
}

fn find_sub_graphs_start_end<'s>(network: &'s Network) -> Vec<(&'s str, &'s str)> {
    let trees_start = {
        let broadcaster = network.modules.get("broadcaster").unwrap();
        broadcaster.output.clone()
    };

    let penultimate: Vec<_> = network
        .modules
        .iter()
        .filter(|(_name, module)| {
            module
                .output
                .iter()
                .position(|output_name| *output_name == "rx")
                .is_some()
        })
        .map(|(_name, module)| module)
        .collect();

    if penultimate.len() != 1 {
        unreachable!("Good luck");
    }

    if !matches!(penultimate[0].kind, Kind::Conjunction(_)) {
        unreachable!("Good luck");
    }

    let trees_end: Vec<&str> = network
        .modules
        .iter()
        .filter(|(_name, module)| {
            module
                .output
                .iter()
                .position(|output_name| *output_name == penultimate[0].name)
                .is_some()
        })
        .map(|(name, _module)| *name)
        .collect();

    let mut start_end = vec![];

    for node_end in &trees_end {
        let mut edges = vec![];
        let mut visited = HashSet::new();

        let node = network.modules.get(node_end).unwrap();
        for input_name in node.input.iter() {
            edges.push((*node_end, *input_name));
        }

        while let Some(edge) = edges.pop() {
            if trees_start.contains(&edge.1) {
                start_end.push((edge.1, *node_end));
                break;
            }

            if visited.contains(&edge) {
                continue;
            }
            visited.insert(edge);

            let node = network.modules.get(edge.1).unwrap();
            for input_name in node.input.iter() {
                edges.push((edge.1, *input_name));
                if trees_start.contains(input_name) {
                    break;
                }
            }
        }
    }

    return start_end;
}

fn partition<'s>(
    network: &'s Network,
    start_end: &Vec<(&'s str, &'s str)>,
) -> Vec<(&'s str, &'s str, Network<'s>)> {
    let mut ret: Vec<(&'s str, &'s str, Network<'s>)> = Vec::new();

    for &(start, end) in start_end.iter() {
        let mut modules = ModuleMap::new();

        let mut pending = vec![
            network.modules.get(start).unwrap().clone(),
            network.modules.get(end).unwrap().clone(),
        ];

        while let Some(module) = pending.pop() {
            if !modules.contains_key(module.name) {
                if module.name != end {
                    for &output in &module.output {
                        if let Some(consumer) = network.modules.get(output) {
                            pending.push(consumer.clone());
                        }
                    }
                }

                modules.insert(module.name, module);
            }
        }

        let subnetwork = Network { modules };

        ret.push((start, end, subnetwork));
    }

    ret
}

fn solve(network: &Network) -> u128 {
    let start_end = find_sub_graphs_start_end(network);
    let mut subnetworks = partition(network, &start_end);

    let mut ret = 1;
    for (start, end, subnetwork) in subnetworks.iter_mut() {
        let (cycle_start, cycle_end) = calculate_cycle(subnetwork, start, end);

        // println!("{start}->{end} with cycle_start {cycle_start} and cycle_end {cycle_end}");
        ret = lcm(ret, cycle_end - cycle_start);
    }

    ret
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum Task {
    #[default]
    Solve,
    Dot,
}

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,

    #[arg(long, short, value_enum, default_value_t=Task::Solve)]
    task: Task,
}

fn main() {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(&args.input);
    let contents = std::fs::read_to_string(path).expect("Reading input file");
    let mut network = parse_text(&contents);

    match args.task {
        Task::Solve => {
            let solution = solve(&mut network);

            println!("{solution}");
        }
        Task::Dot => {
            println!("digraph G {{");

            println!("rx [color=blue]");

            for (_, module) in network.modules.iter() {
                let module_name = network.full_name(module.name);

                for output in module.output.iter() {
                    let output = network.full_name(output);

                    println!("{module_name} -> {output}");
                }
            }

            println!("}}");
        }
    }
}
