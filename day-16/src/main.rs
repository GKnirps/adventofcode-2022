use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let valves = parse_input(&content)?;

    if let Some(max_release) = find_max_release(&valves) {
        println!("The most pressure I can release is {max_release}");
    } else {
        println!("There must be something wrong with our map…");
    }

    Ok(())
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Valve<'s> {
    id: &'s str,
    flow_rate: u32,
    neighbours: Vec<&'s str>,
}

fn parse_valve(line: &str) -> Result<Valve, String> {
    let (valve, connections) = line
        .split_once(';')
        .ok_or_else(|| format!("unable to split line '{line}'"))?;

    let (id, flow_rate) = valve
        .split_once(" has flow rate=")
        .ok_or_else(|| format!("unable to split ID from low rate in '{valve}'"))?;
    let id = id
        .strip_prefix("Valve ")
        .ok_or_else(|| format!("invalid prefix in valve ID: '{id}'"))?;
    let flow_rate: u32 = flow_rate
        .parse()
        .map_err(|e| format!("unable to parse flow rate '{flow_rate}: {e}'"))?;

    let mut neighbours: Vec<&str> = connections
        .strip_prefix(" tunnels lead to valves ")
        .or_else(|| connections.strip_prefix(" tunnel leads to valve "))
        .ok_or_else(|| format!("invalid prefix for neighbours: {connections}"))?
        .split(", ")
        .collect();
    neighbours.sort_unstable();

    Ok(Valve {
        id,
        flow_rate,
        neighbours,
    })
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct State<'s> {
    pos: &'s str,
    time_left: u32,
    valves_open: Vec<&'s str>,
}

impl<'s> State<'s> {
    fn can_open_valve(&self, valve: &Valve) -> bool {
        valve.flow_rate > 0 && !self.valves_open.contains(&valve.id)
    }
    // will return None if no time is left
    fn with_open_valve(&self, valve: &Valve) -> Option<(Self, u32)> {
        let mut valves_open: Vec<&'s str> = Vec::with_capacity(self.valves_open.len());
        let p = self.valves_open.partition_point(|id| *id < self.pos);
        if p < self.valves_open.len() {
            valves_open.extend_from_slice(&self.valves_open[..p]);
            valves_open.push(self.pos);
            valves_open.extend_from_slice(&self.valves_open[p..])
        } else {
            valves_open.extend_from_slice(&self.valves_open);
            valves_open.push(self.pos);
        }

        let time_left = self.time_left.checked_sub(1)?;

        let points = time_left * valve.flow_rate;

        Some((
            State {
                pos: self.pos,
                time_left,
                valves_open,
            },
            points,
        ))
    }

    // will return None if no time is left. assumes you _can_ move from the current valve to the
    // next one in one step, check that beforehand
    fn with_move(&self, next_pos: &'s str) -> Option<Self> {
        Some(State {
            pos: next_pos,
            time_left: self.time_left.checked_sub(1)?,
            valves_open: self.valves_open.clone(),
        })
    }
}

fn parse_input(input: &str) -> Result<HashMap<&str, Valve>, String> {
    input
        .lines()
        .map(|line| parse_valve(line).map(|valve| (valve.id, valve)))
        .collect()
}

#[derive(Clone, Eq, Debug)]
struct HeapEntry<'s>(State<'s>, u32);

impl<'s> Ord for HeapEntry<'s> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

impl<'s> PartialOrd for HeapEntry<'s> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'s> PartialEq for HeapEntry<'s> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

fn find_max_release(valves: &HashMap<&str, Valve>) -> Option<u32> {
    let mut queue: BinaryHeap<HeapEntry> = BinaryHeap::with_capacity(valves.len() * 2);
    queue.push(HeapEntry(
        State {
            pos: "AA",
            time_left: 30,
            valves_open: vec![],
        },
        0,
    ));
    let mut seen: HashMap<State, u32> = HashMap::with_capacity(valves.len() * 16);

    while let Some(HeapEntry(current, current_points)) = queue.pop() {
        if seen
            .get(&current)
            .map(|points| *points >= current_points)
            .unwrap_or(false)
            || current.time_left == 0
        {
            continue;
        }

        let current_valve = valves.get(current.pos)?;
        if current.can_open_valve(current_valve) {
            let (opened, points) = current.with_open_valve(current_valve)?;
            queue.push(HeapEntry(opened, current_points + points));
        }
        for neighbour in &current_valve.neighbours {
            queue.push(HeapEntry(current.with_move(neighbour)?, current_points));
        }

        seen.insert(current, current_points);
    }

    seen.values().max().copied()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II
"#;

    #[test]
    fn find_max_release_works_for_example() {
        // given
        let valves = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let result = find_max_release(&valves);

        // then
        assert_eq!(result, Some(1651));
    }
}
