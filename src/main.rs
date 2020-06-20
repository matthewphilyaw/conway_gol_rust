use std::collections::HashSet;

type CellCoordinate = u64;
type Cell = (CellCoordinate, CellCoordinate);

fn start_value(val: CellCoordinate) -> CellCoordinate {
    if val == CellCoordinate::MIN {
        return val;
    }

    val - 1
}

fn end_value(val: CellCoordinate) -> CellCoordinate {
    if val == CellCoordinate::MAX {
        return val;
    }

    val + 1
}

#[derive(Debug)]
struct NeighborIterator {
    first_col: CellCoordinate,
    current_cell: Cell,
    end_neighbor: Cell,
    current_neighbor: Option<Cell>
}


impl NeighborIterator  {
    fn new((row, col): Cell) -> NeighborIterator {
        NeighborIterator {
            first_col: start_value(col),
            current_cell: (row, col),
            end_neighbor: (end_value(row), end_value(col)),
            current_neighbor: Some((start_value(row), start_value(col)))
        }
    }
}

fn get_next_neighbor(iter: &NeighborIterator) -> Option<Cell> {
    let current_neighbor = match iter.current_neighbor {
        Some(neighbor) => neighbor,
        None => return None // early return out of function
    };

    if current_neighbor == iter.end_neighbor {
        return None;
    }

    let (_, end_col) = iter.end_neighbor;
    let (cur_row, cur_col) = current_neighbor;

    let neighbor;
    if cur_col == end_col {
        neighbor = (cur_row + 1, iter.first_col);
    } else {
        neighbor = (cur_row, cur_col + 1);
    }

    Some(neighbor)
}

/// Iterates through all the possible neighbors excluding the current cell.
///
/// This iterator only emits the viable cells. Any cell that is outside the bounds of the coordinate
/// data type are not generated.
///
/// If CellCoordinate is an unsigned type then cell coordinates emitted would be between 0 and the max of that type
/// If CellCoordinate is a signed type then cells coordinates would between (including) min and max of that type
impl Iterator for NeighborIterator {
    type Item = Cell;
    fn next(&mut self) -> Option<Cell> {
        // First neighbor is set on initialization so lets capture it
        let current_neighbor = self.current_neighbor;

        // Compute next neighbor used next go around
        self.current_neighbor = get_next_neighbor(self);

        // If the current neighbor is the current cell skip it and ask for the next
        // neighbor again
        if self.current_neighbor == Some(self.current_cell) {
            self.current_neighbor = get_next_neighbor(self);
        }

        // return the current neighbor
        current_neighbor
    }
}

/// In normal cases the alive_count + dead_neighbors length should total 7
/// exception being around corners and edges where
#[derive(Debug)]
struct NeighborStatus {
    alive_count: u8,
    dead_neighbors: Vec<Cell>
}

struct GOLGenerationIterator {
    current_gen: HashSet<Cell>
}

impl GOLGenerationIterator {
    fn new(seed: Vec<Cell>) -> GOLGenerationIterator {
        let gen_zero = seed.into_iter().collect();
        GOLGenerationIterator {
            current_gen: gen_zero
        }
    }
}

impl Iterator for GOLGenerationIterator {
    type Item = HashSet<Cell>;
    fn next(&mut self) -> Option<HashSet<Cell>> {
        let next_gen = compute_next_gen(&self.current_gen);
        let current_gen = std::mem::replace(&mut self.current_gen, next_gen);

        Some(current_gen)
    }
}

/// Only applicable to dead cells - simplified logic and not collecting
/// other dead cells around it.
fn bring_cell_back_to_life(cell: Cell, current_gen: &HashSet<Cell>) -> bool {
    let mut alive = 0;
    for neighbor in NeighborIterator::new(cell) {
        if current_gen.contains(&neighbor) {
            alive = alive + 1;
        }

    }

    alive == 3
}

/// Used for live cells to determine the number alive cells surrounding it and
/// collect the valid dead cells that will be used later on to see if those dead cells
/// will be reborn.
fn get_neighbors_status(cell: Cell, current_gen: &HashSet<Cell>) -> NeighborStatus {
    let mut neighbor_status = NeighborStatus {
        alive_count: 0,
        dead_neighbors: vec![]
    };

    for neighbor in NeighborIterator::new(cell) {
       if current_gen.contains(&neighbor) {
            neighbor_status.alive_count += 1;
       } else {
            neighbor_status.dead_neighbors.push(neighbor);
       }
    }

    return neighbor_status;
}

fn compute_next_gen(current_gen: &HashSet<Cell>) -> HashSet<Cell> {
    let mut next = HashSet::new();

    for cell in current_gen.iter() {
        let neighbors = get_neighbors_status(*cell, current_gen);

        let alive = neighbors.alive_count;
        if alive == 2 || alive  == 3 {
            next.insert(*cell);
        }

        // These neighbors need to be checked to see if they can be brought back to life
        // Only ones that are adjacent to a live neighbor in some form could be brought back
        // and they are not in the generation set.
        //
        // So capturing these and processing them as they are found.
        //
        // worst case all neighbors are dead and this executes 7 times and curious if how rust
        // optimizes this vs me writing the for loop by hand.
        neighbors.dead_neighbors
            .iter()
            .filter(|p| bring_cell_back_to_life(**p, current_gen))
            .for_each(|p| {
                next.insert(*p);
            });
    }

    return next;
}


fn main() {
    let seed: Vec<Cell> = vec![
        (1,3),
        (2,2),
        (2,3),
        (2,4),
        (3,2),
        (3,4),
        (4,3),
    ];

    let iter = GOLGenerationIterator::new(seed);

    // gen zero is the seed state so you have to take 2 to get the first generation after
    // the seed.
    for gen in iter.take(10) {
        let row_end: CellCoordinate = 20;
        let col_end: CellCoordinate = 20;

        for row in 0..row_end {
            for col in 0..col_end {
                if gen.contains(&(row, col)) {
                    print!("x  ");
                } else {
                    print!("-  ");
                }
            }
            println!();
        }

        println!();
    }
}
