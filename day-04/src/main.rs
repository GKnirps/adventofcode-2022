use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let pairs = parse_pairs(&content)?;

    let containing = find_containing_pairs(&pairs);
    println!("There are {containing} pairs where one completely contains the other.");

    let overlapping = find_overlapping_pairs(&pairs);
    println!("There are {overlapping} overlapping pairs.");

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct Assignment(u32, u32);

impl Assignment {
    fn contains(&self, other: &Assignment) -> bool {
        self.0 <= other.0 && self.1 >= other.1
    }
    fn overlaps(&self, other: &Assignment) -> bool {
        self.0 <= other.0 && self.1 >= other.0 || other.0 <= self.0 && other.1 >= self.0
    }
}

fn parse_assignment(input: &str) -> Result<Assignment, String> {
    let (from, to) = input
        .split_once('-')
        .ok_or_else(|| format!("Unable to parse assignment '{input}': Missing '-'"))?;
    Ok(Assignment(
        from.parse::<u32>()
            .map_err(|e| format!("Unable to parse assignment '{input}': {e}"))?,
        to.parse::<u32>()
            .map_err(|e| format!("Unable to parse assignment '{input}': {e}"))?,
    ))
}

fn parse_pair(line: &str) -> Result<(Assignment, Assignment), String> {
    let (first, second) = line
        .split_once(',')
        .ok_or_else(|| format!("Unable to parse line '{line}': missing ','"))?;
    Ok((parse_assignment(first)?, parse_assignment(second)?))
}

fn parse_pairs(content: &str) -> Result<Vec<(Assignment, Assignment)>, String> {
    content.lines().map(parse_pair).collect()
}

fn find_containing_pairs(pairs: &[(Assignment, Assignment)]) -> usize {
    pairs
        .iter()
        .filter(|(a, b)| a.contains(b) || b.contains(a))
        .count()
}

fn find_overlapping_pairs(pairs: &[(Assignment, Assignment)]) -> usize {
    pairs.iter().filter(|(a, b)| a.overlaps(b)).count()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8
"#;

    #[test]
    fn find_containing_pairs_works_for_example() {
        // given
        let pairs = parse_pairs(EXAMPLE).expect("Expected successful parsing");

        // when
        let c = find_containing_pairs(&pairs);

        // then
        assert_eq!(c, 2);
    }

    #[test]
    fn find_overlapping_pairs_works_for_example() {
        // given
        let pairs = parse_pairs(EXAMPLE).expect("Expected successful parsing");

        // when
        let c = find_overlapping_pairs(&pairs);

        // then
        assert_eq!(c, 4);
    }
}
