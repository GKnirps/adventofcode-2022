use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let sensors = parse_input(&content)?;

    let covered_in_row_2000000 = covered_cells_in_row(&sensors, 2000000);
    println!("In the row where y = 2000000, {covered_in_row_2000000} cells are covered.");

    Ok(())
}

type P = (i64, i64);

fn parse_sensor(line: &str) -> Result<(P, P), String> {
    let (sensor, beacon) = line
        .split_once(": closest beacon is at x=")
        .ok_or_else(|| format!("malformed sensor/beacon line '{line}'"))?;

    let (sensor_x, sensor_y) = sensor
        .split_once(", y=")
        .ok_or_else(|| format!("malformed sensor coordinates: '{sensor}'"))?;
    let sensor_x: i64 = sensor_x
        .strip_prefix("Sensor at x=")
        .ok_or_else(|| format!("sensor x is missing a correct prefix: '{sensor_x}'"))?
        .parse()
        .map_err(|e| format!("Unable to parse sensor x: {e}"))?;
    let sensor_y: i64 = sensor_y
        .parse()
        .map_err(|e| format!("Unable to parse sensor y: {e}"))?;

    let (beacon_x, beacon_y) = beacon
        .split_once(", y=")
        .ok_or_else(|| format!("malformed beacon coordinates: '{beacon}'"))?;
    let beacon_x: i64 = beacon_x
        .parse()
        .map_err(|e| format!("Unable to parse beacon x: {e}"))?;
    let beacon_y: i64 = beacon_y
        .parse()
        .map_err(|e| format!("Unable to parse beacon y: {e}"))?;

    Ok(((sensor_x, sensor_y), (beacon_x, beacon_y)))
}

fn parse_input(content: &str) -> Result<Vec<(P, P)>, String> {
    content.lines().map(parse_sensor).collect()
}

fn dist((x1, y1): P, (x2, y2): P) -> i64 {
    (x1 - x2).abs() + (y1 - y2).abs()
}

fn covered_cells_in_row(sensors: &[(P, P)], row: i64) -> i64 {
    let mut ranges: Vec<(i64, bool)> = sensors
        .iter()
        .filter_map(|(sensor, beacon)| {
            let range = dist(*sensor, *beacon);
            let row_dist = (sensor.1 - row).abs();
            if range >= row_dist {
                let row_range = range - row_dist;
                Some([(sensor.0 - row_range, false), (sensor.0 + row_range, true)])
            } else {
                None
            }
        })
        .flatten()
        .collect();

    ranges.sort_unstable_by(|(v1, is_to1), (v2, is_to2)| {
        let ord = v1.cmp(v2);
        if ord.is_eq() {
            is_to1.cmp(is_to2)
        } else {
            ord
        }
    });

    let mut depth: usize = 0;
    let mut from = i64::MIN;
    let mut count: i64 = 0;
    for (v, is_to) in ranges {
        if is_to {
            depth -= 1;
            if depth == 0 {
                count += v - from;
            }
        } else {
            if depth == 0 {
                from = v;
            }
            depth += 1;
        }
    }
    count
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3
"#;

    #[test]
    fn covered_cells_in_row_works_for_example() {
        // given
        let sensors = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let count = covered_cells_in_row(&sensors, 10);

        // then
        assert_eq!(count, 26);
    }
}
