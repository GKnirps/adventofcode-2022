use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let instructions = parse_instructions(&content)?;

    let tail_count = count_tail_tiles(&instructions);
    println!("The tail visited {tail_count} tiles at least once.");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Dir {
    U,
    R,
    D,
    L,
}

type Instruction = (Dir, i32);

fn parse_instruction(line: &str) -> Result<Instruction, String> {
    let (ds, is) = line
        .split_once(' ')
        .ok_or_else(|| format!("Invalid instruction '{line}'"))?;
    let dir = match ds {
        "U" => Dir::U,
        "R" => Dir::R,
        "D" => Dir::D,
        "L" => Dir::L,
        _ => return Err(format!("Invalid direction '{ds}' in instruction '{line}'.")),
    };
    let n: i32 = is
        .parse()
        .map_err(|e| format!("Unable to parse number of steps in instruction '{line}': {e}"))?;

    Ok((dir, n))
}

fn parse_instructions(input: &str) -> Result<Vec<Instruction>, String> {
    input.lines().map(parse_instruction).collect()
}

type V2 = (i32, i32);

fn count_tail_tiles(instructions: &[Instruction]) -> usize {
    let mut tail_trail: HashSet<V2> = HashSet::with_capacity(instructions.len());
    let mut head: V2 = (0, 0);
    let mut tail: V2 = (0, 0);
    tail_trail.insert(tail);
    for inst in instructions {
        let (dir, n) = inst;
        let v: V2 = match dir {
            Dir::U => (0, -1),
            Dir::R => (1, 0),
            Dir::D => (0, 1),
            Dir::L => (-1, 0),
        };
        for _ in 0..*n {
            head.0 += v.0;
            head.1 += v.1;
            tail = move_tail(head, tail);
            tail_trail.insert(tail);
        }
    }
    tail_trail.len()
}

fn move_tail(head: V2, tail: V2) -> V2 {
    let dx = (head.0 - tail.0).abs();
    let dy = (head.1 - tail.1).abs();
    if (dy > 0 && dx > 1) || (dy > 1 && dx > 0) {
        (
            tail.0 + (head.0 - tail.0).signum(),
            tail.1 + (head.1 - tail.1).signum(),
        )
    } else if dx > 1 {
        (tail.0 + (head.0 - tail.0).signum(), tail.1)
    } else if dy > 1 {
        (tail.0, tail.1 + (head.1 - tail.1).signum())
    } else {
        tail
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2
"#;

    #[test]
    fn count_tail_tiles_works_for_example() {
        // given
        let instructions = parse_instructions(EXAMPLE).expect("expected successful parsing");

        // when
        let count = count_tail_tiles(&instructions);

        // then
        assert_eq!(count, 13);
    }
}
