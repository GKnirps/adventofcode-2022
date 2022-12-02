use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let strategy_guide_part1 = parse_part1(&content)?;

    let strat_score = evaluate_part1(&strategy_guide_part1);
    println!("The score of the strategy guide is {strat_score}");

    let strategy_guide_part2 = parse_part2(&content)?;

    let part2_score = evaluate_part2(&strategy_guide_part2);
    println!("The score of the stragy using the correct interpretation is {part2_score}");

    Ok(())
}

fn parse_part1(content: &str) -> Result<Vec<(Hand, Hand)>, String> {
    content.lines().map(parse_line_part1).collect()
}

fn parse_line_part1(line: &str) -> Result<(Hand, Hand), String> {
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

fn parse_part2(content: &str) -> Result<Vec<(Hand, Outcome)>, String> {
    content.lines().map(parse_line_part2).collect()
}

fn parse_line_part2(line: &str) -> Result<(Hand, Outcome), String> {
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
        "X" => Outcome::Loss,
        "Y" => Outcome::Draw,
        "Z" => Outcome::Win,
        s => return Err(format!("'{s}' is not a valid outcome")),
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
    fn score(self) -> u32 {
        match self {
            Hand::Rock => 1,
            Hand::Paper => 2,
            Hand::Scissors => 3,
        }
    }
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
        self.outcome_against(other).score() + self.score()
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

fn hand_required(outcome: Outcome, opponent: Hand) -> Hand {
    match (outcome, opponent) {
        (Outcome::Loss, Hand::Rock) => Hand::Scissors,
        (Outcome::Draw, Hand::Rock) => Hand::Rock,
        (Outcome::Win, Hand::Rock) => Hand::Paper,
        (Outcome::Loss, Hand::Paper) => Hand::Rock,
        (Outcome::Draw, Hand::Paper) => Hand::Paper,
        (Outcome::Win, Hand::Paper) => Hand::Scissors,
        (Outcome::Loss, Hand::Scissors) => Hand::Paper,
        (Outcome::Draw, Hand::Scissors) => Hand::Scissors,
        (Outcome::Win, Hand::Scissors) => Hand::Rock,
    }
}

fn evaluate_part2(strat_guide: &[(Hand, Outcome)]) -> u32 {
    strat_guide
        .iter()
        .map(|(op, outcome)| outcome.score() + hand_required(*outcome, *op).score())
        .sum()
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
        let guide = parse_part1(EXAMPLE).expect("expected successful parsing");

        // when
        let score = evaluate_part1(&guide);

        // then
        assert_eq!(score, 15);
    }

    #[test]
    fn evaluate_part2_works_correctly() {
        // given
        let guide = parse_part2(EXAMPLE).expect("expected successful parsing");

        // when
        let score = evaluate_part2(&guide);

        // then
        assert_eq!(score, 12);
    }
}
