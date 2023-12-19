use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};
use std::collections::BTreeMap;

const MIN_PART: u128 = 1;
const MAX_PART: u128 = 4000;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
enum Condition {
    Less(Kind, u128),
    Greater(Kind, u128),
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

type MyRange = (u128, u128);

#[derive(Debug, Clone)]
struct Part {
    x: MyRange,
    m: MyRange,
    a: MyRange,
    s: MyRange,
}

impl Default for Part {
    fn default() -> Self {
        Part {
            x: (MIN_PART, MAX_PART),
            m: (MIN_PART, MAX_PART),
            a: (MIN_PART, MAX_PART),
            s: (MIN_PART, MAX_PART),
        }
    }
}

type Workflows = BTreeMap<String, Workflow>;

fn parse_text(text: &str) -> Result<Workflows> {
    let mut workflows = Workflows::new();

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
                                condition: Some(Condition::Greater(kind, value)),
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
            // VV: No parts for this puzzle
            break;
        }
    }

    Ok(workflows)
}

fn parse_path(path: &std::path::Path) -> Result<Workflows> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input")?;
    parse_text(&contents)
}

impl Condition {
    fn inverse(&self) -> Self {
        match self {
            Condition::Less(kind, value) => Condition::Greater(kind.clone(), *value - 1),
            Condition::Greater(kind, value) => Condition::Less(kind.clone(), *value + 1),
        }
    }
}

impl Part {
    fn get_mut(&mut self, kind: &Kind) -> &mut MyRange {
        match kind {
            Kind::X => &mut self.x,
            Kind::M => &mut self.m,
            Kind::A => &mut self.a,
            Kind::S => &mut self.s,
        }
    }

    fn constrain_greater(&mut self, kind: &Kind, value: &u128) {
        let member = self.get_mut(kind);
        member.0 = member.0.max(*value + 1)
    }

    fn constrain_less(&mut self, kind: &Kind, value: &u128) {
        let member = self.get_mut(kind);
        member.1 = member.1.min(*value - 1)
    }

    fn constrain(&mut self, condition: &Condition) {
        match condition {
            Condition::Greater(kind, value) => self.constrain_greater(kind, value),
            Condition::Less(kind, value) => self.constrain_less(kind, value),
        }
    }

    fn population(&self) -> Option<u128> {
        if self.x.1 > self.x.0 && self.m.1 > self.m.0 && self.a.1 > self.a.0 && self.s.1 > self.s.0
        {
            let mut product = self.x.1 - self.x.0 + 1;
            product *= self.m.1 - self.m.0 + 1;
            product *= self.a.1 - self.a.0 + 1;
            product *= self.s.1 - self.s.0 + 1;

            Some(product)
        } else {
            None
        }
    }
}

fn process_layers(part: &Part, layers: &[Layer], workflows: &Workflows) -> u128 {
    if layers.len() > 0 {
        let layer = &layers[0];

        let mut part_layer = part.clone();
        let mut part_rest = part.clone();

        if let Some(condition) = &layer.condition {
            part_layer.constrain(condition);
            part_rest.constrain(&condition.inverse());
        }

        let layer = match &layer.decision {
            Decision::Accept => part_layer.population().or(Some(0)).unwrap(),
            Decision::Reject => 0,
            Decision::Delegate(other_wf) => {
                let downstream = &workflows.get(other_wf).unwrap().layers;
                process_layers(&part_layer, downstream, workflows)
            }
        };

        let remaining = if layers.len() > 1 {
            let downstream = &layers[1..];
            process_layers(&part_rest, downstream, workflows)
        } else {
            0
        };

        layer + remaining
    } else {
        if let Some(population) = part.population() {
            population
        } else {
            0
        }
    }
}

fn solve(workflows: &Workflows) -> u128 {
    let part = Part::default();
    let wf = workflows.get("in").unwrap();

    process_layers(&part, &wf.layers, workflows)
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

    let workflows = parse_text(sample).unwrap();

    let solution = solve(&workflows);

    assert_eq!(solution, 167409079868000);
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

    let workflows = parse_path(&path)?;

    let solution = solve(&workflows);

    println!("{solution}");

    Ok(())
}
