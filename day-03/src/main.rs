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

    if rucksacks.len() % 3 != 0 {
        return Err("it appears we have lost an elf somewhere in the jungle".to_owned());
    }
    let sum_badge_priorities = sum_common(&rucksacks);
    println!("The sum of badge priorities is {sum_badge_priorities}.");

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

fn sum_common(rucksacks: &[(&[u8], &[u8])]) -> u32 {
    // any remainder (group of less than 3) will be ignored. should be checked beforehand
    rucksacks
        .chunks_exact(3)
        .map(|group| find_common(group[0], group[1], group[2]) as u32)
        .sum::<u32>()
}

fn find_common(e1: (&[u8], &[u8]), e2: (&[u8], &[u8]), e3: (&[u8], &[u8])) -> u8 {
    // We have these backpacks in compartments now and I'm too lazy to change that, so we work with
    // the compartments
    let mut found: [u8; 53] = [0; 53];
    for prio in e1.0.iter().chain(e1.1) {
        found[*prio as usize] = 1;
    }
    for prio in e2.0.iter().chain(e2.1) {
        if found[*prio as usize] == 1 {
            found[*prio as usize] = 2;
        }
    }
    for prio in e3.0.iter().chain(e3.1) {
        if found[*prio as usize] == 2 {
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

    #[test]
    fn sum_common_works_for_example() {
        // given
        let priorities = to_priorities(EXAMPLE.to_vec()).expect("Expected valid input");
        let rucksacks = parse_rucksacks(&priorities);

        // when
        let sum = sum_common(&rucksacks);

        // then
        assert_eq!(sum, 70);
    }
}
