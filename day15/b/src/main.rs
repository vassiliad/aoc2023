use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};
use std::collections::{LinkedList, VecDeque};

#[derive(Debug)]
enum Character {
    Letter(char),
    Digit(u8),
    Equal,
    Minus,
}

type Word = Vec<Character>;

#[derive(Debug)]
struct Instruction {
    box_index: usize,
    label: String,
    add_or_remove: Option<u8>,
}

impl Instruction {
    fn from_word(word: &Word) -> Self {
        let box_index = word.to_hash() as usize;
        let label = word
            .iter()
            .filter(|c| matches!(c, Character::Letter(_)))
            .map(|c| match c {
                Character::Letter(c) => c,
                _ => unreachable!(),
            })
            .into_iter();
        let label = String::from_iter(label);

        let add_or_remove = match word.last().unwrap() {
            Character::Digit(focal_length) => Some(*focal_length),
            Character::Minus => None,
            last => unreachable!("Unexpected character {last:?}"),
        };

        Self {
            box_index,
            label,
            add_or_remove,
        }
    }
}

impl Character {
    fn ascii(&self) -> u128 {
        match self {
            Character::Letter(c) => *c as u128,
            Character::Digit(d) => '0' as u128 + *d as u128,
            Character::Equal => '=' as u128,
            Character::Minus => '-' as u128,
        }
    }
}

#[derive(Clone, Debug)]
struct Lens {
    label: String,
    focal_length: u8,
}

type Box = VecDeque<Lens>;
type Boxes = [Box; 256];

trait MyHash {
    fn to_hash(&self) -> u128;
}

trait MyRemove {
    type Object;
    fn delete(&mut self, index: usize) -> Option<Self::Object>;
}

impl MyHash for Word {
    fn to_hash(&self) -> u128 {
        self.iter()
            .filter(|x| matches!(x, Character::Letter(_)))
            .fold(0u128, |hash, curr| ((hash + curr.ascii()) * 17) % 256)
    }
}

impl<T> MyRemove for LinkedList<T> {
    type Object = T;

    /// An implementation for remove() (nightly feature)
    fn delete(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        let mut trailing = self.split_off(index);
        let removed = trailing.pop_front();
        self.append(&mut trailing);

        removed
    }
}

fn parse_text(text: &str) -> Result<Vec<Word>> {
    let mut words = vec![];

    for many in text.trim().split(',').map(|x| x.trim()) {
        if text.len() == 0 {
            bail!("Consecutive , are invalid");
        }

        let word: Word = many
            .chars()
            .map(|c| match c {
                'a'..='z' => Character::Letter(c),
                '0'..='9' => Character::Digit(c.to_digit(10).unwrap() as u8),
                '-' => Character::Minus,
                '=' => Character::Equal,
                _ => unimplemented!("Unexpected {c}"),
            })
            .collect();

        words.push(word);
    }

    Ok(words)
}

fn parse_path(path: &std::path::Path) -> Result<Vec<Word>> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading input file")?;
    parse_text(&contents)
}

fn solve(words: &Vec<Word>) -> u128 {
    const EMPTY_BOX: Box = Box::new();
    let mut boxes: Boxes = [EMPTY_BOX; 256];
    let instructions: Vec<_> = words.iter().map(|w| Instruction::from_word(w)).collect();

    for instr in instructions.iter() {
        let b = &mut boxes[instr.box_index];

        let lens_position = b.iter().position(|x| x.label == instr.label);

        if let Some(focal_length) = instr.add_or_remove {
            // VV: Replace the lens with the same label, or put one at the top of the stack

            if let Some(lens_index) = lens_position {
                b[lens_index].focal_length = focal_length;
            } else {
                b.push_back(Lens {
                    focal_length,
                    label: instr.label.clone(),
                })
            }
        } else if let Some(lens_index) = lens_position {
            // VV: Remove the lens
            if lens_index == 0 {
                b.pop_front();
            } else if lens_index == b.len() - 1 {
                b.pop_back();
            } else {
                b.remove(lens_index);
            }
        }
    }

    boxes
        .iter()
        .enumerate()
        .filter(|(_, b)| b.len() > 0)
        .map(|(idx, b)| {
            let box_product = (1 + idx as u128)
                * b.iter()
                    .enumerate()
                    .map(|(idx, l)| (idx as u128 + 1) * l.focal_length as u128)
                    .sum::<u128>();

            box_product
        })
        .sum()
}

#[test]
fn test_sample() {
    let sample = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    let words = parse_text(sample).unwrap();

    assert_eq!(solve(&words), 145);
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

    let words = parse_path(&path)?;
    let solution = solve(&words);

    println!("{solution}");

    Ok(())
}
