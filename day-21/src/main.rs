use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let monkeys = parse_input(&content)?;

    if let Some(root) = find_root_value(&monkeys) {
        println!("The value of the root monkey is {root}");
    } else {
        println!("I can't find a value for the root monkey!");
    }

    if let Some(value) = human_value(&monkeys) {
        println!("You must yell the numner {value} to satisfy the equation");
    } else {
        println!("I have no idea what number to yell.");
    }

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Action<'s> {
    Lit(i64),
    Add(&'s str, &'s str),
    Sub(&'s str, &'s str),
    Mul(&'s str, &'s str),
    Div(&'s str, &'s str),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Monkey<'s> {
    name: &'s str,
    action: Action<'s>,
}

fn parse_monkey(line: &str) -> Result<Monkey, String> {
    let (name, rest) = line
        .split_once(": ")
        .ok_or_else(|| format!("unable to split monkey name from action in line '{line}'"))?;

    if let Ok(literal) = rest.parse::<i64>() {
        return Ok(Monkey {
            name,
            action: Action::Lit(literal),
        });
    }

    if let Some((lhs, rhs)) = rest.split_once(" + ") {
        return Ok(Monkey {
            name,
            action: Action::Add(lhs, rhs),
        });
    }

    if let Some((lhs, rhs)) = rest.split_once(" - ") {
        return Ok(Monkey {
            name,
            action: Action::Sub(lhs, rhs),
        });
    }

    if let Some((lhs, rhs)) = rest.split_once(" * ") {
        return Ok(Monkey {
            name,
            action: Action::Mul(lhs, rhs),
        });
    }

    if let Some((lhs, rhs)) = rest.split_once(" / ") {
        return Ok(Monkey {
            name,
            action: Action::Div(lhs, rhs),
        });
    }
    Err(format!("unable to interpret monkey action '{rest}'"))
}

fn parse_input(input: &str) -> Result<Vec<Monkey>, String> {
    input.lines().map(parse_monkey).collect()
}

fn action_value(action: Action, values: &HashMap<&str, i64>) -> Option<i64> {
    Some(match action {
        Action::Lit(v) => v,
        Action::Add(l, r) => values.get(l)? + values.get(r)?,
        Action::Sub(l, r) => values.get(l)? - values.get(r)?,
        Action::Mul(l, r) => values.get(l)? * values.get(r)?,
        Action::Div(l, r) => values.get(l)? / values.get(r)?,
    })
}

fn reverse_action_value<'s>(
    action: Action<'s>,
    values: &HashMap<&str, i64>,
    result: i64,
) -> Option<(&'s str, i64)> {
    match action {
        Action::Lit(_) => None,
        Action::Add(l, r) => match (values.get(l), values.get(r)) {
            (Some(vl), None) => Some((r, result - vl)),
            (None, Some(vr)) => Some((l, result - vr)),
            _ => None,
        },
        Action::Sub(l, r) => match (values.get(l), values.get(r)) {
            (Some(vl), None) => Some((r, vl - result)),
            (None, Some(vr)) => Some((l, result + vr)),
            _ => None,
        },
        Action::Mul(l, r) => match (values.get(l), values.get(r)) {
            (Some(vl), None) => Some((r, result / vl)),
            (None, Some(vr)) => Some((l, result / vr)),
            _ => None,
        },
        Action::Div(l, r) => match (values.get(l), values.get(r)) {
            (Some(vl), None) => Some((r, vl / result)),
            (None, Some(vr)) => Some((l, result * vr)),
            _ => None,
        },
    }
}

fn monkey_values<'s>(monkeys: &[Monkey<'s>]) -> HashMap<&'s str, i64> {
    // let's try a simple (but quadratic) solution first
    let mut values: HashMap<&str, i64> = HashMap::with_capacity(monkeys.len());

    let mut found_anything = true;
    while found_anything {
        found_anything = false;
        for monkey in monkeys {
            if values.contains_key(monkey.name) {
                continue;
            }
            if let Some(value) = action_value(monkey.action, &values) {
                values.insert(monkey.name, value);
                found_anything = true;
            }
        }
    }
    values
}

fn find_root_value(monkeys: &[Monkey]) -> Option<i64> {
    monkey_values(monkeys).get("root").copied()
}

fn human_value(monkeys: &[Monkey]) -> Option<i64> {
    let non_root_monkeys: Vec<Monkey> = monkeys
        .iter()
        .filter(|monkey| monkey.name != "humn" && monkey.name != "root")
        .copied()
        .collect();
    let partial_values = monkey_values(&non_root_monkeys);
    let monkeys_by_name: HashMap<&str, Monkey> = monkeys
        .iter()
        .map(|monkey| (monkey.name, *monkey))
        .collect();

    // Assumption for this solution: there is only on path from root to humn.
    // For more general cases, this approach won't work
    let root_monkey = monkeys_by_name.get("root")?;
    let (lhs, rhs) = match root_monkey.action {
        Action::Lit(_) => return None,
        Action::Add(l, r) => (l, r),
        Action::Sub(l, r) => (l, r),
        Action::Mul(l, r) => (l, r),
        Action::Div(l, r) => (l, r),
    };
    let (mut unknown, mut value) = match (partial_values.get(lhs), partial_values.get(rhs)) {
        (Some(lv), None) => (rhs, *lv),
        (None, Some(rv)) => (lhs, *rv),
        _ => {
            return None;
        }
    };
    loop {
        if unknown == "humn" {
            return Some(value);
        }
        (unknown, value) =
            reverse_action_value(monkeys_by_name.get(unknown)?.action, &partial_values, value)?;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32
"#;

    #[test]
    fn find_root_value_works_for_example() {
        // given
        let monkeys = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let result = find_root_value(&monkeys);

        // then
        assert_eq!(result, Some(152));
    }

    #[test]
    fn human_value_works_for_example() {
        // given
        let monkeys = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let result = human_value(&monkeys);

        // then
        assert_eq!(result, Some(301));
    }
}
