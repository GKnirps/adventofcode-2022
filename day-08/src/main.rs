use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let grid = parse_grid(&content)?;

    let visibility = find_visible_trees(&grid);

    let visible_trees = count_visible_trees(&visibility);
    println!("There are {visible_trees} trees visible from the outside");

    if let Some(rating) = max_scenic_rating(&grid) {
        println!("The highest scenic rating is {rating}.");
    } else {
        println!("Where have all the trees gone?");
    }

    Ok(())
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Grid<T: Copy> {
    trees: Vec<T>,
    width: usize,
}

impl<T: Copy> Grid<T> {
    // will panic with out of bounds x and y
    fn at(&self, x: usize, y: usize) -> T {
        self.trees[x + y * self.width]
    }
    fn set(&mut self, x: usize, y: usize, value: T) {
        self.trees[x + y * self.width] = value;
    }

    fn height(&self) -> usize {
        self.trees.len() / self.width
    }
}

fn parse_grid(input: &str) -> Result<Grid<u8>, String> {
    // just assume every character is just one byte. Otherwise, parsing would fail anyways since we
    // expect only digits
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "empty grid input".to_owned())?
        .len();

    let trees: Vec<u8> = input
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|c| c as u8)
        .collect();

    if trees.len() % width != 0 {
        Err(format!(
            "assumed row length {}, but the grid size {} is not divisible by that",
            trees.len(),
            width
        ))
    } else {
        Ok(Grid { trees, width })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
struct VisibleFrom {
    top: bool,
    right: bool,
    bottom: bool,
    left: bool,
    tmax: u8,
    rmax: u8,
    bmax: u8,
    lmax: u8,
}

impl VisibleFrom {
    fn with_top(&self, visible: bool, max: u8) -> VisibleFrom {
        VisibleFrom {
            top: visible,
            tmax: max,
            ..*self
        }
    }
    fn with_right(&self, visible: bool, max: u8) -> VisibleFrom {
        VisibleFrom {
            right: visible,
            rmax: max,
            ..*self
        }
    }
    fn with_bottom(&self, visible: bool, max: u8) -> VisibleFrom {
        VisibleFrom {
            bottom: visible,
            bmax: max,
            ..*self
        }
    }
    fn with_left(&self, visible: bool, max: u8) -> VisibleFrom {
        VisibleFrom {
            left: visible,
            lmax: max,
            ..*self
        }
    }

    fn visible(&self) -> bool {
        self.top || self.bottom || self.right || self.left
    }
}

fn find_visible_trees(grid: &Grid<u8>) -> Grid<VisibleFrom> {
    let width = grid.width;
    let height = grid.height();
    let trees: Vec<VisibleFrom> = vec![VisibleFrom::default(); grid.trees.len()];
    let mut visible = Grid { trees, width };
    for i in 0..width {
        visible.set(i, 0, VisibleFrom::default().with_top(true, grid.at(i, 0)));
        visible.set(
            i,
            height - 1,
            VisibleFrom::default().with_bottom(true, grid.at(i, height - 1)),
        );
    }
    for i in 0..height {
        visible.set(0, i, visible.at(0, i).with_left(true, grid.at(0, i)));
        visible.set(
            width - 1,
            i,
            visible
                .at(width - 1, i)
                .with_right(true, grid.at(width - 1, i)),
        );
    }
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            visible.set(
                x,
                y,
                visible.at(x, y).with_top(
                    visible.at(x, y - 1).tmax < grid.at(x, y),
                    grid.at(x, y).max(visible.at(x, y - 1).tmax),
                ),
            );
            visible.set(
                x,
                height - 1 - y,
                visible.at(x, height - 1 - y).with_bottom(
                    visible.at(x, height - y).bmax < grid.at(x, height - 1 - y),
                    grid.at(x, height - 1 - y)
                        .max(visible.at(x, height - y).bmax),
                ),
            );
            visible.set(
                x,
                y,
                visible.at(x, y).with_left(
                    visible.at(x - 1, y).lmax < grid.at(x, y),
                    grid.at(x, y).max(visible.at(x - 1, y).lmax),
                ),
            );
            visible.set(
                width - 1 - x,
                y,
                visible.at(width - 1 - x, y).with_right(
                    visible.at(width - x, y).rmax < grid.at(width - 1 - x, y),
                    grid.at(width - 1 - x, y).max(visible.at(width - x, y).rmax),
                ),
            );
        }
    }
    visible
}

fn count_visible_trees(grid: &Grid<VisibleFrom>) -> usize {
    grid.trees.iter().filter(|tree| tree.visible()).count()
}

fn scenic_rating(grid: &Grid<u8>, x: usize, y: usize) -> u32 {
    let width = grid.width;
    let height = grid.height();
    let tree_height = grid.at(x, y);
    // screw this, let's just brute force it
    let mut ltrees: u32 = 0;
    for dx in 1..=x {
        ltrees += 1;
        if grid.at(x - dx, y) >= tree_height {
            break;
        }
    }
    let mut rtrees: u32 = 0;
    for dx in (x + 1)..width {
        rtrees += 1;
        if grid.at(dx, y) >= tree_height {
            break;
        }
    }
    let mut ttrees: u32 = 0;
    for dy in 1..=y {
        ttrees += 1;
        if grid.at(x, y - dy) >= tree_height {
            break;
        }
    }
    let mut btrees: u32 = 0;
    for dy in (y + 1)..height {
        btrees += 1;
        if grid.at(x, dy) >= tree_height {
            break;
        }
    }
    ltrees * rtrees * ttrees * btrees
}

fn max_scenic_rating(grid: &Grid<u8>) -> Option<u32> {
    (0..grid.width)
        .flat_map(|x| (0..grid.height()).map(move |y| (x, y)))
        .map(|(x, y)| scenic_rating(grid, x, y))
        .max()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"30373
25512
65332
33549
35390
"#;

    #[test]
    fn find_visible_trees_works_for_example() {
        // given
        let trees = parse_grid(EXAMPLE).expect("expected successful parsing");

        // when
        let visible = find_visible_trees(&trees);
        let count = count_visible_trees(&visible);

        // then
        assert_eq!(count, 21);
    }

    #[test]
    fn max_scenic_rating_works_for_example() {
        // given
        let trees = parse_grid(EXAMPLE).expect("expected successful parsing");

        // when
        let rating = max_scenic_rating(&trees);

        // then
        assert_eq!(rating, Some(8));
    }
}
