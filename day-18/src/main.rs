use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let cubes = parse_input(&content)?;

    let a = surface_area(&cubes);
    println!("The surface area of the lava droplets is {a}");

    let b = reachable_surface_area(&cubes);
    println!("The exterior surface area is {b}");

    Ok(())
}

type P = (i8, i8, i8);

fn parse_line(line: &str) -> Result<P, String> {
    let mut ords = line.split(',');
    let x: i8 = ords
        .next()
        .ok_or_else(|| format!("expected x coordinate in line '{line}'"))?
        .parse()
        .map_err(|e| format!("unable to parse x coordinate in line '{line}': {e}"))?;
    let y: i8 = ords
        .next()
        .ok_or_else(|| format!("expected y coordinate in line '{line}'"))?
        .parse()
        .map_err(|e| format!("unable to parse y coordinate in line '{line}': {e}"))?;
    let z: i8 = ords
        .next()
        .ok_or_else(|| format!("expected z coordinate in line '{line}'"))?
        .parse()
        .map_err(|e| format!("unable to parse z coordinate in line '{line}' {e}"))?;

    Ok((x, y, z))
}

fn parse_input(input: &str) -> Result<HashSet<P>, String> {
    input.lines().map(parse_line).collect()
}

fn neighbours((x, y, z): P) -> [P; 6] {
    [
        (x - 1, y, z),
        (x + 1, y, z),
        (x, y - 1, z),
        (x, y + 1, z),
        (x, y, z - 1),
        (x, y, z + 1),
    ]
}

fn surface_area(cubes: &HashSet<P>) -> u32 {
    // let's use a primitive approach, who knows what will come later
    cubes
        .iter()
        .map(|cube| {
            neighbours(*cube)
                .iter()
                .filter(|neighbour| !cubes.contains(neighbour))
                .count() as u32
        })
        .sum::<u32>()
}

fn reachable_surface_area(cubes: &HashSet<P>) -> u32 {
    // also a primitive approach, but it should be doable
    let min_x = cubes.iter().map(|(x, _, _)| x).min().copied().unwrap_or(1) - 1;
    let min_y = cubes.iter().map(|(_, y, _)| y).min().copied().unwrap_or(1) - 1;
    let min_z = cubes.iter().map(|(_, _, z)| z).min().copied().unwrap_or(1) - 1;
    let max_x = cubes.iter().map(|(x, _, _)| x).max().copied().unwrap_or(-1) + 1;
    let max_y = cubes.iter().map(|(_, y, _)| y).max().copied().unwrap_or(-1) + 1;
    let max_z = cubes.iter().map(|(_, _, z)| z).max().copied().unwrap_or(-1) + 1;

    let max_space = ((max_x as isize - min_x as isize + 1)
        * (max_y as isize - min_y as isize + 1)
        * (max_z as isize - min_z as isize + 1)) as usize;
    let mut flooded: HashSet<P> = HashSet::with_capacity(max_space);
    let mut surface_area: u32 = 0;
    let mut stack: Vec<P> = Vec::with_capacity(max_space);
    stack.push((min_x, min_y, min_z));

    while let Some(p) = stack.pop() {
        if cubes.contains(&p) {
            surface_area += 1;
            continue;
        }
        if flooded.contains(&p) {
            continue;
        }
        flooded.insert(p);
        for (x, y, z) in neighbours(p) {
            if x >= min_x
                && x <= max_x
                && y >= min_y
                && y <= max_y
                && z >= min_z
                && z <= max_z
                && !flooded.contains(&(x, y, z))
            {
                stack.push((x, y, z));
            }
        }
    }
    surface_area
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5
"#;

    #[test]
    fn surface_area_works_for_example() {
        // given
        let cubes = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let a = surface_area(&cubes);

        // then
        assert_eq!(a, 64);
    }

    #[test]
    fn reachable_surface_area_works_for_example() {
        // given
        let cubes = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let a = reachable_surface_area(&cubes);

        // then
        assert_eq!(a, 58);
    }
}
