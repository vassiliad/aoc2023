use a::logistics::Book;
use anyhow::{Context, Result};
use clap::{arg, command, Parser};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn solve(book: &Book) -> usize {
    book.seeds
        .iter()
        .map(|seed| {
            // println!("Seed: {seed}");
            let last = book.rules.iter().fold(*seed, |src, rule| {
                let n = rule.iter().fold((src, None), |init, mapper| {
                    if init.1.is_none() {
                        (init.0, mapper.src_to_dest(init.0))
                    } else {
                        init
                    }
                });
                // println!(" {n:?}");
                n.1.or(Some(src)).unwrap()
            });

            last
        })
        .min()
        .with_context(|| "No rules")
        .unwrap()
}

#[test]
fn test_sample() -> Result<()> {
    let sample = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

    let book = a::logistics::Book::parse_text(sample)?;
    let solution = solve(&book);

    assert_eq!(solution, 35);

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(args.input);

    let book = a::logistics::Book::parse_path(&path)?;
    let solution = solve(&book);

    println!("{solution}");

    Ok(())
}
