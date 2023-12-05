use anyhow::Result;
use b::logistics::Book;
use clap::{arg, command, Parser};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(long, short, default_value = "input/mine")]
    input: std::path::PathBuf,
}

fn solve(book: &mut Book) -> usize {
    book.introduce_seed_layer();

    // VV: [start_index, stop_index_including_last_index]

    let mut source: Vec<(usize, usize)> = book
        .rules
        .get(0)
        .unwrap()
        .iter()
        .map(|x| (x.src, x.src + x.len - 1))
        .collect();
    let mut dest: Vec<(usize, usize)> = vec![];

    // VV: For each Mapper calculate its overlap with mappers in the current layer.
    // From those produce the Mappers-of-interest which correspond to the layer one level deeper.
    // Then swap source/dest and continue till the final layer.
    // The answer is the Mapper in `source` with the smallest `src` value.

    for layer in book.rules.iter().skip(1) {
        for &(start, end) in source.iter() {
            let mut start = start;

            while start < end {
                let mut next_src = start;
                let mut next_src_end = end;

                let mut matched = false;
                let mut next_start = end;
                let mut next_min = end;

                for candidate in layer.iter() {
                    if let Some(value) = candidate.src_to_dest(start) {
                        let candidate_end = candidate.src + candidate.len;

                        next_start = end.min(candidate_end);
                        next_src = value;
                        next_src_end = value + (next_start - start);

                        matched = true;
                        break;
                    }

                    if start <= candidate.src && candidate.src <= end {
                        next_min = next_min.min(candidate.src);
                    }
                }

                // VV: If none of the Mappers were relevant for the `start` index, then try the
                // closest larger Mapper or just skip to the end of the current range if there's
                // no Mapper in the next layer with a .src that's between [start, end]
                if !matched {
                    next_src = start;
                    if next_min > start {
                        next_src_end = next_min;
                    } else {
                        next_src_end = end;
                    }

                    next_start = next_src_end;
                }

                dest.push((next_src, next_src_end));

                start = next_start;
            }
        }

        std::mem::swap(&mut dest, &mut source);
        dest.clear();
    }

    source.iter().map(|m| m.0).min().unwrap()
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

    let mut book = Book::parse_text(sample)?;
    let solution = solve(&mut book);

    assert_eq!(solution, 46);

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = std::env::current_dir().unwrap().join(args.input);

    let mut book = Book::parse_path(&path)?;
    let solution = solve(&mut book);

    println!("{solution}");

    Ok(())
}
