use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let elves = parse_input(&content);

    let elves_after_10 = run_rounds(elves, 10);
    if let Some(a) = empty_squares(&elves_after_10) {
        println!("After 10 rounds, there are {a} empty squares in the bounding rectangle around the elves");
    } else {
        println!("Elves? I didn't see any elves.");
    }

    Ok(())
}

type P = (i64, i64);

fn parse_input(input: &str) -> HashSet<P> {
    input
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars().enumerate().filter_map(move |(x, c)| {
                if c == '#' {
                    Some((x as i64, y as i64))
                } else {
                    None
                }
            })
        })
        .collect()
}

fn do_round(elves: &mut HashSet<P>, round_index: usize) {
    let mut proposals: HashMap<P, Option<P>> = HashMap::with_capacity(elves.len());
    for (x, y) in elves.iter().copied() {
        let nw = elves.contains(&(x - 1, y - 1));
        let n = elves.contains(&(x, y - 1));
        let ne = elves.contains(&(x + 1, y - 1));
        let w = elves.contains(&(x - 1, y));
        let e = elves.contains(&(x + 1, y));
        let sw = elves.contains(&(x - 1, y + 1));
        let s = elves.contains(&(x, y + 1));
        let se = elves.contains(&(x + 1, y + 1));

        if nw || n || ne || w || e || sw || s || se {
            let mut elf_proposals: [Option<P>; 4] = [None; 4];
            if !nw && !n && !ne {
                elf_proposals[0] = Some((x, y - 1));
            }
            if !sw && !s && !se {
                elf_proposals[1] = Some((x, y + 1));
            }
            if !nw && !w && !sw {
                elf_proposals[2] = Some((x - 1, y));
            }
            if !ne && !e && !se {
                elf_proposals[3] = Some((x + 1, y));
            }
            for i in 0..4 {
                if let Some(to) = elf_proposals[(round_index + i) % 4] {
                    proposals
                        .entry(to)
                        .and_modify(|e| *e = None)
                        .or_insert(Some((x, y)));
                    break;
                }
            }
        }
    }

    for (to, maybe_from) in proposals.iter() {
        if let Some(from) = maybe_from {
            elves.remove(from);
            elves.insert(*to);
        }
    }
}

fn run_rounds(mut elves: HashSet<P>, rounds: usize) -> HashSet<P> {
    for i in 0..rounds {
        do_round(&mut elves, i);
    }
    elves
}

fn empty_squares(elves: &HashSet<P>) -> Option<i64> {
    let lower_x = elves.iter().map(|(x, _)| x).min()?;
    let upper_x = elves.iter().map(|(x, _)| x).max()?;

    let lower_y = elves.iter().map(|(_, y)| y).min()?;
    let upper_y = elves.iter().map(|(_, y)| y).max()?;

    Some((upper_x - lower_x + 1) * (upper_y - lower_y + 1) - elves.len() as i64)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"..............
..............
.......#......
.....###.#....
...#...#.#....
....#...##....
...#.###......
...##.#.##....
....#..#......
..............
..............
..............
"#;
    const EXAMPLE_AFTER_10: &str = r#".......#......
...........#..
..#.#..#......
......#.......
...#.....#..#.
.#......##....
.....##.......
..#........#..
....#.#..#....
..............
....#..#..#...
..............
"#;

    #[test]
    fn run_rounds_works_for_example() {
        // given
        let elves = parse_input(EXAMPLE);
        let expected_elves = parse_input(EXAMPLE_AFTER_10);

        // when
        let elves = run_rounds(elves, 10);

        // then
        assert_eq!(elves, expected_elves);
    }

    #[test]
    fn do_round_works_for_small_example() {
        // given
        let mut elves = parse_input(
            r#".....
..##.
..#..
.....
..##.
.....
"#,
        );
        let expected_elves = parse_input(
            r#"..##.
.....
..#..
...#.
..#..
.....
"#,
        );

        // when
        do_round(&mut elves, 0);

        // then
        assert_eq!(elves, expected_elves);
    }

    #[test]
    fn empty_squares_works_for_example() {
        // given
        let elves = parse_input(EXAMPLE_AFTER_10);

        // when
        let result = empty_squares(&elves);

        // then
        assert_eq!(elves.len(), 22);
        assert_eq!(result, Some(110));
    }
}
