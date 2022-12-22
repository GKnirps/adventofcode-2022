use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (map, path) = parse_input(&content)?;

    let password = walk_path(&map, &path);
    println!("The password is {password}");

    let cube = map_to_cube(&map)?;
    let cube_password = walk_cube_path(&cube, &path);
    println!("The password from the cube is {cube_password}");

    Ok(())
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct MapBlock {
    offset: usize,
    width: usize,
    grid: Vec<bool>,
}

impl MapBlock {
    fn height(&self) -> usize {
        self.grid.len() / self.width
    }

    // will panic if y is out of bounds
    fn get_xwrap(&self, x: usize, y: usize) -> bool {
        self.grid[y * self.width + (x % self.width)]
    }

    fn global_x(&self, x: usize) -> usize {
        self.offset + x
    }

    fn local_x(&self, global_x: usize) -> Option<usize> {
        if global_x >= self.offset && global_x < self.offset + self.width {
            Some(global_x - self.offset)
        } else {
            None
        }
    }

    fn clone_square(&self, xoffset: usize, yoffset: usize, edge_length: usize) -> Vec<bool> {
        let mut square: Vec<bool> = Vec::with_capacity(edge_length * edge_length);
        for y in yoffset..(yoffset + edge_length) {
            for x in xoffset..(xoffset + edge_length) {
                square.push(self.grid[y * self.width + x]);
            }
        }
        square
    }
}

type Map = Vec<MapBlock>;

fn parse_map(input: &str) -> Result<Map, String> {
    let mut map: Map = Vec::with_capacity(16);
    let mut lines = input.lines();

    let line = lines.next().ok_or_else(|| "map is empty".to_owned())?;
    let mut offset = line.bytes().take_while(|c| *c == b' ').count();
    let mut width = line.as_bytes().len() - offset;
    if width == 0 {
        return Err("map line has zero width".to_owned());
    }
    let mut grid: Vec<bool> = Vec::with_capacity(input.len());
    // ok, we're making the assumption that the input is well-formed and only contains valid
    // characters
    grid.extend(line.as_bytes()[offset..].iter().map(|c| *c != b'#'));

    for line in lines {
        let new_offset = line.bytes().take_while(|c| *c == b' ').count();
        let new_width = line.as_bytes().len() - new_offset;
        if offset != new_offset || width != new_width {
            map.push(MapBlock {
                offset,
                width,
                grid,
            });
            offset = new_offset;
            width = new_width;
            grid = Vec::with_capacity(input.len());
        }
        if width == 0 {
            return Err("map line has zero width".to_owned());
        }
        // ok, we're making the assumption that the input is well-formed and only contains valid
        // characters
        grid.extend(line.as_bytes()[offset..].iter().map(|c| *c != b'#'));
    }
    map.push(MapBlock {
        offset,
        width,
        grid,
    });

    Ok(map)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Turn {
    R,
    L,
    Straight(u32),
}

fn parse_path(input: &str) -> Result<Vec<Turn>, String> {
    let mut path: Vec<Turn> = Vec::with_capacity(input.len());
    let mut number = 0;
    for c in input.trim().chars() {
        if c == 'R' {
            if number != 0 {
                path.push(Turn::Straight(number));
                number = 0;
            }
            path.push(Turn::R);
        } else if c == 'L' {
            if number != 0 {
                path.push(Turn::Straight(number));
                number = 0;
            }
            path.push(Turn::L);
        } else if let Some(digit) = c.to_digit(10) {
            number = number * 10 + digit;
        } else {
            return Err(format!("Unexpected character '{c}' in path definition."));
        }
    }
    if number != 0 {
        path.push(Turn::Straight(number));
    }
    Ok(path)
}

fn parse_input(input: &str) -> Result<(Map, Vec<Turn>), String> {
    let (raw_map, raw_path) = input
        .split_once("\n\n")
        .ok_or_else(|| "unable to split path from map".to_owned())?;
    let map = parse_map(raw_map)?;
    let path = parse_path(raw_path)?;

    Ok((map, path))
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl Dir {
    fn rot_clockwise(self, r: u8) -> Self {
        let mut rotated = self;
        for _ in 0..(r % 4) {
            rotated = match rotated {
                Dir::Up => Dir::Right,
                Dir::Left => Dir::Up,
                Dir::Down => Dir::Left,
                Dir::Right => Dir::Down,
            };
        }
        rotated
    }
}

fn walk_path(map: &Map, path: &[Turn]) -> usize {
    if map.is_empty() {
        return 1004;
    }
    let mut dir = Dir::Right;
    let mut block: usize = 0;
    let mut x: usize = 0;
    let mut y: usize = 0;

    for turn in path {
        match turn {
            Turn::L => {
                dir = match dir {
                    Dir::Up => Dir::Left,
                    Dir::Right => Dir::Up,
                    Dir::Down => Dir::Right,
                    Dir::Left => Dir::Down,
                };
            }
            Turn::R => {
                dir = match dir {
                    Dir::Up => Dir::Right,
                    Dir::Right => Dir::Down,
                    Dir::Down => Dir::Left,
                    Dir::Left => Dir::Up,
                };
            }
            Turn::Straight(n) => {
                for _ in 0..*n {
                    match dir {
                        Dir::Up => {
                            if y > 0 {
                                if map[block].get_xwrap(x, y - 1) {
                                    y -= 1;
                                }
                            } else {
                                let global_x = map[block].global_x(x);
                                for di in 1..=map.len() {
                                    let i = (block as isize - di as isize)
                                        .rem_euclid(map.len() as isize)
                                        as usize;
                                    if let Some(local_x) = map[i].local_x(global_x) {
                                        if map[i].get_xwrap(local_x, map[i].height() - 1) {
                                            block = i;
                                            x = local_x;
                                            y = map[block].height() - 1;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Dir::Right => {
                            if map[block].get_xwrap(x + 1, y) {
                                x = (x + 1) % map[block].width;
                            }
                        }
                        Dir::Down => {
                            if y + 1 < map[block].height() {
                                if map[block].get_xwrap(x, y + 1) {
                                    y += 1;
                                }
                            } else {
                                let global_x = map[block].global_x(x);
                                for di in 1..=map.len() {
                                    if let Some(local_x) =
                                        map[(block + di) % map.len()].local_x(global_x)
                                    {
                                        if map[(block + di) % map.len()].get_xwrap(local_x, 0) {
                                            block = (block + di) % map.len();
                                            x = local_x;
                                            y = 0;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Dir::Left => {
                            let next_x = if x == 0 { map[block].width - 1 } else { x - 1 };
                            if map[block].get_xwrap(next_x, y) {
                                x = next_x;
                            }
                        }
                    }
                }
            }
        }
    }

    1000 * (y + 1 + map[0..block].iter().map(|b| b.height()).sum::<usize>())
        + 4 * (map[block].global_x(x) + 1)
        + match dir {
            Dir::Right => 0,
            Dir::Down => 1,
            Dir::Left => 2,
            Dir::Up => 3,
        }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct CubeSide {
    grid: Vec<bool>,
    top: CubeLink,
    right: CubeLink,
    bottom: CubeLink,
    left: CubeLink,
    grid_coordinates: (usize, usize),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Cube {
    sides: [CubeSide; 6],
    edge_length: usize,
}

impl Cube {
    fn get(&self, side: usize, x: usize, y: usize) -> bool {
        assert!(x < self.edge_length);
        assert!(y < self.edge_length);
        self.sides[side].grid[y * self.edge_length + x]
    }
}

// (linked side, 90° rotations counterclockwise)
type CubeLink = (usize, u8);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
struct ProtoCubeSide {
    grid: Vec<bool>,
    top: Option<CubeLink>,
    right: Option<CubeLink>,
    bottom: Option<CubeLink>,
    left: Option<CubeLink>,
    grid_coordinates: (isize, isize),
}

impl ProtoCubeSide {
    fn into_cube_side(self) -> Result<CubeSide, String> {
        match (self.top, self.right, self.bottom, self.left) {
            (Some(top), Some(right), Some(bottom), Some(left)) => Ok(CubeSide {
                grid: self.grid,
                top,
                right,
                bottom,
                left,
                grid_coordinates: (
                    self.grid_coordinates.0 as usize,
                    self.grid_coordinates.1 as usize,
                ),
            }),
            _ => Err("Unable to create cube side, not all neighbours are defined".to_owned()),
        }
    }
}

// the cube is more readable if each side is on one line, trust me
#[rustfmt::skip]
static NORM_CUBE: Cube = Cube {
    sides: [
        CubeSide{grid: vec![], top: (4, 0), right: (5, 3), bottom: (1, 0), left: (3, 1), grid_coordinates: (1, 0)},
        CubeSide{grid: vec![], top: (0, 0), right: (5, 2), bottom: (2, 0), left: (3, 2), grid_coordinates: (1, 1)},
        CubeSide{grid: vec![], top: (1, 0), right: (5, 1), bottom: (4, 0), left: (3, 3), grid_coordinates: (1, 2)},
        CubeSide{grid: vec![], top: (2, 1), right: (4, 0), bottom: (0, 3), left: (1, 2), grid_coordinates: (0, 3)},
        CubeSide{grid: vec![], top: (2, 0), right: (5, 0), bottom: (0, 0), left: (3, 0), grid_coordinates: (1, 3)},
        CubeSide{grid: vec![], top: (2, 3), right: (1, 2), bottom: (0, 1), left: (4, 0), grid_coordinates: (2, 3)},
    ],
    edge_length: 1,
};

fn map_to_cube(map: &Map) -> Result<Cube, String> {
    let edge_length = map
        .iter()
        .flat_map(|block| [block.width, block.height()])
        .min()
        .ok_or_else(|| "expected to find a edge length, but map size is 0".to_owned())?;

    // sanity check: what we have actually amounts to six cube sides
    let n_cube_sides = map
        .iter()
        .filter(|block| block.width % edge_length == 0 && block.height() % edge_length == 0)
        .map(|block| (block.width / edge_length) * (block.height() / edge_length))
        .sum::<usize>();
    if n_cube_sides != 6 {
        return Err(format!(
            "map has {n_cube_sides} potential sides, need exactly 6"
        ));
    }

    let mut proto_cube: [ProtoCubeSide; 6] = [
        ProtoCubeSide::default(),
        ProtoCubeSide::default(),
        ProtoCubeSide::default(),
        ProtoCubeSide::default(),
        ProtoCubeSide::default(),
        ProtoCubeSide::default(),
    ];
    let mut side_index = 0;
    let mut y_offset = 0;
    // step one: list the sides
    for block in map {
        for y in 0..(block.height() / edge_length) {
            for x in 0..(block.width / edge_length) {
                proto_cube[side_index] = ProtoCubeSide {
                    grid: block.clone_square(x * edge_length, y * edge_length, edge_length),
                    top: None,
                    right: None,
                    bottom: None,
                    left: None,
                    grid_coordinates: (
                        (block.offset / edge_length + x) as isize,
                        y_offset as isize,
                    ),
                };
                side_index += 1;
            }
            y_offset += 1;
        }
    }

    // step two: find out how the sides are linked
    let mut side_to_norm_side: [Option<(usize, u8)>; 6] = [None; 6];
    side_to_norm_side[0] = Some((0, 0));
    let mut stack: Vec<(usize, usize, u8)> = Vec::with_capacity(6);
    let mut seen = [false; 6];
    stack.push((0, 0, 0));
    while let Some((i, norm_cube_i, norm_cube_rot)) = stack.pop() {
        if seen[i] {
            continue;
        }
        seen[i] = true;
        let (grid_x, grid_y) = proto_cube[i].grid_coordinates;
        for side_i in (0..proto_cube.len()).filter(|j| *j != i) {
            let (sgrid_x, sgrid_y) = proto_cube[side_i].grid_coordinates;
            if grid_x + 1 == sgrid_x && grid_y == sgrid_y {
                // neighbour is right
                proto_cube[i].right = Some((side_i, 0));
                proto_cube[side_i].left = Some((i, 0));
                let (nb_norm_cube_i, nb_norm_cube_rot) = get_link_rot(
                    &NORM_CUBE.sides[norm_cube_i],
                    Dir::Right,
                    (norm_cube_rot) % 4,
                );
                let rot_to_norm = (norm_cube_rot + (nb_norm_cube_rot) % 4) % 4;
                stack.push((side_i, nb_norm_cube_i, rot_to_norm));
                side_to_norm_side[side_i] = Some((nb_norm_cube_i, rot_to_norm));
            } else if grid_x - 1 == sgrid_x && grid_y == sgrid_y {
                // neighbour is left
                proto_cube[i].left = Some((side_i, 0));
                proto_cube[side_i].right = Some((i, 0));
                let (nb_norm_cube_i, nb_norm_cube_rot) = get_link_rot(
                    &NORM_CUBE.sides[norm_cube_i],
                    Dir::Left,
                    (norm_cube_rot) % 4,
                );
                let rot_to_norm = (norm_cube_rot + (nb_norm_cube_rot) % 4) % 4;
                stack.push((side_i, nb_norm_cube_i, rot_to_norm));
                side_to_norm_side[side_i] = Some((nb_norm_cube_i, rot_to_norm));
            } else if grid_x == sgrid_x && grid_y + 1 == sgrid_y {
                // neighbour is down
                proto_cube[i].bottom = Some((side_i, 0));
                proto_cube[side_i].top = Some((i, 0));
                let (nb_norm_cube_i, nb_norm_cube_rot) = get_link_rot(
                    &NORM_CUBE.sides[norm_cube_i],
                    Dir::Down,
                    (norm_cube_rot) % 4,
                );
                let rot_to_norm = (norm_cube_rot + (nb_norm_cube_rot) % 4) % 4;
                stack.push((side_i, nb_norm_cube_i, rot_to_norm));
                side_to_norm_side[side_i] = Some((nb_norm_cube_i, rot_to_norm));
            } else if grid_x == sgrid_x && grid_y - 1 == sgrid_y {
                // neighbour is up
                proto_cube[i].top = Some((side_i, 0));
                proto_cube[side_i].bottom = Some((i, 0));
                let (nb_norm_cube_i, nb_norm_cube_rot) =
                    get_link_rot(&NORM_CUBE.sides[norm_cube_i], Dir::Up, (norm_cube_rot) % 4);
                let rot_to_norm = (norm_cube_rot + (nb_norm_cube_rot) % 4) % 4;
                stack.push((side_i, nb_norm_cube_i, rot_to_norm));
                side_to_norm_side[side_i] = Some((nb_norm_cube_i, rot_to_norm));
            }
        }
    }
    if side_to_norm_side.iter().any(|s| s.is_none()) {
        return Err("expected all sides to have been visited".to_owned());
    }
    let side_to_norm_side = side_to_norm_side.map(|side| side.unwrap());
    for i in 0..proto_cube.len() {
        if proto_cube[i].top.is_none() {
            let (norm_i, norm_rot) = side_to_norm_side[i];
            let (norm_neighbour_i, norm_neighbour_rot) =
                get_link_rot(&NORM_CUBE.sides[norm_i], Dir::Up, norm_rot);
            let (neighbour_i, neighbour_rot) = side_to_norm_side
                .iter()
                .enumerate()
                .filter(|(_, (i, _))| *i == norm_neighbour_i)
                .map(|(i, (_, rot))| (i, (4 - rot) % 4))
                .next()
                .ok_or_else(|| "side to norm array is not bijective".to_owned())?;
            proto_cube[i].top = Some((
                neighbour_i,
                (norm_rot + norm_neighbour_rot + neighbour_rot) % 4,
            ));
        }
        if proto_cube[i].right.is_none() {
            let (norm_i, norm_rot) = side_to_norm_side[i];
            let (norm_neighbour_i, norm_neighbour_rot) =
                get_link_rot(&NORM_CUBE.sides[norm_i], Dir::Right, norm_rot);
            let (neighbour_i, neighbour_rot) = side_to_norm_side
                .iter()
                .enumerate()
                .filter(|(_, (i, _))| *i == norm_neighbour_i)
                .map(|(i, (_, rot))| (i, (4 - rot) % 4))
                .next()
                .ok_or_else(|| "side to norm array is not bijective".to_owned())?;
            proto_cube[i].right = Some((
                neighbour_i,
                (norm_rot + norm_neighbour_rot + neighbour_rot) % 4,
            ));
        }
        if proto_cube[i].left.is_none() {
            let (norm_i, norm_rot) = side_to_norm_side[i];
            let (norm_neighbour_i, norm_neighbour_rot) =
                get_link_rot(&NORM_CUBE.sides[norm_i], Dir::Left, norm_rot);
            let (neighbour_i, neighbour_rot) = side_to_norm_side
                .iter()
                .enumerate()
                .filter(|(_, (i, _))| *i == norm_neighbour_i)
                .map(|(i, (_, rot))| (i, (4 - rot) % 4))
                .next()
                .ok_or_else(|| "side to norm array is not bijective".to_owned())?;
            proto_cube[i].left = Some((
                neighbour_i,
                (norm_rot + norm_neighbour_rot + neighbour_rot) % 4,
            ));
        }
        if proto_cube[i].bottom.is_none() {
            let (norm_i, norm_rot) = side_to_norm_side[i];
            let (norm_neighbour_i, norm_neighbour_rot) =
                get_link_rot(&NORM_CUBE.sides[norm_i], Dir::Down, norm_rot);
            let (neighbour_i, neighbour_rot) = side_to_norm_side
                .iter()
                .enumerate()
                .filter(|(_, (i, _))| *i == norm_neighbour_i)
                .map(|(i, (_, rot))| (i, (4 - rot) % 4))
                .next()
                .ok_or_else(|| "side to norm array is not bijective".to_owned())?;
            proto_cube[i].bottom = Some((
                neighbour_i,
                (norm_rot + norm_neighbour_rot + neighbour_rot) % 4,
            ));
        }
    }

    // step three ???

    if proto_cube.iter().any(|pc| {
        pc.top.is_none() || pc.right.is_none() || pc.bottom.is_none() || pc.left.is_none()
    }) {
        return Err("At least one cube side is missing a connection after folding".to_owned());
    }

    // step four: profit
    let sides = proto_cube.map(|side| {
        side.into_cube_side()
            .expect("expected to have all side edge links")
    });
    let cube = Cube { sides, edge_length };

    sanity_check_cube(&cube)?;
    Ok(cube)
}

// another sanitiy check (just to see if I implemented everything correctly, because honestly,
// I only have a vague idea of what I'm doing): Check if we can loop around from any side in
// any direction
fn sanity_check_cube(cube: &Cube) -> Result<(), String> {
    for i in 0..cube.sides.len() {
        let mut top_i = i;
        let mut top_rot = 0;
        let mut right_i = i;
        let mut right_rot = 0;
        let mut bottom_i = i;
        let mut bottom_rot = 0;
        let mut left_i = i;
        let mut left_rot = 0;
        for _ in 0..4 {
            let (next, rot) = get_link_rot(&cube.sides[top_i], Dir::Up, top_rot);
            top_i = next;
            top_rot = (top_rot + rot) % 4;

            let (next, rot) = get_link_rot(&cube.sides[right_i], Dir::Right, right_rot);
            right_i = next;
            right_rot = (right_rot + rot) % 4;

            let (next, rot) = get_link_rot(&cube.sides[bottom_i], Dir::Down, bottom_rot);
            bottom_i = next;
            bottom_rot = (bottom_rot + rot) % 4;

            let (next, rot) = get_link_rot(&cube.sides[left_i], Dir::Left, left_rot);
            left_i = next;
            left_rot = (left_rot + rot) % 4;
        }

        if top_i != i && top_rot != 0 {
            return Err(format!("Sanity check for cube side {i} failed: top roundtrip result: {top_i}, rotation: {top_rot}"));
        }
        if right_i != i && right_rot != 0 {
            return Err(format!("Sanity check for cube side {i} failed: right roundtrip result: {right_i}, rotation: {right_rot}"));
        }
        if bottom_i != i && bottom_rot != 0 {
            return Err(format!("Sanity check for cube side {i} failed: bottom roundtrip result: {bottom_i}, rotation: {bottom_rot}"));
        }
        if left_i != i && left_rot != 0 {
            return Err(format!("Sanity check for cube side {i} failed: left roundtrip result: {left_i}, rotation: {left_rot}"));
        }
    }
    Ok(())
}

// a different kind of link rot than the one that is common on the www
fn get_link_rot(side: &CubeSide, dir: Dir, rot: u8) -> (usize, u8) {
    match dir.rot_clockwise(rot) {
        Dir::Up => side.top,
        Dir::Right => side.right,
        Dir::Down => side.bottom,
        Dir::Left => side.left,
    }
}

fn walk_cube_path(cube: &Cube, path: &[Turn]) -> usize {
    let mut dir = Dir::Right;
    let mut side: usize = 0;
    let mut x: usize = 0;
    let mut y: usize = 0;

    let edge_length = cube.edge_length;

    for turn in path {
        match turn {
            Turn::L => {
                dir = match dir {
                    Dir::Up => Dir::Left,
                    Dir::Right => Dir::Up,
                    Dir::Down => Dir::Right,
                    Dir::Left => Dir::Down,
                };
            }
            Turn::R => {
                dir = match dir {
                    Dir::Up => Dir::Right,
                    Dir::Right => Dir::Down,
                    Dir::Down => Dir::Left,
                    Dir::Left => Dir::Up,
                };
            }
            Turn::Straight(n) => {
                for _ in 0..*n {
                    match dir {
                        Dir::Up => {
                            if y > 0 {
                                if cube.get(side, x, y - 1) {
                                    y -= 1;
                                }
                            } else {
                                let (top_side, rotation) = cube.sides[side].top;
                                let (new_x, new_y, new_dir) = match rotation % 4 {
                                    0 => (x, edge_length - 1, Dir::Up),
                                    1 => (0, x, Dir::Right),
                                    2 => (edge_length - 1 - x, 0, Dir::Down),
                                    3 => (edge_length - 1, edge_length - 1 - x, Dir::Left),
                                    _ => panic!("unmatched rotation"),
                                };
                                if cube.get(top_side, new_x, new_y) {
                                    x = new_x;
                                    y = new_y;
                                    dir = new_dir;
                                    side = top_side;
                                }
                            }
                        }
                        Dir::Right => {
                            if x + 1 < cube.edge_length {
                                if cube.get(side, x + 1, y) {
                                    x += 1;
                                }
                            } else {
                                let (right_side, rotation) = cube.sides[side].right;
                                let (new_x, new_y, new_dir) = match rotation % 4 {
                                    0 => (0, y, Dir::Right),
                                    1 => (edge_length - 1 - y, 0, Dir::Down),
                                    2 => (edge_length - 1, edge_length - 1 - y, Dir::Left),
                                    3 => (y, edge_length - 1, Dir::Up),
                                    _ => panic!("unmatched rotation"),
                                };
                                if cube.get(right_side, new_x, new_y) {
                                    x = new_x;
                                    y = new_y;
                                    dir = new_dir;
                                    side = right_side;
                                }
                            }
                        }
                        Dir::Down => {
                            if y + 1 < cube.edge_length {
                                if cube.get(side, x, y + 1) {
                                    y += 1;
                                }
                            } else {
                                let (bottom_side, rotation) = cube.sides[side].bottom;
                                let (new_x, new_y, new_dir) = match rotation % 4 {
                                    0 => (x, 0, Dir::Down),
                                    1 => (edge_length - 1, x, Dir::Left),
                                    2 => (edge_length - 1 - x, edge_length - 1, Dir::Up),
                                    3 => (0, edge_length - 1 - x, Dir::Right),
                                    _ => panic!("unmatched rotation"),
                                };
                                if cube.get(bottom_side, new_x, new_y) {
                                    x = new_x;
                                    y = new_y;
                                    dir = new_dir;
                                    side = bottom_side;
                                }
                            }
                        }
                        Dir::Left => {
                            if x > 0 {
                                if cube.get(side, x - 1, y) {
                                    x -= 1;
                                }
                            } else {
                                let (left_side, rotation) = cube.sides[side].left;
                                let (new_x, new_y, new_dir) = match rotation % 4 {
                                    0 => (edge_length - 1, y, Dir::Left),
                                    1 => (edge_length - 1 - y, edge_length - 1, Dir::Up),
                                    2 => (0, edge_length - 1 - y, Dir::Right),
                                    3 => (y, 0, Dir::Down),
                                    _ => panic!("unmatched rotation"),
                                };
                                if cube.get(left_side, new_x, new_y) {
                                    x = new_x;
                                    y = new_y;
                                    dir = new_dir;
                                    side = left_side;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let global_x = cube.sides[side].grid_coordinates.0 * cube.edge_length + x + 1;
    let global_y = cube.sides[side].grid_coordinates.1 * cube.edge_length + y + 1;
    global_y * 1000
        + global_x * 4
        + match dir {
            Dir::Right => 0,
            Dir::Down => 1,
            Dir::Left => 2,
            Dir::Up => 3,
        }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn walk_path_works_for_example() {
        // given
        let (map, path) = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let key = walk_path(&map, &path);

        // then
        assert_eq!(key, 6032);
    }

    #[test]
    fn map_to_cube_works_for_example() {
        // given
        let (map, _) = parse_input(EXAMPLE).expect("expected successful parsing");

        // when
        let result = map_to_cube(&map);

        // then
        let cube = result.expect("expected successful cube folding");
        assert_eq!(cube.edge_length, 4);
        assert_eq!(cube.sides[0].top, (1, 2));
        assert_eq!(cube.sides[0].right, (5, 2));
        assert_eq!(cube.sides[0].left, (2, 3));
        assert_eq!(cube.sides[0].bottom, (3, 0));

        assert_eq!(cube.sides[1].top, (0, 2));
        assert_eq!(cube.sides[1].right, (2, 0));
        assert_eq!(cube.sides[1].left, (5, 1));
        assert_eq!(cube.sides[1].bottom, (4, 2));

        assert_eq!(cube.sides[2].top, (0, 1));
        assert_eq!(cube.sides[2].right, (3, 0));
        assert_eq!(cube.sides[2].left, (1, 0));
        assert_eq!(cube.sides[2].bottom, (4, 3));

        assert_eq!(cube.sides[3].top, (0, 0));
        assert_eq!(cube.sides[3].right, (5, 1));
        assert_eq!(cube.sides[3].left, (2, 0));
        assert_eq!(cube.sides[3].bottom, (4, 0));

        assert_eq!(cube.sides[4].top, (3, 0));
        assert_eq!(cube.sides[4].right, (5, 0));
        assert_eq!(cube.sides[4].left, (2, 1));
        assert_eq!(cube.sides[4].bottom, (1, 2));

        assert_eq!(cube.sides[5].top, (3, 3));
        assert_eq!(cube.sides[5].right, (0, 2));
        assert_eq!(cube.sides[5].left, (4, 0));
        assert_eq!(cube.sides[5].bottom, (1, 3));
    }

    #[test]
    fn map_to_cube_works_for_norm_cube() {
        // given
        let input = " #\n #\n #\n###\n\n42\n";
        let (map, _) = parse_input(input).expect("expected successful parsing");

        // when
        let result = map_to_cube(&map);

        // then
        let mut cube = result.expect("expected successful cube folding");
        for side in &mut cube.sides {
            side.grid = vec![];
        }
        assert_eq!(cube, NORM_CUBE);
    }

    #[test]
    fn norm_cube_passes_sanity_check() {
        // given
        let cube = &NORM_CUBE;

        // when
        let result = sanity_check_cube(&cube);

        // then
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn walk_cube_path_works_for_example() {
        // given
        let (map, path) = parse_input(EXAMPLE).expect("expected successful parsing");
        let cube = map_to_cube(&map).expect("expected successful transformation from map to cube");

        // when
        let pw = walk_cube_path(&cube, &path);

        // then
        assert_eq!(pw, 5031);
    }

    const EXAMPLE: &str = r#"        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5
"#;
}
