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

fn find_root_value(monkeys: &[Monkey]) -> Option<i64> {
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
    values.get("root").copied()
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
}
