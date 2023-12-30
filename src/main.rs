use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{self, BufRead};


const MAX_STEPS: usize = 1000;


#[derive(Debug, PartialEq, Clone, Copy)]
enum MirrorType {
    Empty,
    PositiveMirror,  // as in f(x) = x
    NegativeMirror,  // as in f(x) = -x
    VerticalSplitter,
    HorizontalSplitter,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    North,
    South,
    East,
    West
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Beam {
    position: (usize, usize),
    direction: Direction
}

/// This is a variable size two dimensional field of mirrors.
#[derive(Debug)]
struct MirrorField {
    matrix: Vec<Vec<MirrorType>>

}

impl MirrorField {

    fn parse(lines: Vec<String>) -> Self {
        let matrix = lines
            .into_iter()
            .map(|line| {
                line
                    .chars()
                    .map(|c| match c {
                        '.' => MirrorType::Empty,
                        '/' => MirrorType::PositiveMirror,
                        '\\' => MirrorType::NegativeMirror,
                        '|' => MirrorType::VerticalSplitter,
                        '-' => MirrorType::HorizontalSplitter,
                        _ => panic!("Invalid character!")
                    })
                    .collect()
            })
            .collect();

        MirrorField { matrix }

    }

    fn load_from_file(path: &str) -> Self {
        Self::parse(
            io::BufReader::new(
                fs::File::open(path).expect("Could not open file!"))
                .lines()
                .flatten()
                .collect()
        )
    }

    fn to_str(&self) -> String {
        self.matrix
            .iter()
            .map(|row| {
                row
                    .iter()
                    .map(|c| match c {
                        MirrorType::Empty => '.',
                        MirrorType::HorizontalSplitter => '-',
                        MirrorType::VerticalSplitter => '|',
                        MirrorType::PositiveMirror => '/',
                        MirrorType::NegativeMirror => '\\'
                    })
                    .collect::<String>()
            })
            .reduce(|a, b| { a + "\n" + &b })
            .unwrap()
    }

    fn get_next_positions(&self, beam: Beam) -> Vec<Beam> {

        let mut next_positions: Vec<Beam> = vec![];
        let (x, y) = beam.position;
        
        match (beam.direction, self.matrix[y][x]) {
            // go north
            (Direction::East, MirrorType::PositiveMirror) |
            (Direction::West, MirrorType::NegativeMirror) | 
            (Direction::North, MirrorType::VerticalSplitter | MirrorType::Empty) => if y > 0 { next_positions.push(Beam { position: (x, y - 1), direction: Direction::North }) },
            
            // go south
            (Direction::East, MirrorType::NegativeMirror) |
            (Direction::West, MirrorType::PositiveMirror) |
            (Direction::South, MirrorType::VerticalSplitter | MirrorType::Empty) => if y < self.matrix.len() - 1 { next_positions.push(Beam { position: (x, y + 1), direction: Direction::South }) },

            // go west
            (Direction::South, MirrorType::PositiveMirror) |
            (Direction::North, MirrorType::NegativeMirror) |
            (Direction::West, MirrorType::HorizontalSplitter | MirrorType::Empty) => if x > 0 { next_positions.push(Beam { position: (x - 1, y), direction: Direction::West }) },

            // go east
            (Direction::South, MirrorType::NegativeMirror) |
            (Direction::North, MirrorType::PositiveMirror) |
            (Direction::East, MirrorType::HorizontalSplitter | MirrorType::Empty) => if x < self.matrix[0].len() - 1 { next_positions.push(Beam { position: (x + 1, y), direction: Direction::East }) },

            // go west and east
            (Direction::North | Direction::South, MirrorType::HorizontalSplitter) => {
                if x > 0 { next_positions.push(Beam { position: (x - 1, y), direction: Direction::West }) }
                if x < self.matrix[0].len() - 1 { next_positions.push(Beam { position: (x + 1, y), direction: Direction::East }) }
            }

            // go north and south
            (Direction::East | Direction::West, MirrorType::VerticalSplitter) => {
                if y > 0 { next_positions.push(Beam { position: (x, y - 1), direction: Direction::North }) }
                if y < self.matrix.len() - 1 { next_positions.push(Beam { position: (x, y + 1), direction: Direction::South }) }
            }

        }

        next_positions

    }

    fn get_power(&self, beams: &[Beam]) -> usize {
        let mut powered: BTreeSet<(usize, usize)> = BTreeSet::new();

        let mut current_beams: Vec<Beam> = beams.into();
        let mut previous_beams: BTreeSet<Beam> = BTreeSet::new();
        let mut steps: usize = 0;
        while steps < MAX_STEPS && !current_beams.is_empty() {
            powered.extend(current_beams.iter().map(|b| b.position));
            for beam in current_beams.iter() { previous_beams.insert(beam.clone()); }
            current_beams = current_beams
                .into_iter()
                .flat_map(|b| self.get_next_positions(b))
                .filter(|np| !previous_beams.contains(np))  // don't add looping beams
                .collect();
            steps += 1;
        }
        if !current_beams.is_empty() { println!("WARN: unfinished beams!") };

        powered.len()
    }

}


fn main() {
    let path = env::args().nth(1).expect("Missing required parameter path!");

    let field = MirrorField::load_from_file(&path);
    println!("Power going east from 0, 0: {}", field.get_power(&[Beam {direction: Direction::East, position: (0, 0)}]));

    let max_power_from_west: usize = (0..field.matrix.len())
        .map(|y| field.get_power(&[Beam {direction: Direction::East, position: (0, y)}]))
        .max()
        .unwrap();
    println!("Max power from western border: {}", max_power_from_west);

    let max_power_from_east: usize = (0..field.matrix.len())
        .map(|y| field.get_power(&[Beam {direction: Direction::West, position: (field.matrix[0].len() - 1, y)}]))
        .max()
        .unwrap();
    println!("Max power from eastern border: {}", max_power_from_east);

    let max_power_from_north: usize = (0..field.matrix[0].len())
        .map(|x| field.get_power(&[Beam {direction: Direction::South, position: (x, 0)}]))
        .max()
        .unwrap();
    println!("Max power from northern border: {}", max_power_from_north);

    let max_power_from_south: usize = (0..field.matrix.len())
        .map(|x| field.get_power(&[Beam {direction: Direction::North, position: (x, field.matrix.len() - 1)}]))
        .max()
        .unwrap();
    println!("Max power from eastern border: {}", max_power_from_south);
}
