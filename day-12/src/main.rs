use std::collections::{HashSet, VecDeque};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (startpos, endpos, grid) = parse_input(&content)?;

    if let Some(len) = shortest_path_length(&grid, startpos, endpos) {
        println!("The shortest path to the point with best signal is {len}");
    } else {
        println!("There is no way to get up there. Good luck.");
    }

    if let Some(len) = shortest_hiking_trail(&grid, endpos) {
        println!("The shortest hiking trail is {len}");
    }

    Ok(())
}

type P = (usize, usize);

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
struct Grid {
    width: usize,
    heights: Vec<u8>,
}

impl Grid {
    fn get(&self, (px, py): P) -> Option<u8> {
        if px < self.width && py < self.heights.len() / self.width {
            self.heights.get(px + py * self.width).copied()
        } else {
            None
        }
    }
    fn neighbours(&self, (px, py): P) -> Vec<P> {
        let pos_height = match self.get((px, py)) {
            Some(h) => h,
            _ => return vec![],
        };
        // I could avoid this allocation, but this is more convenient so here we are
        let mut n: Vec<P> = Vec::with_capacity(8);
        for (dx, dy) in [(0, 1), (2, 1), (1, 0), (1, 2)] {
            if let Some(ny) = (py + dy).checked_sub(1) {
                if let Some(nx) = (px + dx).checked_sub(1) {
                    if let Some(h) = self.get((nx, ny)) {
                        if h <= pos_height + 1 {
                            n.push((nx, ny));
                        }
                    }
                }
            }
        }
        n
    }
}

fn parse_grid(input: &str) -> Result<Grid, String> {
    // assume Ascii-only input. For non-Ascii, width/height may be messed up
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "input is empty".to_owned())?
        .len();
    let heights: Vec<u8> = input
        .bytes()
        .map(|c| match c {
            b'S' => b'a',
            b'E' => b'z',
            _ => c,
        })
        .filter(|c| c.is_ascii_lowercase())
        .map(|c| c - b'a')
        .collect();
    if heights.len() % width != 0 {
        Err(format!(
            "Width of grid is {width}, but total number of tiles {0} is not divisible by it",
            heights.len()
        ))
    } else {
        Ok(Grid { width, heights })
    }
}

fn parse_input(input: &str) -> Result<(P, P, Grid), String> {
    let grid = parse_grid(input)?;
    let offset_s: usize = input
        .bytes()
        .filter(|c| c.is_ascii_alphabetic())
        .enumerate()
        .filter(|(_, c)| *c == b'S')
        .map(|(i, _)| i)
        .next()
        .ok_or_else(|| "Unable to find start marker 'S' in input".to_owned())?;
    let offset_e: usize = input
        .bytes()
        .filter(|c| c.is_ascii_alphabetic())
        .enumerate()
        .filter(|(_, c)| *c == b'E')
        .map(|(i, _)| i)
        .next()
        .ok_or_else(|| "Unable to find end marker 'E' in input".to_owned())?;

    Ok((
        (offset_s % grid.width, offset_s / grid.width),
        (offset_e % grid.width, offset_e / grid.width),
        grid,
    ))
}

fn shortest_path_length(grid: &Grid, start: P, end: P) -> Option<u32> {
    let mut queue: VecDeque<(P, u32)> = VecDeque::with_capacity(grid.heights.len());
    queue.push_back((start, 0));
    let mut visited: HashSet<P> = HashSet::with_capacity(grid.heights.len());
    while let Some((current, distance)) = queue.pop_front() {
        if current == end {
            return Some(distance);
        }
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);
        for n in grid.neighbours(current) {
            queue.push_back((n, distance + 1));
        }
    }
    None
}

fn shortest_hiking_trail(grid: &Grid, end: P) -> Option<u32> {
    // Let's brute force this with the previous shortest path alg, should be fine.
    // I can think of at least two ways to make this more efficient, but doing it this way is more
    // efficient on my development time.
    grid.heights
        .iter()
        .enumerate()
        .filter(|(_, h)| **h == 0)
        .filter_map(|(i, _)| shortest_path_length(grid, (i % grid.width, i / grid.width), end))
        .min()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi
"#;

    #[test]
    fn shortest_path_length_works_for_example() {
        // given
        let (start, end, grid) = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let len = shortest_path_length(&grid, start, end);

        // then
        assert_eq!(len, Some(31));
    }

    #[test]
    fn shortest_hiking_trail_works_for_example() {
        // given
        let (_, end, grid) = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let len = shortest_hiking_trail(&grid, end);

        // then
        assert_eq!(len, Some(29));
    }
}
