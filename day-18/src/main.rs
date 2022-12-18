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
}
