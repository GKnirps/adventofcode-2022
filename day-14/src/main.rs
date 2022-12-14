use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let paths = parse_input(&content)?;
    let (initial_cave, height) = init_cave(&paths);

    let settled_sand = drop_until_overflow(initial_cave.clone(), height);
    println!("{settled_sand} units of sand settle before the rest flows in the abyss below.");

    let piled_sand = drop_to_floor_until_block(initial_cave, height);
    println!("{piled_sand} units of sand have piled on the ground.");

    Ok(())
}

type RockPath = Vec<(i32, i32)>;

fn parse_path(line: &str) -> Result<RockPath, String> {
    line.split(" -> ")
        .map(|pair| {
            let (xs, ys) = pair
                .split_once(',')
                .ok_or_else(|| format!("Unable to split pair '{pair}'"))?;
            let x: i32 = xs
                .parse()
                .map_err(|e| format!("Unable to parse '{xs}' as usize: {e}"))?;
            let y: i32 = ys
                .parse()
                .map_err(|e| format!("Unable to parse '{ys}' as usize: {e}"))?;
            Ok((x, y))
        })
        .collect()
}

fn parse_input(input: &str) -> Result<Vec<RockPath>, String> {
    input.lines().map(parse_path).collect()
}

const SAND_ORIGIN: i32 = 500;

fn init_cave(paths: &[RockPath]) -> (HashSet<(i32, i32)>, i32) {
    let height = paths
        .iter()
        .flat_map(|path| path.iter().map(|(_, y)| y))
        .max()
        .copied()
        .unwrap_or(0)
        + 2;

    let mut tiles: HashSet<(i32, i32)> = HashSet::with_capacity((height * height) as usize);

    for path in paths {
        for step in path.windows(2) {
            draw_line(&mut tiles, step[0], step[1]);
        }
    }
    (tiles, height)
}

fn sort(a: i32, b: i32) -> (i32, i32) {
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}

fn draw_line(grid: &mut HashSet<(i32, i32)>, (xa, ya): (i32, i32), (xb, yb): (i32, i32)) {
    if xa == xb {
        let (yfrom, yto) = sort(ya, yb);
        for y in yfrom..=yto {
            grid.insert((xa, y));
        }
    } else if ya == yb {
        let (xfrom, xto) = sort(xa, xb);
        for x in xfrom..=xto {
            grid.insert((x, ya));
        }
    } else {
        eprintln!("line {xa},{ya} -> {xb},{yb} is parallel to any axis, ignoring line");
    }
}

// return true if sand settled inside the grid
fn drop_sand(grid: &mut HashSet<(i32, i32)>, height: i32) -> bool {
    if grid.contains(&(SAND_ORIGIN, 0)) {
        eprintln!("Unable to spawn sand, space occupied.");
        return false;
    }
    let mut x: i32 = SAND_ORIGIN;
    let mut y: i32 = 0;
    let mut moved = true;
    while moved && y < height {
        moved = false;
        if !grid.contains(&(x, y + 1)) {
            moved = true;
            y += 1;
            continue;
        }
        if !grid.contains(&(x - 1, y + 1)) {
            moved = true;
            x -= 1;
            y += 1;
            continue;
        }
        if !grid.contains(&(x + 1, y + 1)) {
            moved = true;
            x += 1;
            y += 1;
        }
    }
    if y < height {
        grid.insert((x, y));
        true
    } else {
        false
    }
}

fn drop_until_overflow(mut grid: HashSet<(i32, i32)>, height: i32) -> usize {
    let intial_blocks = grid.len();
    while drop_sand(&mut grid, height) {}
    grid.len() - intial_blocks
}

// return true if sand could be placed
// panic if sand goes out of bound
fn drop_sand_with_floor(grid: &mut HashSet<(i32, i32)>, height: i32) -> bool {
    if grid.contains(&(SAND_ORIGIN, 0)) {
        return false;
    }
    let mut x: i32 = SAND_ORIGIN;
    let mut y: i32 = 0;
    let mut moved = true;
    while moved && y + 1 < height {
        moved = false;
        if !grid.contains(&(x, y + 1)) {
            moved = true;
            y += 1;
            continue;
        }
        if !grid.contains(&(x - 1, y + 1)) {
            moved = true;
            x -= 1;
            y += 1;
            continue;
        }
        if !grid.contains(&(x + 1, y + 1)) {
            moved = true;
            x += 1;
            y += 1;
        }
    }
    grid.insert((x, y));
    true
}

fn drop_to_floor_until_block(mut grid: HashSet<(i32, i32)>, height: i32) -> usize {
    let initial_blocks = grid.len();
    while drop_sand_with_floor(&mut grid, height) {}
    grid.len() - initial_blocks
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9
"#;

    #[test]
    fn drop_until_overflow_works_for_example() {
        // given
        let paths = parse_input(EXAMPLE).expect("expeced successful parsing");
        let (grid, height) = init_cave(&paths);

        // when
        let count = drop_until_overflow(grid, height);

        // then
        assert_eq!(count, 24);
    }

    #[test]
    fn drop_to_floor_until_block_works_for_example() {
        // given
        let paths = parse_input(EXAMPLE).expect("expeced successful parsing");
        let (grid, height) = init_cave(&paths);

        // when
        let count = drop_to_floor_until_block(grid, height);

        // then
        assert_eq!(count, 93);
    }
}
