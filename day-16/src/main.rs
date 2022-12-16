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
    let valves = simplify_broken_valves(parse_input(&content)?).ok_or_else(|| {
        "Unable to simplify broken valves, something must be wrong with the input".to_owned()
    })?;

    // TODO: can valves be a simpel Vec?
    if let Some(max_release) = find_max_release(&valves) {
        println!("The most pressure I can release is {max_release}");
    } else {
        println!("There must be something wrong with our map…");
    }

    if let Some(max_release) = find_max_release_with_support(&valves) {
        println!("The most pressure I can release with elephant assistance is {max_release}");
    } else {
        println!("No really, there must be something wrong.");
    }

    Ok(())
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Valve<'s> {
    id: &'s str,
    flow_rate: u32,
    neighbours: Vec<(&'s str, u32)>,
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

    let neighbours: Vec<(&str, u32)> = connections
        .strip_prefix(" tunnels lead to valves ")
        .or_else(|| connections.strip_prefix(" tunnel leads to valve "))
        .ok_or_else(|| format!("invalid prefix for neighbours: {connections}"))?
        .split(", ")
        .map(|id| (id, 1))
        .collect();

    Ok(Valve {
        id,
        flow_rate,
        neighbours,
    })
}

// return None if there are broken references in the input
// assuming this is an undirected graph
fn simplify_broken_valves<'s>(
    mut valves: HashMap<&'s str, Valve<'s>>,
) -> Option<HashMap<&'s str, Valve<'s>>> {
    let broken_valves: Vec<&str> = valves
        .values()
        .filter(|valve| valve.flow_rate == 0)
        .map(|valve| valve.id)
        .collect();
    for valve_id in broken_valves {
        let broken_valve = valves.remove(valve_id)?;
        // assumption here is that all broken valves have exactly two neighbours. I they haven't
        // just re-insert the broken valve
        if broken_valve.neighbours.len() != 2 {
            valves.insert(broken_valve.id, broken_valve);
            continue;
        }
        let (n0, d0) = broken_valve.neighbours[0];
        let (n1, d1) = broken_valve.neighbours[1];

        let first_valve = valves.get_mut(n0)?;
        first_valve
            .neighbours
            .retain(|(id, _)| *id != broken_valve.id);
        first_valve.neighbours.push((n1, d0 + d1));

        let second_valve = valves.get_mut(n1)?;
        second_valve
            .neighbours
            .retain(|(id, _)| *id != broken_valve.id);
        second_valve.neighbours.push((n0, d0 + d1));
    }
    Some(valves)
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct State<'s> {
    pos: &'s str,
    elephant_pos: &'s str,
    time_left: u32,
    elephant_time_left: u32,
    valves_open: Vec<&'s str>,
}

impl<'s> State<'s> {
    // wil return None if not enough time is left
    fn with_move_and_open_valve(
        &self,
        valve: &Valve<'s>,
        dist: u32,
        human: bool,
    ) -> Option<(Self, u32)> {
        let (time_left, pos) = if human {
            (self.time_left.checked_sub(1 + dist)?, valve.id)
        } else {
            (self.time_left, self.pos)
        };

        let (elephant_time_left, elephant_pos) = if !human {
            (self.elephant_time_left.checked_sub(1 + dist)?, valve.id)
        } else {
            (self.elephant_time_left, self.elephant_pos)
        };

        let mut valves_open: Vec<&'s str> = Vec::with_capacity(self.valves_open.len() + 1);
        let p = self.valves_open.partition_point(|id| *id < valve.id);
        if p < self.valves_open.len() {
            valves_open.extend_from_slice(&self.valves_open[..p]);
            valves_open.push(valve.id);
            valves_open.extend_from_slice(&self.valves_open[p..])
        } else {
            valves_open.extend_from_slice(&self.valves_open);
            valves_open.push(valve.id);
        }

        let points = valve.flow_rate * if human { time_left } else { elephant_time_left };

        Some((
            State {
                pos,
                elephant_pos,
                time_left,
                elephant_time_left,
                valves_open,
            },
            points,
        ))
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
    find_max_release_for_initial_state(
        valves,
        State {
            pos: "AA",
            elephant_pos: "AA",
            time_left: 30,
            elephant_time_left: 0,
            valves_open: vec![],
        },
    )
}

fn distances_between_valves<'s>(
    valves: &HashMap<&'s str, Valve<'s>>,
) -> HashMap<(&'s str, &'s str), u32> {
    let mut d: HashMap<(&'s str, &'s str), u32> = valves
        .values()
        .flat_map(|valve| {
            valve
                .neighbours
                .iter()
                .map(|(n_id, dist)| ((valve.id, *n_id), *dist))
        })
        .chain(valves.values().map(|valve| ((valve.id, valve.id), 0)))
        .collect();

    for k in valves.values().map(|valve| valve.id) {
        for i in valves.values().map(|valve| valve.id) {
            for j in valves.values().map(|valve| valve.id) {
                let dij = d.get(&(i, j)).copied();
                let dik = d.get(&(i, k)).copied();
                let dkj = d.get(&(k, j)).copied();
                let dk = dik.and_then(|a| dkj.map(|b| a + b));
                if let Some(next_dij) = dij.and_then(|a| dk.map(|b| a.min(b))).or(dij).or(dk) {
                    d.insert((i, j), next_dij);
                }
            }
        }
    }
    d
}

fn find_max_release_with_support(valves: &HashMap<&str, Valve>) -> Option<u32> {
    find_max_release_for_initial_state(
        valves,
        State {
            pos: "AA",
            elephant_pos: "AA",
            time_left: 26,
            elephant_time_left: 26,
            valves_open: vec![],
        },
    )
}

fn find_max_release_for_initial_state(valves: &HashMap<&str, Valve>, state: State) -> Option<u32> {
    let valve_dists = distances_between_valves(valves);
    let mut queue: BinaryHeap<HeapEntry> = BinaryHeap::with_capacity(valves.len() * 2);
    queue.push(HeapEntry(state, 0));
    let mut seen: HashMap<State, u32> = HashMap::with_capacity(valves.len() * 16);

    while let Some(HeapEntry(current, current_points)) = queue.pop() {
        // this "inverted" stuff was a last minute addition to use this symmetry I just then
        // noticed in a final effort to make this run in somewhat acceptable time.
        // It still takes over a minute, but I guess there is some leverage in this symmetry to
        // make it run fast. But I worked the whole day on this and now I'm just tired.
        let mut inverted = current.clone();
        std::mem::swap(&mut inverted.pos, &mut inverted.elephant_pos);
        std::mem::swap(&mut inverted.time_left, &mut inverted.elephant_time_left);
        if seen
            .get(&current)
            .map(|points| *points >= current_points)
            .unwrap_or(false)
            || seen
                .get(&inverted)
                .map(|points| *points >= current_points)
                .unwrap_or(false)
        {
            continue;
        }

        let mut acted = false;
        if current.time_left >= current.elephant_time_left {
            for valve in valves
                .values()
                .filter(|v| v.flow_rate > 0 && !current.valves_open.contains(&v.id))
            {
                if let Some(dist) = valve_dists.get(&(current.pos, valve.id)) {
                    if let Some((opened, points)) =
                        current.with_move_and_open_valve(valve, *dist, true)
                    {
                        queue.push(HeapEntry(opened, current_points + points));
                        acted = true
                    }
                }
            }
        }
        if !acted {
            for valve in valves.values().filter(|v| {
                v.flow_rate > 0
                    && v.id != current.elephant_pos
                    && !current.valves_open.contains(&v.id)
            }) {
                if let Some(dist) = valve_dists.get(&(current.elephant_pos, valve.id)) {
                    if let Some((opened, points)) =
                        current.with_move_and_open_valve(valve, *dist, false)
                    {
                        queue.push(HeapEntry(opened, current_points + points));
                    }
                }
            }
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
        let simplified_valves =
            simplify_broken_valves(valves).expect("expected successful simplification");

        // when
        let result = find_max_release(&simplified_valves);

        // then
        assert_eq!(result, Some(1651));
    }

    #[test]
    fn find_max_release_with_support_works_for_example() {
        // given
        let valves = parse_input(EXAMPLE).expect("expected successful parsing");
        let simplified_valves =
            simplify_broken_valves(valves).expect("expected successful simplification");

        // when
        let result = find_max_release_with_support(&simplified_valves);

        // then
        assert_eq!(result, Some(1707));
    }

    #[test]
    fn distances_between_valves_works_for_small_graph() {
        // given
        let valves: HashMap<&str, Valve> = [
            (
                "A",
                Valve {
                    id: "A",
                    flow_rate: 1,
                    neighbours: vec![("B", 2)],
                },
            ),
            (
                "B",
                Valve {
                    id: "B",
                    flow_rate: 1,
                    neighbours: vec![("A", 2), ("C", 3)],
                },
            ),
            (
                "C",
                Valve {
                    id: "C",
                    flow_rate: 1,
                    neighbours: vec![("B", 3), ("D", 5), ("E", 50)],
                },
            ),
            (
                "D",
                Valve {
                    id: "D",
                    flow_rate: 1,
                    neighbours: vec![("C", 5), ("E", 1)],
                },
            ),
            (
                "E",
                Valve {
                    id: "E",
                    flow_rate: 1,
                    neighbours: vec![("C", 50), ("D", 1)],
                },
            ),
        ]
        .into_iter()
        .collect();

        // when
        let pairs = distances_between_valves(&valves);

        // then
        let expected_pairs: HashMap<(&str, &str), u32> = [
            (("A", "A"), 0),
            (("A", "B"), 2),
            (("A", "C"), 5),
            (("A", "D"), 10),
            (("A", "E"), 11),
            (("B", "A"), 2),
            (("B", "B"), 0),
            (("B", "C"), 3),
            (("B", "D"), 8),
            (("B", "E"), 9),
            (("C", "A"), 5),
            (("C", "B"), 3),
            (("C", "C"), 0),
            (("C", "D"), 5),
            (("C", "E"), 6),
            (("D", "A"), 10),
            (("D", "B"), 8),
            (("D", "C"), 5),
            (("D", "D"), 0),
            (("D", "E"), 1),
            (("E", "A"), 11),
            (("E", "B"), 9),
            (("E", "C"), 6),
            (("E", "D"), 1),
            (("E", "E"), 0),
        ]
        .into_iter()
        .collect();
        assert_eq!(pairs.len(), expected_pairs.len());
        assert_eq!(pairs, expected_pairs);
    }
}
