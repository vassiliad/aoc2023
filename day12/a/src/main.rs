use anyhow::{Context, Result};
use clap::{arg, command, Parser};
use std::fmt::Formatter;

#[derive(Copy, Clone, Debug)]
enum Spring {
    Good = 0,
    Bad = 1,
    Unknown = 2,
}

#[derive(Clone, Debug)]
struct SpringRow {
    springs: Vec<Spring>,
    ecc: Vec<u8>,
}

fn parse_text(text: &str) -> Result<Vec<SpringRow>> {
    Ok(text
        .lines()
        .map(|x| x.trim())
        .filter(|x| x.len() > 0)
        .map(|line| {
            let (springs, ecc) = line
                .split_once(' ')
                .with_context(|| "Splitting line into springs and ecc")
                .unwrap();

            let springs = springs
                .chars()
                .map(|c| match c {
                    '.' => Spring::Good,
                    '#' => Spring::Bad,
                    '?' => Spring::Unknown,
                    _ => unreachable!("Unexpected character {c}"),
                })
                .collect();
            let ecc = ecc
                .split(',')
                .map(|word| word.trim())
                .map(|word| {
                    word.parse::<u8>()
                        .with_context(|| "Parsing ecc mask")
                        .unwrap()
                })
                .collect();

            SpringRow { springs, ecc }
        })
        .collect())
}

fn parse_path(path: &std::path::Path) -> Result<Vec<SpringRow>> {
    let contents = std::fs::read_to_string(path).with_context(|| "Reading file")?;

    parse_text(&contents)
}

impl std::fmt::Display for SpringRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for s in self.springs.iter() {
            match s {
                Spring::Good => write!(f, ".").unwrap(),
                Spring::Bad => write!(f, "#").unwrap(),
                Spring::Unknown => write!(f, "?").unwrap(),
            }
        }

        write!(
            f,
            " {}",
            self.ecc
                .iter()
                .fold(String::new(), |text, num| { format!("{},{}", text, num) })
                .strip_prefix(',')
                .or(Some(","))
                .unwrap()
        )
    }
}

impl SpringRow {
    fn is_valid(&self) -> bool {
        // VV: First do some coarse grain tests
        let first_unknown = self
            .springs
            .iter()
            .position(|x| matches!(x, Spring::Unknown));

        if first_unknown.is_none() {
            let num_good = self
                .springs
                .iter()
                .filter(|x| matches!(x, Spring::Good))
                .count();

            if num_good != self.ecc.len() - 1 {
                return false;
            }
        }

        let idx_unknown = first_unknown.or(Some(self.springs.len())).unwrap();

        let mut skip = 0;
        for idx in 0..self.ecc.len() {
            let next_good = self
                .springs
                .iter()
                .skip(skip)
                .position(|x| matches!(x, Spring::Good));

            let number_bad = if let Some(idx_good) = next_good {
                idx_good
            } else {
                if skip < self.springs.len() {
                    self.springs.len() - skip
                } else {
                    // VV: We're already scanned to the end of this SpringRow, it's invalid
                    return false;
                }
            };

            if number_bad != self.ecc[idx] as usize {
                // VV: This looks like an invalid SpringRow however, give it the benefit of
                // the doubt if there's at least 1 unknown Spring in this group of Bad springs
                // or those that came before
                return idx_unknown < skip + number_bad;
            }

            skip += number_bad + 1;
        }

        return true;
    }

    /// Removes Good springs from beginning and end of SpringRow.
    /// Also replaces multiple consecutive Good springs with 1
    fn trim_good(&mut self) -> bool {
        let mut trimmed = false;

        let mut i = 1;

        while i < self.springs.len() {
            if matches!(self.springs[i], Spring::Good)
                && matches!(self.springs[i - 1], Spring::Good)
            {
                self.springs.remove(i);
                trimmed = true;
            } else {
                i += 1;
            }
        }

        if self.springs.len() > 0 && matches!(self.springs[0], Spring::Good) {
            self.springs.remove(0);
            trimmed = true;
        }
        if self.springs.len() > 0 && matches!(self.springs.last().unwrap(), Spring::Good) {
            self.springs.pop();
            trimmed = true;
        }

        trimmed
    }
}

fn count_valid_permutations(springs: &SpringRow) -> u128 {
    let mut valid = 0;

    let mut pending = vec![springs.clone()];

    while pending.len() > 0 {
        let current = pending.pop().unwrap();

        if current.is_valid() {
            if let Some(first_unknown) = current
                .springs
                .iter()
                .position(|x| matches!(x, Spring::Unknown))
            {
                let mut good = current.clone();
                good.springs[first_unknown] = Spring::Good;
                good.trim_good();

                let mut bad = current.clone();
                bad.springs[first_unknown] = Spring::Bad;
                bad.trim_good();

                pending.push(good);
                pending.push(bad);
            } else {
                valid += 1;
            }
        }
    }

    valid
}

fn solve(springs: &mut Vec<SpringRow>) -> u128 {
    springs
        .iter_mut()
        .map(|s| {
            let repres = format!("{s}");

            s.trim_good();
            let x = count_valid_permutations(s);

            println!("{repres} -> {x}");
            x
        })
        .sum()
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1";

    let mut springs = parse_text(sample)?;

    let solution = solve(&mut springs);

    assert_eq!(solution, 21);

    Ok(())
}

#[test]
fn test_one() -> Result<()> {
    let sample = "??????.??..? 2,1,2";
    let mut springs = parse_text(sample)?;

    let solution = solve(&mut springs);

    assert_eq!(solution, 6);

    Ok(())
}

#[test]
fn test_two() -> Result<()> {
    let sample = "???.###????.###????.###????.###????.### 1,1,3,1,1,3,1,1,3,1,1,3,1,1,3";
    let mut springs = parse_text(sample)?;

    let solution = solve(&mut springs);

    assert_eq!(solution, 6);

    Ok(())
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

    let mut springs = parse_path(&path)?;

    let solution = solve(&mut springs);

    println!("{solution}");

    Ok(())
}
