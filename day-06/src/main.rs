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
        println!("No start of packet marker found.");
    }

    if let Some(offset) = start_of_msg_offset(&content) {
        println!("Start of message marker after: {offset}");
    } else {
        println!("no start of message marker found.");
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

fn start_of_msg_offset(input: &str) -> Option<usize> {
    input
        .as_bytes()
        .windows(14)
        .enumerate()
        .filter(|(_, w)| {
            let mut seen: [bool; 256] = [false; 256];
            for c in *w {
                if seen[*c as usize] {
                    return false;
                }
                seen[*c as usize] = true;
            }
            true
        })
        .map(|(i, _)| i + 14)
        .next()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_of_packet_offset_works_for_examples() {
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

    #[test]
    fn start_of_msg_offset_works_for_examples() {
        assert_eq!(
            start_of_msg_offset("mjqjpqmgbljsphdztnvjfqwrcgsmlb"),
            Some(19)
        );
        assert_eq!(
            start_of_msg_offset("bvwbjplbgvbhsrlpgdmjqwftvncz"),
            Some(23)
        );
        assert_eq!(
            start_of_msg_offset("nppdvjthqldpwncqszvftbrmjlhg"),
            Some(23)
        );
        assert_eq!(
            start_of_msg_offset("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"),
            Some(29)
        );
        assert_eq!(
            start_of_msg_offset("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"),
            Some(26)
        );
    }
}
