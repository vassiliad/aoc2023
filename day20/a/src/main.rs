use clap::{arg, command, Parser};
use std::collections::{BTreeMap, HashSet, VecDeque};

#[derive(Debug, Clone, Hash)]
enum Kind {
    Broadcast,
    FlipFlop(bool),
    Conjunction(Vec<bool>),
}

#[derive(Debug, Clone, Hash)]
struct Module<'s> {
    kind: Kind,
    name: &'s str,
    input: Vec<&'s str>,
    output: Vec<&'s str>,
}

type ModuleMap<'s> = BTreeMap<&'s str, Module<'s>>;

// VV: Keys are names of modules receiving a pulse, values are (Producer module, Pulse)
type ModuleInputs<'s> = BTreeMap<&'s str, VecDeque<(&'s str, bool)>>;

#[derive(Debug, Clone, Hash)]
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
        if let Some(mut module) = modules.get_mut(consumer) {
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
}

fn solve<'s>(network: &'s mut Network) -> u128 {
    let mut pulses_low = 0;
    let mut pulses_high = 0;

    for _ in 0..1000 {
        pulses_low += 1;

        let mut pulses = network.pulse("broadcaster", false, "button");

        while let Some((sender, receiver, pulse)) = pulses.pop_front() {
            (pulses_low, pulses_high) = if pulse {
                (pulses_low, pulses_high + 1)
            } else {
                (pulses_low + 1, pulses_high)
            };

            if network.modules.contains_key(receiver) {
                let new_pulses = network.pulse(receiver, pulse, sender);
                pulses.extend(new_pulses);
            }
        }
    }

    pulses_low * pulses_high
}

#[test]
fn test_sample_1() {
    let sample = "broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a";

    let mut network = parse_text(&sample);

    let solution = solve(&mut network);

    assert_eq!(solution, 32000000)
}

#[test]
fn test_sample_2() {
    let sample = "broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output";

    let mut network = parse_text(&sample);

    let solution = solve(&mut network);

    assert_eq!(solution, 11687500)
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
    let contents = std::fs::read_to_string(path).expect("Reading input file");
    let mut network = parse_text(&contents);

    let solution = solve(&mut network);

    println!("{solution}");
}
