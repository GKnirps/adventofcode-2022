use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let ciphertext = parse_input(&content)?;

    let plaintext = mix(&ciphertext);
    if let Some(csum) = grove_coordinate_sum(&plaintext) {
        println!("The sum of the grove coordinates is {csum}.");
    } else {
        println!("Unable to find grove coordinate sum in the plaintext.");
    }

    Ok(())
}

fn parse_input(input: &str) -> Result<Vec<isize>, String> {
    input
        .lines()
        .map(|line| {
            line.parse::<isize>()
                .map_err(|e| format!("Unable to parse line '{line}': {e}"))
        })
        .collect()
}

fn grove_coordinate_sum(plaintext: &[isize]) -> Option<isize> {
    let zero = plaintext
        .iter()
        .enumerate()
        .filter(|(_, v)| **v == 0)
        .map(|(i, _)| i)
        .next()?;
    Some(
        plaintext[(zero + 1000) % plaintext.len()]
            + plaintext[(zero + 2000) % plaintext.len()]
            + plaintext[(zero + 3000) % plaintext.len()],
    )
}

fn mix(ciphertext: &[isize]) -> Vec<isize> {
    let mut plaintext: Vec<(isize, bool)> = ciphertext.iter().map(|v| (*v, false)).collect();
    // let's first try a primitive approach and see how far this gets us
    let mut i: usize = 0;
    while i < plaintext.len() {
        let (value, moved) = plaintext[i];
        if moved {
            i += 1;
            continue;
        }

        let target =
            (i as isize + value - 1).rem_euclid(ciphertext.len() as isize - 1) as usize + 1;

        plaintext[i] = (value, true);
        if target > i {
            for j in i..target {
                plaintext.swap(j, j + 1);
            }
        } else {
            for j in 0..(i - target) {
                plaintext.swap(i - j, i - j - 1);
            }
            i += 1;
        }
    }
    plaintext.iter().map(|(v, _)| v).copied().collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mix_works_for_example() {
        // given
        let ciphertext: &[isize] = &[1, 2, -3, 3, -2, 0, 4];

        // when
        let plaintext = mix(ciphertext);

        // then
        assert_eq!(&plaintext, &[1, 2, -3, 4, 0, 3, -2]);
    }

    #[test]
    fn grove_coordinate_sum_works_for_example() {
        // given
        let plaintext: &[isize] = &[1, 2, -3, 4, 0, 3, -2];

        // when
        let csum = grove_coordinate_sum(plaintext);

        // then
        assert_eq!(csum, Some(3));
    }
}
