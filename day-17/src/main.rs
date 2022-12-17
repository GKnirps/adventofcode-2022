use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let jet_pattern = parse_jet_pattern(&content);

    let stack_height = drop_rocks(2022, &jet_pattern);
    println!("After 2022 rocks, the stack is {stack_height} units high.");

    Ok(())
}

fn parse_jet_pattern(input: &str) -> Vec<bool> {
    input
        .chars()
        .filter_map(|c| match c {
            '<' => Some(false),
            '>' => Some(true),
            _ => None,
        })
        .collect()
}

static SHAPE_BAR: [u8; 1] = [0b11110000];
static SHAPE_PLUS: [u8; 3] = [0b01000000, 0b11100000, 0b01000000];
static SHAPE_CORNER: [u8; 3] = [0b00100000, 0b00100000, 0b11100000];
static SHAPE_COLUMN: [u8; 4] = [0b10000000, 0b10000000, 0b10000000, 0b10000000];
static SHAPE_SQUARE: [u8; 2] = [0b11000000, 0b11000000];

static SHAPES: [(&[u8], u8); 5] = [
    (&SHAPE_BAR, 4),
    (&SHAPE_PLUS, 3),
    (&SHAPE_CORNER, 3),
    (&SHAPE_COLUMN, 1),
    (&SHAPE_SQUARE, 2),
];

const CAVE_WIDTH: u8 = 7;

fn drop_rock(
    mut stack: Vec<u8>,
    mut pattern_index: usize,
    jet_pattern: &[bool],
    shape: &[u8],
    width: u8,
) -> (Vec<u8>, usize) {
    let required_space = 3 + shape.len();
    let available_space = free_top_layers(&stack);
    if required_space > available_space {
        stack.resize(stack.len() + required_space - available_space, 0);
    }
    let mut bottom = stack.len()
        - (shape.len()
            + if available_space > required_space {
                available_space - required_space
            } else {
                0
            });
    let mut left: u8 = 2;

    loop {
        if jet_pattern[pattern_index % jet_pattern.len()]
            && left + width < CAVE_WIDTH
            && !intersect(&stack, shape, bottom, left + 1)
        {
            left += 1;
        } else if !jet_pattern[pattern_index % jet_pattern.len()]
            && left > 0
            && !intersect(&stack, shape, bottom, left - 1)
        {
            left -= 1;
        }
        pattern_index += 1;
        if bottom > 0 && !intersect(&stack, shape, bottom - 1, left) {
            bottom -= 1;
        } else {
            break;
        }
    }
    for (i, row) in shape.iter().rev().enumerate() {
        stack[bottom + i] |= row >> left;
    }
    (stack, pattern_index % jet_pattern.len())
}

fn intersect(stack: &[u8], shape: &[u8], bottom: usize, left: u8) -> bool {
    if stack.len() < shape.len() + bottom {
        return false;
    }

    shape
        .iter()
        .rev()
        .enumerate()
        .any(|(i, row)| (row >> left) & stack[bottom + i] != 0)
}

fn drop_rocks(max_rocks: usize, jet_pattern: &[bool]) -> usize {
    let mut stack: Vec<u8> = Vec::with_capacity(max_rocks * 4);

    let mut pattern_index: usize = 0;

    for i in 0..max_rocks {
        let (shape, width) = SHAPES[i % SHAPES.len()];
        (stack, pattern_index) = drop_rock(stack, pattern_index, jet_pattern, shape, width);
    }

    stack.len() - free_top_layers(&stack)
}

fn free_top_layers(stack: &[u8]) -> usize {
    stack
        .iter()
        .rev()
        .enumerate()
        .filter(|(_, layer)| **layer != 0)
        .map(|(i, _)| i)
        .next()
        .unwrap_or(stack.len())
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

    #[test]
    fn drop_rocks_works_for_example() {
        // given
        let pattern = parse_jet_pattern(EXAMPLE);

        // when
        let height = drop_rocks(2022, &pattern);

        // then
        assert_eq!(height, 3068);
    }

    #[test]
    fn drop_rocks_works_correctly_for_10_rocks_in_example() {
        // given
        let pattern = parse_jet_pattern(EXAMPLE);

        // when
        let height = drop_rocks(10, &pattern);

        // then
        assert_eq!(height, 17);
    }

    #[test]
    fn intersect_works_correctly() {
        // given
        let stack = &[0b11101110, 0, 0, 0, 0];
        let shape = &SHAPE_PLUS;

        // when/then
        assert!(!intersect(stack, shape, 0, 2));
        assert!(!intersect(stack, shape, 1, 3));
        assert!(intersect(stack, shape, 0, 3));
        assert!(intersect(stack, shape, 0, 5));
    }
}
