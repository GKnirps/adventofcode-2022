use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let strategy_guide = parse(&content)?;

    let strat_score = evaluate_part1(&strategy_guide);
    println!("The score of the strategy guide is {strat_score}");

    Ok(())
}

fn parse(content: &str) -> Result<Vec<(Hand, Hand)>, String> {
    content.lines().map(parse_line).collect()
}

fn parse_line(line: &str) -> Result<(Hand, Hand), String> {
    let (left, right) = line
        .split_once(' ')
        .ok_or_else(|| format!("line '{line}' has no whitespace to split"))?;
    let left_hand = match left {
        "A" => Hand::Rock,
        "B" => Hand::Paper,
        "C" => Hand::Scissors,
        s => return Err(format!("'{s}' is not a valid hand")),
    };
    let right_hand = match right {
        "X" => Hand::Rock,
        "Y" => Hand::Paper,
        "Z" => Hand::Scissors,
        s => return Err(format!("'{s}' is not a valid hand")),
    };
    Ok((left_hand, right_hand))
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum Hand {
    Rock,
    Paper,
    Scissors,
}

impl Hand {
    fn outcome_against(self, other: Hand) -> Outcome {
        match (self, other) {
            (Hand::Rock, Hand::Rock) => Outcome::Draw,
            (Hand::Rock, Hand::Paper) => Outcome::Loss,
            (Hand::Rock, Hand::Scissors) => Outcome::Win,
            (Hand::Paper, Hand::Rock) => Outcome::Win,
            (Hand::Paper, Hand::Paper) => Outcome::Draw,
            (Hand::Paper, Hand::Scissors) => Outcome::Loss,
            (Hand::Scissors, Hand::Rock) => Outcome::Loss,
            (Hand::Scissors, Hand::Paper) => Outcome::Win,
            (Hand::Scissors, Hand::Scissors) => Outcome::Draw,
        }
    }
    fn score_against(self, other: Hand) -> u32 {
        self.outcome_against(other).score()
            + match self {
                Hand::Rock => 1,
                Hand::Paper => 2,
                Hand::Scissors => 3,
            }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum Outcome {
    Loss,
    Draw,
    Win,
}

impl Outcome {
    fn score(self) -> u32 {
        match self {
            Outcome::Loss => 0,
            Outcome::Draw => 3,
            Outcome::Win => 6,
        }
    }
}

fn evaluate_part1(strat_guide: &[(Hand, Hand)]) -> u32 {
    strat_guide
        .iter()
        .map(|(left, right)| right.score_against(*left))
        .sum::<u32>()
}

#[cfg(test)]
mod test {
    use super::*;

    static EXAMPLE: &str = r#"A Y
B X
C Z"#;

    #[test]
    fn evaluate_part1_works_correctly() {
        // given
        let guide = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let score = evaluate_part1(&guide);

        // then
        assert_eq!(score, 15);
    }
}
