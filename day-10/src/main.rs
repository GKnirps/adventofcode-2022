use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let instructions = parse_instructions(&content)?;

    let signal_strength_sum = run_and_inspect(&instructions);
    println!("The signal strength sum is {signal_strength_sum}");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Inst {
    Noop,
    Addx(i64),
}

fn parse_instruction(line: &str) -> Result<Inst, String> {
    if line == "noop" {
        Ok(Inst::Noop)
    } else {
        let (operator, operand) = line
            .split_once(' ')
            .ok_or_else(|| format!("Unable to parse instruction '{line}'"))?;
        if operator == "addx" {
            let opv: i64 = operand
                .parse()
                .map_err(|e| format!("Unable to parse operand '{operand}': {e}"))?;
            Ok(Inst::Addx(opv))
        } else {
            Err(format!("unknown operator '{operator}'"))
        }
    }
}

fn parse_instructions(input: &str) -> Result<Vec<Inst>, String> {
    input.lines().map(parse_instruction).collect()
}

fn run_and_inspect(instructions: &[Inst]) -> i64 {
    let mut x: i64 = 1;
    let mut sig_strength: i64 = 0;
    let mut cycle_count: i64 = 0;

    for inst in instructions {
        if cycle_count > 220 {
            break;
        }
        match inst {
            Inst::Noop => {
                if cycle_count % 40 == 19 {
                    sig_strength += x * (20 + 40 * (cycle_count / 40));
                }
                cycle_count += 1;
            }
            Inst::Addx(v) => {
                if cycle_count % 40 == 18 || cycle_count % 40 == 19 {
                    sig_strength += x * (20 + 40 * (cycle_count / 40));
                }
                x += v;
                cycle_count += 2;
            }
        };
    }
    sig_strength
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn run_and_inspect_works_for_example() {
        // given
        let instructions = parse_instructions(EXAMPLE).expect("expected successful parsing");

        // when
        let sum = run_and_inspect(&instructions);

        // then
        assert_eq!(sum, 13140);
    }

    const EXAMPLE: &str = r#"addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop
"#;
}
