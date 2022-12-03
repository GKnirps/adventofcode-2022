use std::env;
use std::fs::read;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read(Path::new(&filename)).map_err(|e| e.to_string())?;
    let priorities = to_priorities(content)?;
    let rucksacks = parse_rucksacks(&priorities);

    let sum_double_items = sum_doubles(&rucksacks);
    println!(
        "The sum of priorities of items that appear in both compartments is {sum_double_items}."
    );

    Ok(())
}

fn to_priorities(mut content: Vec<u8>) -> Result<Vec<u8>, String> {
    for b in &mut content {
        if *b >= b'a' && *b <= b'z' {
            *b = *b - b'a' + 1;
        } else if *b >= b'A' && *b <= b'Z' {
            *b = *b - b'A' + 27;
        } else if *b == b'\n' {
            *b = 0;
        } else {
            return Err(format!("unexpected byte {b} in input"));
        }
    }
    Ok(content)
}

fn parse_rucksacks(content: &[u8]) -> Vec<(&[u8], &[u8])> {
    content
        .split(|b| *b == 0)
        .filter(|line| !line.is_empty())
        .map(|line| line.split_at(line.len() / 2))
        .collect()
}

fn sum_doubles(rucksacks: &[(&[u8], &[u8])]) -> u32 {
    rucksacks
        .iter()
        .map(|(c1, c2)| find_double(c1, c2) as u32)
        .sum::<u32>()
}

fn find_double(c1: &[u8], c2: &[u8]) -> u8 {
    let mut found: [bool; 53] = [false; 53];
    for prio in c1 {
        found[*prio as usize] = true;
    }
    for prio in c2 {
        if found[*prio as usize] {
            return *prio;
        }
    }
    0
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &[u8] = br#"vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw
"#;

    #[test]
    fn sum_doubles_works_for_example() {
        // given
        let priorities = to_priorities(EXAMPLE.to_vec()).expect("Expected valid input");
        let rucksacks = parse_rucksacks(&priorities);

        // when
        let sum = sum_doubles(&rucksacks);

        // then
        assert_eq!(sum, 157);
    }
}
