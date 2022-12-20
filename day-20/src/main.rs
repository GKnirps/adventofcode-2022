use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let ciphertext = parse_input(&content)?;

    let plaintext = mix(&ciphertext, 1);
    if let Some(csum) = grove_coordinate_sum(&plaintext) {
        println!("The sum of the grove coordinates is {csum}.");
    } else {
        println!("Unable to find grove coordinate sum in the plaintext.");
    }

    let ciphertext_with_key: Vec<isize> = ciphertext.iter().map(|v| *v * DECRYPTION_KEY).collect();
    let plaintext = mix(&ciphertext_with_key, 10);
    if let Some(csum) = grove_coordinate_sum(&plaintext) {
        println!("The sum of the _actual_ grove coordinates is {csum}.");
    } else {
        println!("Unable to find grove coordinate sum in the plaintext.");
    }

    Ok(())
}

const DECRYPTION_KEY: isize = 811589153;

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

fn mix(ciphertext: &[isize], iterations: usize) -> Vec<isize> {
    let mut positions: Vec<usize> = (0..ciphertext.len()).collect();
    let mut plaintext: Vec<(isize, usize)> = ciphertext
        .iter()
        .enumerate()
        .map(|(i, v)| (*v, i))
        .collect();

    for _ in 0..iterations {
        for pos_i in 0..positions.len() {
            let i = positions[pos_i];
            let (value, _) = plaintext[i];
            if value == 0 {
                continue;
            }

            let target =
                (i as isize + value - 1).rem_euclid(ciphertext.len() as isize - 1) as usize + 1;

            if target > i {
                for j in i..target {
                    let (_, p1) = plaintext[j];
                    let (_, p2) = plaintext[j + 1];
                    positions[p1] = j + 1;
                    positions[p2] = j;
                    plaintext.swap(j, j + 1);
                }
            } else {
                for j in 0..(i - target) {
                    let (_, p1) = plaintext[i - j];
                    let (_, p2) = plaintext[i - j - 1];
                    positions[p1] = i - j - 1;
                    positions[p2] = i - j;
                    plaintext.swap(i - j, i - j - 1);
                }
            }
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
        let plaintext = mix(ciphertext, 1);

        // then
        assert_eq!(&plaintext, &[1, 2, -3, 4, 0, 3, -2]);
    }

    #[test]
    fn mix_works_for_simplified_double_overflowing_example() {
        // given
        let ciphertext: &[isize] = &[0, 0, 7, 0, 0];

        // when
        let plaintext = mix(ciphertext, 1);

        // then
        assert_eq!(&plaintext, &[0, 7, 0, 0, 0]);
    }

    fn mix_works_for_simplified_negative_double_overflowing_example() {
        // given
        let ciphertext: &[isize] = &[0, 0, -7, 0, 0];

        // when
        let plaintext = mix(ciphertext, 1);

        // then
        assert_eq!(&plaintext, &[0, 0, 0, -7, 0]);
    }

    #[test]
    fn mix_works_for_part2_example() {
        // given
        let ciphertext: &[isize] = &[
            811589153,
            1623178306,
            -2434767459,
            2434767459,
            -1623178306,
            0,
            3246356612,
        ];

        // when
        let plaintext = mix(ciphertext, 10);

        // then
        assert_eq!(
            &plaintext,
            &[
                0,
                -2434767459,
                1623178306,
                3246356612,
                -1623178306,
                2434767459,
                811589153
            ]
        );
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
