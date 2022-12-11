use std::env;
use std::fs::read_to_string;
use std::mem::swap;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let monkeys = parse_monkeys(&content)?;

    let monkeys_after_20 = run_rounds(monkeys.clone(), 20, 3)?;
    let mb = monkey_business(&monkeys_after_20);
    println!("After 20 rounds, the monkey business is {mb}.");

    let monkeys_unlimited_10000 = run_rounds(monkeys, 10000, 1)?;
    let mb = monkey_business(&monkeys_unlimited_10000);
    println!("After 10000 rounds with unlimited worry level, the monkey business is {mb}");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Op {
    Add,
    Mul,
}

impl Op {
    fn run(self, left: u64, right: u64) -> u64 {
        match self {
            Op::Add => left + right,
            Op::Mul => left * right,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Monkey {
    items: Vec<u64>,
    operator: Op,
    // if operand is None, the old value needs to be used
    operand: Option<u64>,
    test_mod: u64,
    test_true: usize,
    test_false: usize,
    inspections: usize,
}

fn parse_monkey(block: &str) -> Result<Monkey, String> {
    let mut lines = block.lines();
    // skip the check for the monkey's ID, assume they are always indexed in order.
    // Just do a quick sanity check
    if !lines
        .next()
        .map(|line| line.starts_with("Monkey "))
        .unwrap_or(false)
    {
        return Err("Monkey block did not start with 'Monkey '!".to_owned());
    }

    let line = lines
        .next()
        .ok_or_else(|| "Expected line with starting items, found nothing".to_owned())?;
    let items: Vec<u64> = line
        .strip_prefix("  Starting items: ")
        .ok_or_else(|| format!("Unable to parse '{line}' as starting items"))?
        .split(", ")
        .map(|n| {
            n.parse::<u64>()
                .map_err(|e| format!("Error while parsing worry level of '{n}': {e}"))
        })
        .collect::<Result<Vec<u64>, String>>()?;

    let line = lines
        .next()
        .ok_or_else(|| "Expected line with inspection operation, found nothing".to_owned())?;
    let (operator, operand) = line
        .strip_prefix("  Operation: new = old ")
        .ok_or_else(|| format!("Unable to parse line '{line}' as operation"))?
        .split_once(' ')
        .ok_or_else(|| format!("Unable to split operator and operand in line '{line}'"))?;
    let operator = match operator {
        "+" => Op::Add,
        "*" => Op::Mul,
        _ => {
            return Err(format!("Unexpected operator: '{operator}'"));
        }
    };
    let operand: Option<u64> = if operand == "old" {
        None
    } else {
        Some(
            operand
                .parse()
                .map_err(|e| format!("Unable to parse operand '{operand}': {e}"))?,
        )
    };

    let line = lines
        .next()
        .ok_or_else(|| "Expected line with test condition, found nothing".to_owned())?;
    let test_mod: u64 = line
        .strip_prefix("  Test: divisible by ")
        .ok_or_else(|| format!("Unable to parse line '{line}' as test condition"))?
        .parse()
        .map_err(|e| format!("Unable to parse test condition divisor: {e}"))?;

    let line = lines.next().ok_or_else(|| {
        "Expected line with action on fulfilled test condition, found nothing".to_owned()
    })?;
    let test_true: usize = line
        .strip_prefix("    If true: throw to monkey ")
        .ok_or_else(|| format!("Unable to parse line '{line}' as action"))?
        .parse()
        .map_err(|e| format!("Unable to parse target monkey index: {e}"))?;

    let line = lines.next().ok_or_else(|| {
        "Expected line with action on unfulfilled test condition, found nothing".to_owned()
    })?;
    let test_false: usize = line
        .strip_prefix("    If false: throw to monkey ")
        .ok_or_else(|| format!("Unable to parse line '{line}' as action"))?
        .parse()
        .map_err(|e| format!("Unable to parse target monkey index: {e}"))?;

    Ok(Monkey {
        items,
        operator,
        operand,
        test_mod,
        test_true,
        test_false,
        inspections: 0,
    })
}

fn parse_monkeys(input: &str) -> Result<Vec<Monkey>, String> {
    input
        .split("\n\n")
        .filter(|block| !block.is_empty())
        .map(parse_monkey)
        .collect()
}

fn next_round(
    mut monkeys: Vec<Monkey>,
    worry_level_divisor: u64,
    worry_level_mod: u64,
) -> Result<Vec<Monkey>, String> {
    for monkey_i in 0..monkeys.len() {
        // this is assuming that a monkey never throws items to itself
        for item_i in 0..monkeys[monkey_i].items.len() {
            let monkey = &monkeys[monkey_i];
            let worry_level = (monkey.operator.run(
                monkey.items[item_i],
                monkey.operand.unwrap_or(monkey.items[item_i]),
            ) / worry_level_divisor)
                % worry_level_mod;
            let target_i = if worry_level % monkey.test_mod == 0 {
                monkey.test_true
            } else {
                monkey.test_false
            };
            if target_i == monkey_i {
                // sanity check for assumption above
                return Err(format!(
                    "Apparently, monkey {monkey_i} throws things to itself ☹"
                ));
            }
            monkeys
                .get_mut(target_i)
                .ok_or_else(|| {
                    format!(
                        "Monkey {monkey_i} tried to throw something to missing monkey {target_i}"
                    )
                })?
                .items
                .push(worry_level);
        }
        monkeys[monkey_i].inspections += monkeys[monkey_i].items.len();
        monkeys[monkey_i].items.clear();
    }
    Ok(monkeys)
}

fn run_rounds(
    mut monkeys: Vec<Monkey>,
    rounds: u64,
    worry_level_divisor: u64,
) -> Result<Vec<Monkey>, String> {
    // worry_level_divisor and worry_level_mod don't play nice together and I have no patience to
    // figure out why, so I will only effectively use the modulo if there is a divisor != 1
    let worry_level_mod: u64 = if worry_level_divisor == 1 {
        monkeys.iter().map(|m| m.test_mod).product()
    } else {
        u64::MAX
    };
    for _ in 0..rounds {
        monkeys = next_round(monkeys, worry_level_divisor, worry_level_mod)?;
    }
    Ok(monkeys)
}

fn monkey_business(monkeys: &[Monkey]) -> usize {
    let mut first = 0;
    let mut second = 0;
    for monkey in monkeys {
        if monkey.inspections > second {
            second = monkey.inspections;
            if second > first {
                swap(&mut first, &mut second);
            }
        }
    }
    first * second
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn run_rounds_works_for_example() {
        // given
        let monkeys = parse_monkeys(EXAMPLE).expect("expected successful parsing");

        // when
        let result = run_rounds(monkeys, 20, 3);

        // then
        let after_20 = result.expect("expected successful run");
        assert_eq!(monkey_business(&after_20), 10605);
    }

    #[test]
    fn run_rounds_works_for_unlimited_worry_level() {
        // given
        let monkeys = parse_monkeys(EXAMPLE).expect("expected successful parsing");

        // when
        let result = run_rounds(monkeys, 10000, 1);

        // then
        let after_20 = result.expect("expected successful run");
        assert_eq!(monkey_business(&after_20), 2713310158);
    }

    const EXAMPLE: &str = r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1
"#;
}
