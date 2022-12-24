use std::collections::{HashSet, VecDeque};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let blizz = parse_input(&content)?;

    if let Some(time) = shortest_path(&blizz) {
        println!("The shortest path through the blizzard takes {time} minutes.");
    } else {
        println!("There is no way to get though this blizzard.");
    }

    Ok(())
}

struct Blizz {
    width: usize,
    right: Vec<bool>,
    down: Vec<bool>,
    left: Vec<bool>,
    up: Vec<bool>,
}

impl Blizz {
    fn blocked_at_time(&self, x: usize, y: usize, time: usize) -> bool {
        let width = self.width;
        let height = self.right.len() / width;
        x >= width
            || y >= height
            || self.left[(x + time) % width + y * width]
            || self.up[x + ((y + time) % height) * width]
            || self.right
                [(x as isize - time as isize).rem_euclid(width as isize) as usize + y * width]
            || self.down
                [x + (y as isize - time as isize).rem_euclid(height as isize) as usize * width]
    }
}

fn parse_input(input: &str) -> Result<Blizz, String> {
    let width = input
        .lines()
        .map(|line| line.len() - 2)
        .next()
        .ok_or_else(|| "input is empty".to_owned())?;
    let right = parse_direction(input, '>');
    let down = parse_direction(input, 'v');
    let left = parse_direction(input, '<');
    let up = parse_direction(input, '^');

    if right.len() != down.len() || right.len() != left.len() || right.len() != up.len() {
        Err("map sizes are not uniform for some reason".to_owned())
    } else if right.len() % width != 0 {
        Err("map is not a rectangle".to_owned())
    } else {
        Ok(Blizz {
            width,
            right,
            down,
            left,
            up,
        })
    }
}

fn parse_direction(input: &str, dir: char) -> Vec<bool> {
    input
        .lines()
        .skip(1)
        .filter(|line| !line.starts_with("##"))
        .flat_map(|line| line.chars().filter(|c| *c != '#').map(|c| c == dir))
        .collect()
}

fn shortest_path(blizz: &Blizz) -> Option<usize> {
    let width = blizz.width;
    let height = blizz.right.len() / width;
    // in case this is too large, we can possibly maxe this value smaller by using the lcm of width
    // and height
    let max_cycle = width * height;
    let mut queue: VecDeque<(usize, isize, usize)> = VecDeque::with_capacity(1024);
    let mut seen: HashSet<(usize, isize, usize)> = HashSet::with_capacity(max_cycle);

    queue.push_back((0, -1, 0));

    while let Some((x, y, time)) = queue.pop_front() {
        if seen.contains(&(x, y, time % max_cycle)) {
            continue;
        }
        seen.insert((x, y, time % max_cycle));
        let next_time = time + 1;

        // move down into target position
        if x == width - 1 && y == height as isize - 1 {
            return Some(next_time);
        }
        // wait
        if y == -1 || !blizz.blocked_at_time(x, y as usize, next_time) {
            queue.push_back((x, y, next_time));
        }
        // move up (note: it never makes sense to move back to the start, we could have just waited
        // there)
        if y > 0 && !blizz.blocked_at_time(x, (y - 1) as usize, next_time) {
            queue.push_back((x, y - 1, next_time));
        }
        // move left
        if y >= 0 && x > 0 && !blizz.blocked_at_time(x - 1, y as usize, next_time) {
            queue.push_back((x - 1, y, next_time));
        }
        // move right
        if y >= 0 && !blizz.blocked_at_time(x + 1, y as usize, next_time) {
            queue.push_back((x + 1, y, next_time));
        }
        // move down (regularly)
        if !blizz.blocked_at_time(x, (y + 1) as usize, next_time) {
            queue.push_back((x, y + 1, next_time));
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#
"#;

    #[test]
    fn shortest_path_works_for_example() {
        // given
        let blizz = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let shortest_time = shortest_path(&blizz);

        // then
        assert_eq!(shortest_time, Some(18));
    }

    #[test]
    fn parse_input_works_for_minimal_example() {
        // given
        let input = r#"#.##
#v^#
#<>#
##.#
"#;
        // when
        let result = parse_input(input);

        // then
        let blizz = result.expect("expected successful parsing");
        assert_eq!(blizz.width, 2);
        assert_eq!(&blizz.left, &[false, false, true, false]);
        assert_eq!(&blizz.right, &[false, false, false, true]);
        assert_eq!(&blizz.up, &[false, true, false, false]);
        assert_eq!(&blizz.down, &[true, false, false, false]);
    }
}
