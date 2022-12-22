use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (map, path) = parse_input(&content)?;

    let password = walk_path(&map, &path);
    println!("The password is {password}");

    Ok(())
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct MapBlock {
    offset: usize,
    width: usize,
    grid: Vec<bool>,
}

impl MapBlock {
    fn height(&self) -> usize {
        self.grid.len() / self.width
    }

    // will panic if y is out of bounds
    fn get_xwrap(&self, x: usize, y: usize) -> bool {
        self.grid[y * self.width + (x % self.width)]
    }

    fn global_x(&self, x: usize) -> usize {
        self.offset + x
    }

    fn local_x(&self, global_x: usize) -> Option<usize> {
        if global_x >= self.offset && global_x < self.offset + self.width {
            Some(global_x - self.offset)
        } else {
            None
        }
    }
}

type Map = Vec<MapBlock>;

fn parse_map(input: &str) -> Result<Map, String> {
    let mut map: Map = Vec::with_capacity(16);
    let mut lines = input.lines();

    let line = lines.next().ok_or_else(|| "map is empty".to_owned())?;
    let mut offset = line.bytes().take_while(|c| *c == b' ').count();
    let mut width = line.as_bytes().len() - offset;
    if width == 0 {
        return Err("map line has zero width".to_owned());
    }
    let mut grid: Vec<bool> = Vec::with_capacity(input.len());
    // ok, we're making the assumption that the input is well-formed and only contains valid
    // characters
    grid.extend(line.as_bytes()[offset..].iter().map(|c| *c != b'#'));

    for line in lines {
        let new_offset = line.bytes().take_while(|c| *c == b' ').count();
        let new_width = line.as_bytes().len() - new_offset;
        if offset != new_offset || width != new_width {
            map.push(MapBlock {
                offset,
                width,
                grid,
            });
            offset = new_offset;
            width = new_width;
            grid = Vec::with_capacity(input.len());
        }
        if width == 0 {
            return Err("map line has zero width".to_owned());
        }
        // ok, we're making the assumption that the input is well-formed and only contains valid
        // characters
        grid.extend(line.as_bytes()[offset..].iter().map(|c| *c != b'#'));
    }
    map.push(MapBlock {
        offset,
        width,
        grid,
    });

    Ok(map)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Turn {
    R,
    L,
    Straight(u32),
}

fn parse_path(input: &str) -> Result<Vec<Turn>, String> {
    let mut path: Vec<Turn> = Vec::with_capacity(input.len());
    let mut number = 0;
    for c in input.trim().chars() {
        if c == 'R' {
            if number != 0 {
                path.push(Turn::Straight(number));
                number = 0;
            }
            path.push(Turn::R);
        } else if c == 'L' {
            if number != 0 {
                path.push(Turn::Straight(number));
                number = 0;
            }
            path.push(Turn::L);
        } else if let Some(digit) = c.to_digit(10) {
            number = number * 10 + digit;
        } else {
            return Err(format!("Unexpected character '{c}' in path definition."));
        }
    }
    if number != 0 {
        path.push(Turn::Straight(number));
    }
    Ok(path)
}

fn parse_input(input: &str) -> Result<(Map, Vec<Turn>), String> {
    let (raw_map, raw_path) = input
        .split_once("\n\n")
        .ok_or_else(|| "unable to split path from map".to_owned())?;
    let map = parse_map(raw_map)?;
    let path = parse_path(raw_path)?;

    Ok((map, path))
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Dir {
    Up,
    Right,
    Down,
    Left,
}

fn walk_path(map: &Map, path: &[Turn]) -> usize {
    if map.is_empty() {
        return 1004;
    }
    let mut dir = Dir::Right;
    let mut block: usize = 0;
    let mut x: usize = 0;
    let mut y: usize = 0;

    for turn in path {
        match turn {
            Turn::L => {
                dir = match dir {
                    Dir::Up => Dir::Left,
                    Dir::Right => Dir::Up,
                    Dir::Down => Dir::Right,
                    Dir::Left => Dir::Down,
                };
            }
            Turn::R => {
                dir = match dir {
                    Dir::Up => Dir::Right,
                    Dir::Right => Dir::Down,
                    Dir::Down => Dir::Left,
                    Dir::Left => Dir::Up,
                };
            }
            Turn::Straight(n) => {
                for _ in 0..*n {
                    match dir {
                        Dir::Up => {
                            if y > 0 {
                                if map[block].get_xwrap(x, y - 1) {
                                    y -= 1;
                                }
                            } else {
                                let global_x = map[block].global_x(x);
                                for di in 1..=map.len() {
                                    let i = (block as isize - di as isize)
                                        .rem_euclid(map.len() as isize)
                                        as usize;
                                    if let Some(local_x) = map[i].local_x(global_x) {
                                        if map[i].get_xwrap(local_x, map[i].height() - 1) {
                                            block = i;
                                            x = local_x;
                                            y = map[block].height() - 1;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Dir::Right => {
                            if map[block].get_xwrap(x + 1, y) {
                                x = (x + 1) % map[block].width;
                            }
                        }
                        Dir::Down => {
                            if y + 1 < map[block].height() {
                                if map[block].get_xwrap(x, y + 1) {
                                    y += 1;
                                }
                            } else {
                                let global_x = map[block].global_x(x);
                                for di in 1..=map.len() {
                                    if let Some(local_x) =
                                        map[(block + di) % map.len()].local_x(global_x)
                                    {
                                        if map[(block + di) % map.len()].get_xwrap(local_x, 0) {
                                            block = (block + di) % map.len();
                                            x = local_x;
                                            y = 0;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Dir::Left => {
                            let next_x = if x == 0 { map[block].width - 1 } else { x - 1 };
                            if map[block].get_xwrap(next_x, y) {
                                x = next_x;
                            }
                        }
                    }
                }
            }
        }
    }

    1000 * (y + 1 + map[0..block].iter().map(|b| b.height()).sum::<usize>())
        + 4 * (map[block].global_x(x) + 1)
        + match dir {
            Dir::Right => 0,
            Dir::Down => 1,
            Dir::Left => 2,
            Dir::Up => 3,
        }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn walk_path_works_for_example() {
        // given
        let (map, path) = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let key = walk_path(&map, &path);

        // then
        assert_eq!(key, 6032);
    }

    const EXAMPLE: &str = r#"        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5
"#;
}
