use anyhow::{bail, Context, Result};
use clap::{arg, command, Parser};
use std::collections::HashMap;
use std::fmt::Formatter;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum Spring {
    Good = 0,
    Bad = 1,
    Unknown = 2,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct SpringRow {
    springs: Vec<Spring>,
    ecc: Vec<u8>,
}

fn parse_text(text: &str, expand: bool) -> Result<Vec<SpringRow>> {
    Ok(text
        .lines()
        .map(|x| x.trim())
        .filter(|x| x.len() > 0)
        .map(|line| {
            let (springs, ecc) = line
                .split_once(' ')
                .with_context(|| "Splitting line into springs and ecc")
                .unwrap();

            let springs = springs.trim();
            let ecc = ecc.trim();

            let (springs, ecc) = if expand {
                let springs = format!("{0}?{0}?{0}?{0}?{0}", springs);
                let ecc = format!("{0},{0},{0},{0},{0}", ecc);
                (springs, ecc)
            } else {
                (springs.to_string(), ecc.to_string())
            };

            let springs: Vec<Spring> = springs
                .chars()
                .map(|c| match c {
                    '.' => Spring::Good,
                    '#' => Spring::Bad,
                    '?' => Spring::Unknown,
                    _ => unreachable!("Unexpected character {c}"),
                })
                .collect();
            let ecc: Vec<u8> = ecc
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

    parse_text(&contents, true)
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

/// The memoization cache contains metadata generated during processing ONE SpringRow.
/// The key contains:
///
///     num_bad: How many Bad springs are in the group of Springs we're currently handling
///     start: The index of the Spring we're looking at
///     this_spring: The type of the current spring (this is always Good or Bad even if springs[start] = Unknown)
///     last_good: Whether the spring right before this one was a Good one
///     idx_group: The index of the group of Bad springs we're currently handling
fn memo_kernel(
    springs: &SpringRow,
    memo: &mut HashMap<(usize, Spring, usize, bool, usize), u128>,
    start: usize,
    this_spring: Spring,
    num_bad: usize,
    last_good: bool,
    idx_group: usize,
) -> u128 {
    let key = (start, this_spring, num_bad, last_good, idx_group);
    if let Some(ret) = memo.get(&key) {
        return *ret;
    }

    let ret = count_valid_permutations(
        springs,
        memo,
        start,
        this_spring,
        num_bad,
        last_good,
        idx_group,
    );

    memo.insert(key, ret);
    ret
}

fn count_valid_permutations(
    springs: &SpringRow,
    memo: &mut HashMap<(usize, Spring, usize, bool, usize), u128>,
    start: usize,
    this_spring: Spring,
    num_bad: usize,
    last_good: bool,
    idx_group: usize,
) -> u128 {
    let key = (start, this_spring, num_bad, last_good, idx_group);
    if let Some(ret) = memo.get(&key) {
        return *ret;
    }

    let mut idx_group = idx_group;
    let mut last_good = last_good;
    let mut num_bad = num_bad;

    /// Function handles 1 bad character and returns False if the SpringRow is invalid
    fn process_good_and_decide_if_invalid(
        springs: &SpringRow,
        num_bad: &mut usize,
        idx_group: &mut usize,
        last_good: &mut bool,
        key: &(usize, Spring, usize, bool, usize),
        memo: &mut HashMap<(usize, Spring, usize, bool, usize), u128>,
    ) -> bool {
        if *num_bad > 0 {
            if *idx_group >= springs.ecc.len() {
                // VV: Not enough ECC groups
                memo.insert(*key, 0);
                return false;
            } else {
                if *num_bad > springs.ecc[*idx_group] as usize {
                    // VV: Current ECC group is not large enough for observed number of Bad sprints
                    return false;
                }
            }
        }

        if !*last_good {
            if (*num_bad > 0)
                && ((*idx_group >= springs.ecc.len())
                    || (*num_bad != springs.ecc[*idx_group] as usize))
            {
                // VV: Current ECC group is not large enough for observed number of Bad sprints
                return false;
            }
            *idx_group += 1;
            *num_bad = 0;
        }

        *last_good = true;

        return true;
    }
    let mut first = true;

    for start in start..springs.springs.len() {
        let s = if first {
            this_spring
        } else {
            springs.springs[start]
        };
        first = false;

        match s {
            Spring::Good => {
                if !process_good_and_decide_if_invalid(
                    springs,
                    &mut num_bad,
                    &mut idx_group,
                    &mut last_good,
                    &key,
                    memo,
                ) {
                    return 0;
                }
            }
            Spring::Bad => {
                if idx_group >= springs.ecc.len() {
                    // VV: This would mean that there's at least 1 more ECC group
                    return 0;
                } else {
                    num_bad += 1;

                    if num_bad > springs.ecc[idx_group] as usize {
                        // VV: The current ECC group is not large enough for for observed Bad springs
                        return 0;
                    }

                    let next_spring = springs
                        .springs
                        .get(start + 1)
                        .or(Some(&Spring::Good))
                        .unwrap();

                    let this = memo_kernel(
                        &springs,
                        memo,
                        start + 1,
                        *next_spring,
                        num_bad,
                        false,
                        idx_group,
                    );
                    return this;
                }
            }

            Spring::Unknown => {
                let val_bad = memo_kernel(
                    &springs,
                    memo,
                    start,
                    Spring::Bad,
                    num_bad,
                    last_good,
                    idx_group,
                );

                // VV: First, try handling all Good springs - marking the current Unknown one as a Good spring
                let mut start = start;
                while start < springs.springs.len() {
                    if !process_good_and_decide_if_invalid(
                        springs,
                        &mut num_bad,
                        &mut idx_group,
                        &mut last_good,
                        &key,
                        memo,
                    ) {
                        return val_bad;
                    }

                    // VV: If code gets here then it either reached a Bad spring, an Unknown one, or the end of the Row
                    start = start + 1;

                    // VV: If we're at the end of the Row then, exit out of the loop and let the tail logic
                    // handle this case
                    if start >= springs.springs.len()
                        || !matches!(springs.springs[start], Spring::Good)
                    {
                        break;
                    }
                }

                // VV: Process the next character, if we're past the end of the Row then assume there's a Good spring
                // there
                let this_spring = springs.springs.get(start).or(Some(&Spring::Good)).unwrap();
                let val_good = memo_kernel(
                    &springs,
                    memo,
                    start,
                    *this_spring,
                    num_bad,
                    last_good,
                    idx_group,
                );
                return val_good + val_bad;
            }
        }
    }

    // VV: Code gets here after it has processed the entire Row
    if last_good {
        // VV: If the last Spring was good then there must **not** be any expected Bad springs here
        (idx_group >= springs.ecc.len() && num_bad == 0) as u128
    } else {
        // VV: Here, the last Spring was bad. It **must** have consumed the final group
        !(idx_group + 1 != springs.ecc.len()
            || num_bad != springs.ecc[springs.ecc.len() - 1] as usize) as u128
    }
}

fn solve(springs: &mut Vec<SpringRow>) -> u128 {
    springs
        .iter_mut()
        .map(|s| {
            let mut memo = HashMap::new();
            let this_spring = s.springs.get(0).or(Some(&Spring::Good)).unwrap();
            count_valid_permutations(s, &mut memo, 0, *this_spring, 0, true, 0)
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

    let test_full = true;

    let mut springs = parse_text(sample, test_full)?;

    let solution = solve(&mut springs);

    if test_full {
        assert_eq!(solution, 525152);
    } else {
        assert_eq!(solution, 21);
    }

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
