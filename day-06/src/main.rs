use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    if let Some(offset) = start_of_packet_offset(&content) {
        println!("Start of packet marker after: {offset}");
    } else {
        println!("No start of packet marker found");
    }

    Ok(())
}

fn start_of_packet_offset(input: &str) -> Option<usize> {
    // yes, there are probably more efficient ways to do this. This one should be sufficient though
    // and is still O(N)
    input
        .as_bytes()
        .windows(4)
        .enumerate()
        .filter(|(_, w)| {
            w[0] != w[1]
                && w[0] != w[2]
                && w[0] != w[3]
                && w[1] != w[2]
                && w[1] != w[3]
                && w[2] != w[3]
        })
        .map(|(i, _)| i + 4)
        .next()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_of_pacjet_offset_works_for_examples() {
        assert_eq!(
            start_of_packet_offset("mjqjpqmgbljsphdztnvjfqwrcgsmlb"),
            Some(7)
        );
        assert_eq!(
            start_of_packet_offset("bvwbjplbgvbhsrlpgdmjqwftvncz"),
            Some(5)
        );
        assert_eq!(
            start_of_packet_offset("nppdvjthqldpwncqszvftbrmjlhg"),
            Some(6)
        );
        assert_eq!(
            start_of_packet_offset("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"),
            Some(10)
        );
        assert_eq!(
            start_of_packet_offset("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"),
            Some(11)
        );
    }
}
