use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};
use std::collections::BTreeMap;

#[derive(Debug)]
enum Kind {
    X,
    M,
    A,
    S,
}

impl Kind {
    fn from_str(text: &str) -> Result<Kind> {
        let kind = match text {
            "x" => Kind::X,
            "m" => Kind::M,
            "a" => Kind::A,
            "s" => Kind::S,
            _ => bail!("Invalid Kind {text}"),
        };

        Ok(kind)
    }
}

#[derive(Debug)]
enum Decision {
    Accept,
    Reject,
    Delegate(String),
}

#[derive(Debug)]
enum Condition {
    Less(Kind, u128),
    More(Kind, u128),
}

#[derive(Debug)]
struct Layer {
    condition: Option<Condition>,
    decision: Decision,
}

#[derive(Debug)]
struct Workflow {
    name: String,
    layers: Vec<Layer>,
}

#[derive(Debug, Default)]
struct Part {
    x: u128,
    m: u128,
    a: u128,
    s: u128,
}

impl Part {
    fn set(&mut self, kind: &Kind, value: u128) {
        match kind {
            Kind::X => self.x = value,
            Kind::M => self.m = value,
            Kind::A => self.a = value,
            Kind::S => self.s = value,
        }
    }

    fn get(&self, kind: &Kind) -> u128 {
        match kind {
            Kind::X => self.x,
            Kind::M => self.m,
            Kind::A => self.a,
            Kind::S => self.s,
        }
    }

    fn value(&self) -> u128 {
        return self.x + self.m + self.a + self.s;
    }
}

type Workflows = BTreeMap<String, Workflow>;

fn parse_text(text: &str) -> Result<(Vec<Part>, Workflows)> {
    let mut workflows = Workflows::new();
    let mut parts = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.len() == 0 {
            continue;
        }

        if !line.starts_with("{") {
            let (name, workflow) = line.split_once("{").unwrap();
            let recipe = workflow
                .strip_suffix("}")
                .with_context(|| "Strip suffix } from workflow definition")
                .unwrap();
            let layers = recipe
                .split(',')
                .map(|condition| {
                    if let Some((condition, delegate)) = condition.split_once(":") {
                        let decision = match delegate {
                            "A" => Decision::Accept,
                            "R" => Decision::Reject,
                            _ => Decision::Delegate(delegate.to_string()),
                        };

                        if let Some((kind, value)) = condition.split_once("<") {
                            let kind = Kind::from_str(kind).unwrap();
                            let value = value
                                .parse()
                                .with_context(|| "Parsing Value condition")
                                .unwrap();

                            Layer {
                                condition: Some(Condition::Less(kind, value)),
                                decision,
                            }
                        } else if let Some((kind, value)) = condition.split_once(">") {
                            let kind = Kind::from_str(kind).unwrap();
                            let value = value
                                .parse()
                                .with_context(|| "Parsing Value condition")
                                .unwrap();

                            Layer {
                                condition: Some(Condition::More(kind, value)),
                                decision,
                            }
                        } else {
                            panic!("Invalid condition {condition}");
                        }
                    } else {
                        let decision = match condition {
                            "A" => Decision::Accept,
                            "R" => Decision::Reject,
                            _ => Decision::Delegate(condition.to_string()),
                        };

                        Layer {
                            condition: None,
                            decision,
                        }
                    }
                })
                .collect();
            let name = name.to_string();
            workflows.insert(name.clone(), Workflow { name, layers });
        } else {
            let mut part = Part::default();
            let line = line
                .strip_suffix('}')
                .with_context(|| "Strip suffix }")
                .unwrap();
            let line = line
                .strip_prefix('{')
                .with_context(|| "Strip prefix {")
                .unwrap();

            for member in line.split(',') {
                let member = member.trim();
                if let Some((kind, value)) = member.split_once('=') {
                    let kind = Kind::from_str(kind).unwrap();
                    let value = value
                        .parse()
                        .with_context(|| "Parsing Value member")
                        .unwrap();
                    part.set(&kind, value);
                } else {
                    bail!("Invalid member definition {member}")
                }
            }

            parts.push(part);
        }
    }

    Ok((parts, workflows))
}

fn parse_path(path: &std::path::Path) -> Result<(Vec<Part>, Workflows)> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input")?;
    parse_text(&contents)
}

fn process_part(part: &Part, workflows: &Workflows) -> bool {
    let mut wf_name = "in".to_string();

    loop {
        // println!("{part:?} with wf {wf_name}");
        let wf = workflows.get(&wf_name).unwrap();

        for (_idx, layer) in wf.layers.iter().enumerate() {
            let condition = if let Some(condition) = &layer.condition {
                match condition {
                    Condition::Less(kind, value) => part.get(kind) < *value,
                    Condition::More(kind, value) => part.get(kind) > *value,
                }
            } else {
                true
            };

            if condition {
                // println!(" Layer {idx} = {layer:?}");
                match &layer.decision {
                    Decision::Accept => return true,
                    Decision::Reject => return false,
                    Decision::Delegate(other_wf) => {
                        wf_name = other_wf.clone();
                        break;
                    }
                }
            }
        }
    }
}

fn solve(parts: &Vec<Part>, workflows: &Workflows) -> u128 {
    parts
        .iter()
        .filter(|part| process_part(*part, workflows))
        .map(|part| part.value())
        .sum()
}

#[test]
fn test_sample() {
    let sample = "px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}";

    let (parts, workflows) = parse_text(sample).unwrap();

    // println!("Workflows: {workflows:#?}");
    // println!("Parts: {parts:#?}");

    let solution = solve(&parts, &workflows);

    assert_eq!(solution, 19114);
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

    let (parts, workflows) = parse_path(&path)?;

    let solution = solve(&parts, &workflows);

    println!("{solution}");

    Ok(())
}
