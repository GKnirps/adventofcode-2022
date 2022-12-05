use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let (initial_stacks, instructions) = parse_input(&content)?;

    let done_stacks = run_instructions(initial_stacks, &instructions)?;
    println!(
        "The top of the stacks are '{}'",
        get_stack_tops(&done_stacks)
    );

    Ok(())
}

type Stack = Vec<char>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Instruction {
    count: u32,
    from: usize,
    to: usize,
}

fn parse_input(input: &str) -> Result<(Vec<Stack>, Vec<Instruction>), String> {
    let (stack_input, inst_input) = input
        .split_once("\n\n")
        .ok_or_else(|| "Unable to split input between stacks and instructions".to_owned())?;
    let mut stacks: Vec<Stack> = Vec::with_capacity(16);
    for line in stack_input.lines() {
        // stop as soon as the stack numbers show up
        // (of course, if the input is malformed, this may just discard everything after that)
        if line.starts_with(" 1 ") {
            break;
        }
        // for the stack representation, it is actually easier to work on a byte slice
        for (i, chunk) in line
            .as_bytes()
            .chunks(4)
            .map(|c| if c.len() == 4 { &c[..3] } else { c })
            .enumerate()
        {
            if stacks.len() < i + 1 {
                stacks.push(Vec::with_capacity(26));
            }
            if chunk != b"   " {
                let item = chunk
                    .strip_prefix(b"[")
                    .ok_or_else(|| format!("item {i} in line '{line} is missing open bracket"))?
                    .strip_suffix(b"]")
                    .ok_or_else(|| format!("item {i} in line '{line} is missing open bracket"))?;
                let item = *item
                    .first()
                    .ok_or_else(|| format!("item {i} in line {line} is missing an identifier"))?;
                stacks[i].push(char::from(item));
            }
        }
    }
    for stack in &mut stacks {
        stack.reverse();
    }

    let instructions = inst_input
        .lines()
        .map(parse_instruction)
        .collect::<Result<Vec<Instruction>, String>>()?;

    Ok((stacks, instructions))
}

fn parse_instruction(line: &str) -> Result<Instruction, String> {
    let (c, loc) = line
        .split_once(" from ")
        .ok_or_else(|| format!("unable to parse instruction '{line}': missing ' from '"))?;
    let count: u32 = c
        .strip_prefix("move ")
        .ok_or_else(|| format!("unable to parse instruction '{line}': mossing 'move '"))?
        .parse()
        .map_err(|e| format!("unable to parse number in instruction '{line}': {e}"))?;

    let (from_str, to_str) = loc
        .split_once(" to ")
        .ok_or_else(|| format!("unable to parse instruction '{line}': Missing ' to '"))?;
    let from: usize = from_str
        .parse()
        .map_err(|e| format!("unable to parse source in line '{line}': {e}"))?;
    let to: usize = to_str
        .parse()
        .map_err(|e| format!("unable to parse source in line '{line}': {e}"))?;

    Ok(Instruction { count, from, to })
}

fn run_instructions(
    mut stacks: Vec<Stack>,
    instructions: &[Instruction],
) -> Result<Vec<Stack>, String> {
    for inst in instructions {
        if inst.from == 0 || inst.from > stacks.len() {
            return Err(format!(
                "Faulty instruction, referencing out-of-bounds from-stack {}/{}",
                inst.from,
                stacks.len()
            ));
        }
        if inst.to == 0 || inst.to > stacks.len() {
            return Err(format!(
                "Faulty instruction, referencing out-of-bounds to-stack {}/{}",
                inst.from,
                stacks.len()
            ));
        }
        let from = inst.from - 1;
        let to = inst.to - 1;
        for _ in 0..inst.count {
            let item = stacks[from]
                .pop()
                .ok_or_else(|| format!("Trying to take something from empty stack {from}"))?;
            stacks[to].push(item);
        }
    }
    Ok(stacks)
}

fn get_stack_tops(stacks: &[Stack]) -> String {
    stacks
        .iter()
        .map(|s| s.last().copied().unwrap_or(' '))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2
"#;

    #[test]
    fn run_instructions_works_for_example() {
        // given
        let (stacks, instructions) = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let result = run_instructions(stacks, &instructions);

        // then
        let result_stacks = result.expect("expected successful run");
        assert_eq!(&get_stack_tops(&result_stacks), "CMZ");
    }
}
