use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let blueprints = parse_blueprints(&content)?;

    let sum_ql: u32 = blueprints
        .iter()
        .inspect(|bp| println!("Checking blueprint {}", bp.id))
        .map(quality_level)
        .sum();
    println!("The sum of the quality level of all blueprints is {sum_ql}.");

    let max_geodes_product: u32 = blueprints
        .iter()
        .take(3)
        .inspect(|bp| println!("Checking blueprint {}", bp.id))
        .map(|bp| opened_geodes(bp, 32))
        .product();
    println!("The product of open geodes in 32 minutes for the first three blueprints is {max_geodes_product}.");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct Resources {
    ore: u32,
    clay: u32,
    obsidian: u32,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct Blueprint {
    id: u32,
    ore_bot: Resources,
    clay_bot: Resources,
    obsi_bot: Resources,
    geode_bot: Resources,
}

fn parse_blueprint(line: &str) -> Result<Blueprint, String> {
    // a regex would be really convenient here. But I don't want to use external dependencies.
    let (id_part, cost_part) = line
        .split_once(": Each ore robot costs ")
        .ok_or_else(|| format!("Unable to split cost part from ID part in line '{line}'"))?;
    let id: u32 = id_part
        .strip_prefix("Blueprint ")
        .ok_or_else(|| format!("Unexpected prefix in id part '{id_part}'"))?
        .parse()
        .map_err(|e| format!("unable to parse id in '{id_part}': {e}"))?;

    let (ore_bot_cost, rest) = cost_part
        .split_once(" ore. Each clay robot costs ")
        .ok_or_else(|| format!("Unable to split ore bot cost from rest in '{cost_part}'"))?;
    let ore_bot = Resources {
        ore: ore_bot_cost
            .parse()
            .map_err(|e| format!("Unable to parse ore bot cose '{ore_bot_cost}': {e}"))?,
        clay: 0,
        obsidian: 0,
    };

    let (clay_bot_cost, rest) = rest
        .split_once(" ore. Each obsidian robot costs ")
        .ok_or_else(|| format!("Unable to split clay bot cost from rest in '{rest}'"))?;
    let clay_bot = Resources {
        ore: clay_bot_cost
            .parse()
            .map_err(|e| format!("Unable to parse clay bot cost '{clay_bot_cost}': {e}"))?,
        clay: 0,
        obsidian: 0,
    };

    let (obsi_bot_cost, geode_bot_cost) = rest
        .split_once(" clay. Each geode robot costs ")
        .ok_or_else(|| {
            format!("Unable to split obsidian bot cost from geode bot cost in '{rest}'")
        })?;
    let (obsi_bot_ore, obsi_bot_clay) = obsi_bot_cost.split_once(" ore and ").ok_or_else(|| format!("Unable to split obsidian bot ore cost from obsidian bot clay cost in '{obsi_bot_cost}'"))?;
    let obsi_bot = Resources {
        ore: obsi_bot_ore
            .parse()
            .map_err(|e| format!("Unable to parse obsidian bot ore cost '{obsi_bot_ore}': {e}"))?,
        clay: obsi_bot_clay.parse().map_err(|e| {
            format!("Unable to parse obsidian bot clay cost '{obsi_bot_clay}': {e}")
        })?,
        obsidian: 0,
    };

    let geode_bot_cost = geode_bot_cost
        .strip_suffix(" obsidian.")
        .ok_or_else(|| format!("Unexpected suffix for geode bot cost in '{geode_bot_cost}'"))?;
    let (geode_bot_ore, geode_bot_obsi) = geode_bot_cost.split_once(" ore and ").ok_or_else(|| format!("Unable to split geode bot ore cost from geode bot obsidian cost in '{geode_bot_cost}'"))?;
    let geode_bot = Resources {
        ore: geode_bot_ore
            .parse()
            .map_err(|e| format!("Unable to parse geode bot ore cost '{geode_bot_ore}': {e}"))?,
        clay: 0,
        obsidian: geode_bot_obsi.parse().map_err(|e| {
            format!("Unable to parse geode bot obsidian cost '{geode_bot_obsi}': {e}")
        })?,
    };

    Ok(Blueprint {
        id,
        ore_bot,
        clay_bot,
        obsi_bot,
        geode_bot,
    })
}

fn parse_blueprints(input: &str) -> Result<Vec<Blueprint>, String> {
    input.lines().map(parse_blueprint).collect()
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct State {
    res: Resources,
    ore_bots: u32,
    clay_bots: u32,
    obsi_bots: u32,
    geode_bots: u32,
}

fn quality_level(blueprint: &Blueprint) -> u32 {
    opened_geodes(blueprint, 24) * blueprint.id
}

#[derive(Clone, Eq, Debug)]
struct QueueEntry {
    state: State,
    geodes: u32,
    time_left: u32,
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time_left
            .cmp(&other.time_left)
            .then(self.geodes.cmp(&other.geodes))
    }
}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

fn opened_geodes(blueprint: &Blueprint, max_time: u32) -> u32 {
    let mut queue: BinaryHeap<QueueEntry> = BinaryHeap::with_capacity(4096);
    let mut seen: HashSet<State> = HashSet::with_capacity(4096);

    let max_ore_cost = blueprint
        .ore_bot
        .ore
        .max(blueprint.clay_bot.ore)
        .max(blueprint.obsi_bot.ore)
        .max(blueprint.geode_bot.ore);

    queue.push(QueueEntry {
        state: State {
            res: Resources {
                ore: 0,
                clay: 0,
                obsidian: 0,
            },
            ore_bots: 1,
            clay_bots: 0,
            obsi_bots: 0,
            geode_bots: 0,
        },
        geodes: 0,
        time_left: max_time,
    });

    let mut max_geodes: u32 = 0;

    while let Some(QueueEntry {
        state: current,
        geodes: current_geodes,
        time_left,
    }) = queue.pop()
    {
        if seen.contains(&current) {
            continue;
        }
        seen.insert(current.clone());

        max_geodes = max_geodes.max(current_geodes + time_left * current.geode_bots);

        if time_left <= 1 {
            continue;
        }

        // it does not make sense to build more ore bots than the amount of ore we can spent in the
        // same time
        if current.ore_bots < max_ore_cost {
            if let Some(time) = time_to_build(&current, blueprint.ore_bot, time_left - 2) {
                queue.push(QueueEntry {
                    state: State {
                        res: Resources {
                            ore: current.res.ore + time * current.ore_bots - blueprint.ore_bot.ore,
                            clay: current.res.clay + time * current.clay_bots
                                - blueprint.ore_bot.clay,
                            obsidian: current.res.obsidian + time * current.obsi_bots
                                - blueprint.ore_bot.obsidian,
                        },
                        ore_bots: current.ore_bots + 1,
                        ..current
                    },
                    geodes: current_geodes + time * current.geode_bots,
                    time_left: time_left - time,
                });
            }
        }
        if let Some(time) = time_to_build(&current, blueprint.clay_bot, time_left - 2) {
            queue.push(QueueEntry {
                state: State {
                    res: Resources {
                        ore: current.res.ore + time * current.ore_bots - blueprint.clay_bot.ore,
                        clay: current.res.clay + time * current.clay_bots - blueprint.clay_bot.clay,
                        obsidian: current.res.obsidian + time * current.obsi_bots
                            - blueprint.clay_bot.obsidian,
                    },
                    clay_bots: current.clay_bots + 1,
                    ..current
                },
                geodes: current_geodes + time * current.geode_bots,
                time_left: time_left - time,
            });
        }
        if let Some(time) = time_to_build(&current, blueprint.obsi_bot, time_left - 2) {
            queue.push(QueueEntry {
                state: State {
                    res: Resources {
                        ore: current.res.ore + time * current.ore_bots - blueprint.obsi_bot.ore,
                        clay: current.res.clay + time * current.clay_bots - blueprint.obsi_bot.clay,
                        obsidian: current.res.obsidian + time * current.obsi_bots
                            - blueprint.obsi_bot.obsidian,
                    },
                    obsi_bots: current.obsi_bots + 1,
                    ..current
                },
                geodes: current_geodes + time * current.geode_bots,
                time_left: time_left - time,
            });
        }
        if let Some(time) = time_to_build(&current, blueprint.geode_bot, time_left) {
            queue.push(QueueEntry {
                state: State {
                    res: Resources {
                        ore: current.res.ore + time * current.ore_bots - blueprint.geode_bot.ore,
                        clay: current.res.clay + time * current.clay_bots
                            - blueprint.geode_bot.clay,
                        obsidian: current.res.obsidian + time * current.obsi_bots
                            - blueprint.geode_bot.obsidian,
                    },
                    geode_bots: current.geode_bots + 1,
                    ..current
                },

                geodes: current_geodes + time * current.geode_bots,
                time_left: time_left - time,
            });
        }
    }

    max_geodes
}

fn time_to_build(state: &State, cost: Resources, time_left: u32) -> Option<u32> {
    let ore_time = if state.res.ore >= cost.ore {
        0
    } else {
        let to_produce = cost.ore - state.res.ore;
        to_produce
            .checked_div(state.ore_bots)
            .map(|cost| cost + u32::from(to_produce.checked_rem(state.ore_bots) != Some(0)))?
    };

    let clay_time = if state.res.clay >= cost.clay {
        0
    } else {
        let to_produce = cost.clay - state.res.clay;
        to_produce
            .checked_div(state.clay_bots)
            .map(|cost| cost + u32::from(to_produce.checked_rem(state.clay_bots) != Some(0)))?
    };

    let obsi_time = if state.res.obsidian >= cost.obsidian {
        0
    } else {
        let to_produce = cost.obsidian - state.res.obsidian;
        to_produce
            .checked_div(state.obsi_bots)
            .map(|cost| cost + u32::from(to_produce.checked_rem(state.obsi_bots) != Some(0)))?
    };

    let time = ore_time.max(clay_time).max(obsi_time) + 1;

    // if no time is left after building it, it does not help
    if time < time_left {
        Some(time)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quality_level_works_for_first_example() {
        // given
        let blueprint = Blueprint {
            id: 1,
            ore_bot: Resources {
                ore: 4,
                clay: 0,
                obsidian: 0,
            },
            clay_bot: Resources {
                ore: 2,
                clay: 0,
                obsidian: 0,
            },
            obsi_bot: Resources {
                ore: 3,
                clay: 14,
                obsidian: 0,
            },
            geode_bot: Resources {
                ore: 2,
                clay: 0,
                obsidian: 7,
            },
        };

        // when
        let ql = quality_level(&blueprint);

        assert_eq!(ql, 9);
    }

    #[test]
    fn quality_level_works_for_second_example() {
        // given
        let blueprint = Blueprint {
            id: 1,
            ore_bot: Resources {
                ore: 2,
                clay: 0,
                obsidian: 0,
            },
            clay_bot: Resources {
                ore: 3,
                clay: 0,
                obsidian: 0,
            },
            obsi_bot: Resources {
                ore: 3,
                clay: 8,
                obsidian: 0,
            },
            geode_bot: Resources {
                ore: 3,
                clay: 0,
                obsidian: 12,
            },
        };

        // when
        let ql = quality_level(&blueprint);

        assert_eq!(ql, 12);
    }
}
