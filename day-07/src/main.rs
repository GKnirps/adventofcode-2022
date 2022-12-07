use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let dir_tree = construct_directory_tree(&content)?;

    let small_dir_sum = sum_small_dirs(&dir_tree);
    println!("The sum of all small dir sizes is {small_dir_sum}");

    Ok(())
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum FType<'a> {
    Dir(Vec<FsNode<'a>>),
    File,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct FsNode<'a> {
    size: u64,
    name: &'a str,
    ftype: FType<'a>,
}

fn construct_directory_tree(input: &str) -> Result<FsNode, String> {
    let mut lines = input.lines();
    if lines.next() != Some("$ cd /") {
        return Err("Expected first line to beo '$ cd /'".to_owned());
    }
    construct_subtree("/", &mut lines)
}

fn construct_subtree<'a, I: Iterator<Item = &'a str>>(
    dir_name: &'a str,
    lines: &mut I,
) -> Result<FsNode<'a>, String> {
    let mut last_command: &str = "";
    let mut size: u64 = 0;
    let mut children: Vec<FsNode> = Vec::with_capacity(16);
    while let Some(line) = lines.next() {
        if let Some(command) = line.strip_prefix("$ ") {
            last_command = command;
            if command == "cd .." {
                return Ok(FsNode {
                    size,
                    name: dir_name,
                    ftype: FType::Dir(children),
                });
            } else if let Some(subdir_name) = command.strip_prefix("cd ") {
                let subdir = construct_subtree(subdir_name, lines)?;
                size += subdir.size;
                children.push(subdir);
            }
        } else if last_command == "ls" {
            let (info, name) = line
                .split_once(' ')
                .ok_or_else(|| format!("Unable to parse directory entry '{line}'"))?;
            // we ignore directories in the listing because we just implicitly list them when
            // we change to them
            if info != "dir" {
                let file_size: u64 = info
                    .parse::<u64>()
                    .map_err(|_| format!("Unable to parse file size in '{line}'"))?;
                size += file_size;
                children.push(FsNode {
                    size: file_size,
                    name,
                    ftype: FType::File,
                });
            }
        } else {
            return Err(format!("unexpected line '{line}'"));
        }
    }
    Ok(FsNode {
        size,
        name: dir_name,
        ftype: FType::Dir(children),
    })
}

fn sum_small_dirs(node: &FsNode) -> u64 {
    if let FType::Dir(children) = &node.ftype {
        children.iter().map(sum_small_dirs).sum::<u64>()
            + if node.size <= 100000 { node.size } else { 0 }
    } else {
        0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k
"#;

    #[test]
    fn sum_small_dirs_works_correctly() {
        // given
        let tree = construct_directory_tree(EXAMPLE).expect("Expected successfull tree building");

        // when
        let sum = sum_small_dirs(&tree);

        // then
        assert_eq!(sum, 95437);
    }
}
