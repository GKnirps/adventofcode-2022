use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let paths = parse_input(&content)?;
    let initial_cave = init_cave(&paths);

    let settled_sand = drop_until_overflow(initial_cave.clone());
    println!("{settled_sand} units of sand settle before the rest flows in the abyss below.");

    let piled_sand = drop_to_floor_until_block(initial_cave);
    println!("{piled_sand} units of sand have piled on the ground.");

    Ok(())
}

type RockPath = Vec<(usize, usize)>;

fn parse_path(line: &str) -> Result<RockPath, String> {
    line.split(" -> ")
        .map(|pair| {
            let (xs, ys) = pair
                .split_once(',')
                .ok_or_else(|| format!("Unable to split pair '{pair}'"))?;
            let x: usize = xs
                .parse::<usize>()
                .map_err(|e| format!("Unable to parse '{xs}' as usize: {e}"))?;
            let y: usize = ys
                .parse()
                .map_err(|e| format!("Unable to parse '{ys}' as usize: {e}"))?;
            Ok((x, y))
        })
        .collect()
}

fn parse_input(input: &str) -> Result<Vec<RockPath>, String> {
    input.lines().map(parse_path).collect()
}

const SAND_ORIGIN: usize = 500;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Grid {
    offset: usize,
    width: usize,
    height: usize,
    tiles: Vec<bool>,
}

impl Grid {
    fn get(&self, x: usize, y: usize) -> Option<bool> {
        if x >= self.width {
            None
        } else {
            self.tiles
                .get(x + self.offset + (self.width + self.offset) * y)
                .copied()
        }
    }

    fn set(&mut self, x: usize, y: usize) {
        if x >= self.width {
            return;
        }
        if let Some(tile) = self
            .tiles
            .get_mut(x + self.offset + (self.width + self.offset) * y)
        {
            *tile = true;
        }
    }
}

fn init_cave(paths: &[RockPath]) -> Grid {
    let max_x = paths
        .iter()
        .flat_map(|path| path.iter().map(|(x, _)| x))
        .max()
        .copied();

    let height = paths
        .iter()
        .flat_map(|path| path.iter().map(|(_, y)| y))
        .max()
        .copied()
        .unwrap_or(0)
        + 2;

    let width = max_x.unwrap_or(SAND_ORIGIN).max(SAND_ORIGIN) + height;
    let offset = height;

    let tiles: Vec<bool> = vec![false; (width + offset) * height];
    let mut grid = Grid {
        offset,
        width,
        height,
        tiles,
    };

    for path in paths {
        for step in path.windows(2) {
            draw_line(&mut grid, step[0], step[1]);
        }
    }
    grid
}

fn sort(a: usize, b: usize) -> (usize, usize) {
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}

fn draw_line(grid: &mut Grid, (xa, ya): (usize, usize), (xb, yb): (usize, usize)) {
    if xa == xb {
        let (yfrom, yto) = sort(ya, yb);
        for y in yfrom..=yto {
            grid.set(xa, y);
        }
    } else if ya == yb {
        let (xfrom, xto) = sort(xa, xb);
        for x in xfrom..=xto {
            grid.set(x, ya);
        }
    } else {
        eprintln!("line {xa},{ya} -> {xb},{yb} is parallel to any axis, ignoring line");
    }
}

// return true if sand settled inside the grid
fn drop_sand(grid: &mut Grid) -> bool {
    if grid.get(SAND_ORIGIN, 0).unwrap_or(true) {
        eprintln!("Unable to spawn sand, space occupied.");
        return false;
    }
    let mut x: usize = SAND_ORIGIN;
    let mut y: usize = 0;
    let mut moved = true;
    while moved {
        moved = false;
        if let Some(filled) = grid.get(x, y + 1) {
            if !filled {
                moved = true;
                y += 1;
                continue;
            }
        } else {
            return false;
        }
        if x == 0 {
            return false;
        }
        if let Some(filled) = grid.get(x - 1, y + 1) {
            if !filled {
                moved = true;
                x -= 1;
                y += 1;
                continue;
            }
        } else {
            return false;
        }
        if let Some(filled) = grid.get(x + 1, y + 1) {
            if !filled {
                moved = true;
                x += 1;
                y += 1;
            }
        } else {
            return false;
        }
    }
    grid.set(x, y);
    true
}

fn drop_until_overflow(mut grid: Grid) -> u32 {
    let mut count = 0;
    while drop_sand(&mut grid) {
        count += 1;
    }
    count
}

// return true if sand could be placed
// panic if sand goes out of bound
fn drop_sand_with_floor(grid: &mut Grid) -> bool {
    if grid.get(SAND_ORIGIN, 0).unwrap_or(true) {
        return false;
    }
    let mut x: usize = SAND_ORIGIN;
    let mut y: usize = 0;
    let mut moved = true;
    while moved && y + 1 < grid.height {
        moved = false;
        if !grid.get(x, y + 1).unwrap() {
            moved = true;
            y += 1;
            continue;
        }
        if x == 0 {
            panic!("overflow on the left");
        }
        if !grid.get(x - 1, y + 1).unwrap() {
            moved = true;
            x -= 1;
            y += 1;
            continue;
        }
        if x + 1 >= grid.width {
            panic!("overflow on the right");
        }
        if !grid.get(x + 1, y + 1).unwrap() {
            moved = true;
            x += 1;
            y += 1;
        }
    }
    grid.set(x, y);
    true
}

fn drop_to_floor_until_block(mut grid: Grid) -> u32 {
    let mut count = 0;
    while drop_sand_with_floor(&mut grid) {
        count += 1;
    }
    count
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
        let grid = init_cave(&paths);

        // when
        let count = drop_until_overflow(grid);

        // then
        assert_eq!(count, 24);
    }

    #[test]
    fn drop_to_floor_until_block_works_for_example() {
        // given
        let paths = parse_input(EXAMPLE).expect("expeced successful parsing");
        let grid = init_cave(&paths);

        // when
        let count = drop_to_floor_until_block(grid);

        // then
        assert_eq!(count, 93);
    }
}
