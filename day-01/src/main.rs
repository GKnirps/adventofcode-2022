use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let elf_backpacks = parse_input(&content)?;
    let mut elf_calories: Vec<u32> = elf_backpacks
        .iter()
        .map(|elf| elf.iter().sum::<u32>())
        .collect();

    // this may not be the fastest solution for part 2, but it sure is the easiest
    elf_calories.sort_unstable();

    if let Some(cal) = elf_calories.last() {
        println!("The elf carrying the most carries {cal} calories");
    } else {
        println!("Apparently, no one joined the expedition");
    }

    let cal3: u32 = elf_calories.iter().rev().take(3).sum();
    println!("The three most heavily loaded elfs carry {cal3} cal in total.");

    Ok(())
}

fn parse_input(input: &str) -> Result<Vec<Vec<u32>>, String> {
    input
        .split("\n\n")
        .map(|elf| {
            elf.lines()
                .map(|line| {
                    line.parse::<u32>()
                        .map_err(|e| format!("unable to parse line '{line}': {e}"))
                })
                .collect::<Result<Vec<u32>, String>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXAMPLE: &str = r#"1000
2000
3000

4000

5000
6000

7000
8000
9000

10000"#;

    #[test]
    fn parse_input_parses_example_input() {
        // when
        let result = parse_input(EXAMPLE);

        // then
        let elfs = result.expect("expected successful parsing");
        assert_eq!(
            &elfs,
            &[
                vec![1000, 2000, 3000],
                vec![4000],
                vec![5000, 6000],
                vec![7000, 8000, 9000],
                vec![10000]
            ]
        );
    }
}
