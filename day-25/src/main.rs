use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let numbers = parse_input(&content)?;
    let sum: i64 = numbers.iter().sum();

    println!(
        "The sum is {sum} in decimal and {} in snafu",
        fmt_snafu(sum)
    );

    Ok(())
}

fn parse_snafu(input: &str) -> Result<i64, String> {
    let mut number = 0;
    for c in input.chars() {
        if c == '-' {
            number = 5 * number - 1;
        } else if c == '=' {
            number = 5 * number - 2;
        } else if let Some(d) = c.to_digit(3) {
            number = 5 * number + d as i64;
        } else {
            return Err(format!(
                "unable to parse '{input}' as snafu number: '{c}' is not a valid digit."
            ));
        }
    }
    Ok(number)
}

fn parse_input(input: &str) -> Result<Vec<i64>, String> {
    input.lines().map(parse_snafu).collect()
}

fn fmt_snafu(num: i64) -> String {
    // let's take an easy but not very efficient approach
    let mut digits: Vec<char> = Vec::with_capacity(28);
    let sgn = num.signum();
    let mut num = num.unsigned_abs();
    while num != 0 {
        let d = num % 5;
        num /= 5;
        if let Some(c) = char::from_digit(d as u32, 3) {
            digits.push(c);
        } else if d == 3 {
            digits.push('=');
            num += 1;
        } else if d == 4 {
            digits.push('-');
            num += 1;
        }
    }
    if sgn < 0 {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"1=-0-2
12111
2=0=
21
2=01
111
20012
112
1=-1=
1-12
12
1=
122
"#;

    #[test]
    fn parse_input_works_for_example() {
        // when
        let result = parse_input(EXAMPLE);

        // then
        let numbers = result.expect("expected successful parsing");
        assert_eq!(
            &numbers,
            &[1747, 906, 198, 11, 201, 31, 1257, 32, 353, 107, 7, 3, 37]
        );
    }

    #[test]
    fn fmt_snafu_works_for_example() {
        // given
        let numbers = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let formatted: Vec<String> = numbers.iter().map(|n| fmt_snafu(*n)).collect();

        // then
        for (actual, wanted) in formatted.iter().zip(EXAMPLE.lines()) {
            assert_eq!(actual, wanted);
        }
    }
}
