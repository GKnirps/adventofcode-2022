use std::cmp::Ordering;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let pairs = parse_input(&content)?;

    let sum = ordered_pairs_index_sum(&pairs);
    println!("The sum of the indices of the correctly ordered pairs is {sum}");

    Ok(())
}

#[derive(Clone, Eq, Debug)]
enum Packet {
    Int(u32),
    List(Vec<Packet>),
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Packet::Int(left), Packet::Int(right)) => left.cmp(right),
            (Packet::List(left), Packet::List(right)) => left.cmp(right),
            (Packet::Int(left), Packet::List(right)) => {
                (&[Packet::Int(*left)] as &[Packet]).cmp(right as &[Packet])
            }
            (Packet::List(left), Packet::Int(right)) => {
                (left as &[Packet]).cmp(&[Packet::Int(*right)] as &[Packet])
            }
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Packet {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Packet {
    fn push(&mut self, p: Packet) -> Result<(), String> {
        match self {
            Packet::List(sub) => sub.push(p),
            _ => return Err("tried to push to int packet".to_owned()),
        };
        Ok(())
    }
}

fn parse_packet(line: &str) -> Result<Packet, String> {
    let mut stack: Vec<Packet> = Vec::with_capacity(line.len());
    for (i, c) in line.chars().enumerate() {
        match c {
            '[' => {
                stack.push(Packet::List(vec![]));
            }
            ']' => {
                let sub = stack
                    .pop()
                    .ok_or_else(|| format!("error parsing '{line}': unexpected closing bracket"))?;
                match sub {
                    Packet::List(_) => {
                        if let Some(top) = stack.last_mut() {
                            top.push(sub)?;
                        } else {
                            return Ok(sub);
                        }
                    }
                    Packet::Int(_) => {
                        let mut parent = stack.pop().ok_or_else(|| {
                            format!("error parsing '{line}': unexpected closing bracket")
                        })?;
                        parent.push(sub)?;
                        if let Some(top) = stack.last_mut() {
                            top.push(parent)?;
                        } else {
                            return Ok(parent);
                        }
                    }
                }
            }
            ',' => {
                let sub = stack
                    .pop()
                    .ok_or_else(|| format!("error parsing '{line}': unexpected comma"))?;
                match sub {
                    Packet::Int(_) => {
                        if let Some(top) = stack.last_mut() {
                            top.push(sub)?;
                        } else {
                            return Err(format!(
                                "Found comma, but nothing to append the int to in column {i}"
                            ));
                        }
                    }
                    // we don't check if the comma was misplaced (for now)
                    _ => stack.push(sub),
                };
            }
            _ => {
                let d = c
                    .to_digit(10)
                    .ok_or_else(|| format!("Unexpected char in input: '{c}'"))?;
                if let Some(Packet::Int(i)) = stack.last_mut() {
                    *i = *i * 10 + d;
                } else {
                    stack.push(Packet::Int(d));
                }
            }
        }
    }
    if stack.len() != 1 {
        Err(format!(
            "unexpected end of line, {} elements on the stack: {:?}",
            stack.len(),
            stack
        ))
    } else {
        Ok(stack.pop().unwrap())
    }
}

fn parse_pair(lines: &str) -> Result<(Packet, Packet), String> {
    let (first, second) = lines
        .split_once('\n')
        .ok_or_else(|| "expected packets to show up in pairs".to_owned())?;
    Ok((parse_packet(first)?, parse_packet(second.trim())?))
}

fn parse_input(input: &str) -> Result<Vec<(Packet, Packet)>, String> {
    input.split("\n\n").map(parse_pair).collect()
}

fn ordered_pairs_index_sum(pairs: &[(Packet, Packet)]) -> usize {
    pairs
        .iter()
        .enumerate()
        .filter(|(_, (first, second))| first < second)
        .map(|(i, _)| i + 1)
        .sum::<usize>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn packet_cmp_works_as_specified() {
        assert_eq!(Packet::Int(2).cmp(&Packet::Int(3)), Ordering::Less);
        assert_eq!(
            Packet::List(vec![]).cmp(&Packet::List(vec![Packet::Int(0)])),
            Ordering::Less
        );
        assert_eq!(
            Packet::List(vec![Packet::Int(0)]).cmp(&Packet::List(vec![Packet::Int(1)])),
            Ordering::Less
        );
        assert_eq!(
            Packet::List(vec![Packet::Int(0)]).cmp(&Packet::Int(1)),
            Ordering::Less
        );
        assert_eq!(
            Packet::Int(0).cmp(&Packet::List(vec![Packet::Int(1)])),
            Ordering::Less
        );
    }

    #[test]
    fn ordered_pairs_index_sum_works_for_example() {
        // given
        let pairs = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let sum = ordered_pairs_index_sum(&pairs);

        // then
        assert_eq!(sum, 13);
    }

    const EXAMPLE: &str = r#"[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]
"#;
}
