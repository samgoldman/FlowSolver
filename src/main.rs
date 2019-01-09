#![allow(dead_code)]

use std::cmp::max;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

const ALPHA_UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

// Note the modules probably shouldn't be modules, or they should be in different files.
// I just liked how the code folds in this IDE, and I'm lazy, and it works.

// All of the neighbor constants
mod neighbor {
    pub const TOP: usize = 0;
    pub const BOTTOM: usize = 1;
    pub const LEFT: usize = 2;
    pub const RIGHT: usize = 3;
    pub const TOP_LEFT: usize = 2;   // TOP_LEFT and TOP_RIGHT share array locations with LEFT and RIGHT respectively
    pub const TOP_RIGHT: usize = 3;
    pub const BOTTOM_LEFT: usize = 4;
    pub const BOTTOM_RIGHT: usize = 5;

    pub const SQUARE_NEIGHBORS: [usize; 4] = [TOP, BOTTOM, LEFT, RIGHT];
    pub const HEX_NEIGHBORS: [usize; 6] = [TOP, BOTTOM, TOP_LEFT, TOP_RIGHT, BOTTOM_LEFT, BOTTOM_RIGHT];
}

// Structures and implementations related to flows
mod flow {
    use super::cell;
    use super::puzzle;

    #[derive(Debug, Clone)]
    pub struct Flow {
        // TODO: remove pub modifier
        pub id: usize,
        // TODO: remove pub modifier
        pub endpoints: [Option<cell::CellId>; 2],
        // TODO: remove pub modifier
        pub complete: bool,
        // TODO: remove pub modifier
        pub letter: char,
    }

    impl Flow {
        // Update the endpoint at the given index to the given cellID
        // endpoint should be 0 or 1
        pub fn update_endpoint(&mut self, endpoint: usize, cell_id: Option<cell::CellId>) {
            self.endpoints[endpoint] = cell_id;
        }

        // This doesn't actually do anything, because any actual call to it, from any useful location, makes the borrow checker very upset, and we don't want that
        // TODO See about fixing that ^. Maybe keep the logic elsewhere, like it is now, and have this take in a bool
        pub fn update_completeness(&mut self, puzzle: &puzzle::Puzzle) {
            self.complete = puzzle.get_cell(self.endpoints[0].unwrap()).unwrap().is_neighbor(&self.endpoints[1].unwrap());
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct FlowId {
        pub index: usize,
    }
}

// Structures and implementations related to cells
mod cell {
    use super::flow;
    use super::neighbor;
    use super::puzzle;

    #[derive(Debug, Clone, Copy)]
    pub struct Cell {
        // TODO: remove pub modifier
        pub is_endpoint: bool,
        // TODO: remove pub modifier
        // If only it's as easy as deleting it
        pub flow_id_1: Option<flow::FlowId>,
        // TODO: remove pub modifier
        pub flow_id_2: Option<flow::FlowId>,
        // TODO: remove pub modifier
        pub is_bridge: bool,
        // TODO: remove pub modifier
        pub neighbors: [Option<CellId>; 6],
        // TODO: remove pub modifier
        pub is_hex: bool,
    }

    impl Cell {
        // Update the given neighbor
        pub fn set_neighbor(&mut self, neighbor: usize, cell: CellId) {
            self.neighbors[neighbor] = Some(cell);
        }

        // Given a CellId, check if that cell in a neighbor of this cell
        pub fn is_neighbor(&self, other: &CellId) -> bool {
            let is_neighbor =
                if self.is_hex { // Must account for the differing neighbor schemes
                    let mut is_neighbor = false;
                    // Loop through each possible neighbor
                    for n_index in 0..neighbor::HEX_NEIGHBORS.len() {
                        if self.neighbors[neighbor::HEX_NEIGHBORS[n_index]].is_some() && self.neighbors[neighbor::HEX_NEIGHBORS[n_index]].unwrap().index == other.index {
                            is_neighbor = true;
                        }
                    };
                    is_neighbor
                } else { // Same as above essentially
                    let mut is_neighbor = false;
                    for n_index in 0..neighbor::SQUARE_NEIGHBORS.len() {
                        if self.neighbors[neighbor::SQUARE_NEIGHBORS[n_index]].is_some() && self.neighbors[neighbor::SQUARE_NEIGHBORS[n_index]].unwrap().index == other.index {
                            is_neighbor = true;
                        }
                    };
                    is_neighbor
                };

            is_neighbor
        }

        pub fn is_occupied(&self, calling_neighbor: Option<usize>) -> bool {
            // If this cell isn't a bridge, the calling neighbor doesn't matter
            // all that matters if the flow is occupied in some manner (which should only be flow_id_1)
            // Same goes if the calling neighbor is None
            if !self.is_bridge || calling_neighbor.is_none() {
                self.flow_id_1.is_some() || self.flow_id_2.is_some()
            } else {
                // If the cell is a bridge, say this cell is open only if the appropriate direction
                // is open: TOP/BOTTOM is flow_id_1; LEFT/RIGHT if flow_id_2
                let cn = calling_neighbor.unwrap();
                (cn == neighbor::TOP || cn == neighbor::BOTTOM) && self.flow_id_1.is_some()
                    || (cn == neighbor::LEFT || cn == neighbor::RIGHT) && self.flow_id_2.is_some()
            }
        }

        // Return the number of neighboring cells that are not occupied
        pub fn num_open_neighbors(&self, puzzle: &puzzle::Puzzle) -> usize {
            let num_neighbors =
                // Depending on the cell type, loop through each possible neighbor and determine if it's open
                if self.is_hex {
                    let mut count = 0;
                    for n_index in 0..neighbor::HEX_NEIGHBORS.len() {
                        let n_id = self.neighbors[neighbor::HEX_NEIGHBORS[n_index]];
                        if n_id.is_some() {
                            if !puzzle.get_cell(n_id.unwrap()).unwrap().is_occupied(Some(neighbor::HEX_NEIGHBORS[n_index])) {
                                count += 1;
                            }
                        }
                    };
                    count
                } else {
                    let mut count = 0;
                    for n_index in 0..neighbor::SQUARE_NEIGHBORS.len() {
                        let n_id = self.neighbors[neighbor::SQUARE_NEIGHBORS[n_index]];
                        if n_id.is_some() {
                            if !puzzle.get_cell(n_id.unwrap()).unwrap().is_occupied(Some(neighbor::SQUARE_NEIGHBORS[n_index])) {
                                count += 1;
                            }
                        }
                    };
                    count
                };

            num_neighbors
        }

        // Just the number of neighbors
        pub fn num_neighbors(&self, _puzzle: &puzzle::Puzzle) -> usize {
            let num_neighbors =
                // Depending on the cell type, loop through each neighbor and if it is some, count it
                if self.is_hex {
                    let mut count = 0;
                    for n_index in 0..neighbor::HEX_NEIGHBORS.len() {
                        let n = self.neighbors[neighbor::HEX_NEIGHBORS[n_index]];
                        if n.is_some() {
                            count += 1;
                        }
                    };
                    count
                } else {
                    let mut count = 0;
                    for n_index in 0..neighbor::SQUARE_NEIGHBORS.len() {
                        let n = self.neighbors[neighbor::SQUARE_NEIGHBORS[n_index]];
                        if n.is_some() {
                            count += 1;
                        }
                    };
                    count
                };

            num_neighbors
        }
    }

    #[derive(Debug, Default, Copy, Clone)]
    pub struct CellId {
        pub index: usize,
    }
}

// Structures and implementations related to the puzzle
mod puzzle {
    use super::cell;
    use super::flow;

    #[derive(Debug, Clone)]
    pub struct Puzzle {
        cells: Vec<cell::Cell>,
        // TODO: remove pub modifier
        pub flows: Vec<flow::Flow>,
        // TODO: remove pub modifier
        pub is_hex: bool,
    }

    impl Puzzle {
        // Create a new puzzle
        pub fn new(is_hex: bool) -> Puzzle {
            Puzzle { cells: Vec::new(), flows: Vec::new(), is_hex }
        }

        // Crate a new cell
        pub fn new_cell(&mut self, is_endpoint: bool, flow_id_1: Option<flow::FlowId>, is_bridge: bool, is_hex: bool) -> cell::CellId {
            let next_index = self.num_cells();
            self.cells.push(cell::Cell {
                is_endpoint,
                flow_id_1,
                flow_id_2: None,
                is_bridge,
                neighbors: [None; 6],
                is_hex
            });

            cell::CellId { index: next_index }
        }

        pub fn num_cells(&self) -> usize {
            self.cells.len()
        }

        pub fn get_cell(&self, id: cell::CellId) -> Option<&cell::Cell> {
            self.cells.get(id.index)
        }

        pub fn get_cell_mut(&mut self, id: cell::CellId) -> Option<&mut cell::Cell> {
            self.cells.get_mut(id.index)
        }

        pub fn new_flow(&mut self, letter: char) -> flow::FlowId {
            let next_index = self.num_flows();
            self.flows.push(flow::Flow {
                id: next_index,
                endpoints: [None; 2],
                complete: false,
                letter,
            });

            flow::FlowId { index: next_index }
        }

        pub fn num_flows(&self) -> usize {
            self.flows.len()
        }

        pub fn get_flow(&self, id: flow::FlowId) -> Option<&flow::Flow> {
            self.flows.get(id.index)
        }

        pub fn get_flow_mut(&mut self, id: flow::FlowId) -> Option<&mut flow::Flow> {
            self.flows.get_mut(id.index)
        }

        pub fn num_complete(&self) -> usize {
            let mut num = 0;
            for flow in self.flows.iter() {
                // TODO try to move this logic to flow
                // If the flow's endpoints are neighbors, the flow is completed
                if self.get_cell(flow.endpoints[0].unwrap()).unwrap().is_neighbor(&flow.endpoints[1].unwrap()) {
                    num += 1;
                }
            }
            num
        }

        pub fn is_complete(&self) -> bool {
            // Are all flows completed?
            // TODO all cells should be full too, including both bridge slots
            self.num_complete() == self.num_flows()
        }

        // Return a vector of all endpoints for flows that are not complete
        pub fn get_endpoints_for_incomplete_flows(&self) -> Vec<cell::CellId> {
            let mut endpoints = Vec::new();

            for flow in self.flows.iter() {
                // TODO move incompletion logic to flow (again)
                // If the flow is incomplete, push its two endpoints onto the vector
                if !self.get_cell(flow.endpoints[0].unwrap()).unwrap().is_neighbor(&flow.endpoints[1].unwrap()) {
                    endpoints.push(flow.endpoints[0].unwrap());
                    endpoints.push(flow.endpoints[1].unwrap());
                }
            }
            endpoints
        }
    }
}

// Wrapper "node" for puzzle states. Somewhat of a tree
mod puzzle_state {
    use super::puzzle;
    use super::cell;
    use super::neighbor;
    use super::flow;

    #[derive(Debug, Clone)]
    pub struct PuzzleState {
        // TODO: remove pub modifier
        pub puzzle: puzzle::Puzzle,
        // TODO: remove pub modifier
        pub children: Vec<PuzzleState>,
    }

    impl PuzzleState{
        // Create a child PuzzleState for each possible move for the next endpoint to extend
        pub fn create_children(&mut self) {
            let endpoint_id = self.get_endpoint_to_extend();
            let endpoint_cell = self.puzzle.get_cell(endpoint_id).unwrap();
            let flow_id = endpoint_cell.flow_id_1.unwrap();
            let flow = self.puzzle.get_flow(flow_id).unwrap();

            let endpoint_index =
                if flow.endpoints[0].unwrap().index == endpoint_id.index {
                    0
                } else {
                    1
                };

            if self.puzzle.is_hex {
                for n_index in 0..neighbor::HEX_NEIGHBORS.len() {
                    if endpoint_cell.neighbors[neighbor::HEX_NEIGHBORS[n_index]].is_none() {
                        continue;
                    }
                    let n_id = endpoint_cell.neighbors[neighbor::HEX_NEIGHBORS[n_index]].unwrap();
                    // If the neighbor is not occupied, create a child and update the child so that cell is occupied with the flow of the endpoint to extend
                    if !self.puzzle.get_cell(n_id).unwrap().is_occupied(Some(neighbor::HEX_NEIGHBORS[n_index])) {
                        if endpoint_cell.neighbors[neighbor::HEX_NEIGHBORS[n_index]].is_none() {
                            continue;
                        }
                        let mut child = self.clone();

                        // Update the child
                        let mut cell_to_move_to = child.puzzle.get_cell_mut(n_id).unwrap();
                        cell_to_move_to.flow_id_1 = Some(flow::FlowId {index: flow_id.index});
                        cell_to_move_to.is_endpoint = true;

                        let child_flow = child.puzzle.get_flow_mut(flow_id).unwrap();
                        child_flow.update_endpoint(endpoint_index, Some(cell::CellId{index: n_id.index}));

                        // Update this with the new child
                        self.children.push(child);
                    }
                };
            } else {
                for n_index in 0..neighbor::SQUARE_NEIGHBORS.len() {
                    if endpoint_cell.neighbors[neighbor::SQUARE_NEIGHBORS[n_index]].is_none() {
                        continue;
                    }
                    let n_id = endpoint_cell.neighbors[neighbor::SQUARE_NEIGHBORS[n_index]].unwrap();
                    // TODO Update logic to account for bridges
                    // Must move flow into appropriate flow flot in the cell, and then to the next cell
                    // Well, the second part might not be necessary, but it probably helps
                    if !self.puzzle.get_cell(n_id).unwrap().is_occupied(Some(neighbor::SQUARE_NEIGHBORS[n_index])) {
                        let mut child = self.clone();
                        let mut cell_to_move_to = child.puzzle.get_cell_mut(n_id).unwrap();
                        cell_to_move_to.flow_id_1 = Some(flow::FlowId {index: flow_id.index});
                        cell_to_move_to.is_endpoint = true;

                        let child_flow = child.puzzle.get_flow_mut(flow_id).unwrap();
                        child_flow.update_endpoint(endpoint_index, Some(cell::CellId{index: n_id.index}));
                        self.children.push(child);
                    }
                };
            }
        }

        // Not currently used (I think)
        pub fn num_possible_children (self) -> usize {
            let endpoint_id = self.get_endpoint_to_extend();
            let endpoint_cell = self.puzzle.get_cell(endpoint_id).unwrap();
            endpoint_cell.num_open_neighbors(&self.puzzle)
        }

        pub fn is_complete(&self) -> bool {
            self.puzzle.is_complete()
        }

        // Basically, find the endpoint with the fewest opoen neighbors (possibilities) and return that one
        fn get_endpoint_to_extend(&self) -> cell::CellId {
            let possible_endpoints = self.puzzle.get_endpoints_for_incomplete_flows();
            let mut min_open = 7;
            let mut min_open_cell_id = cell::CellId{index: 999};
            for cell_id in possible_endpoints {
                let cell = self.puzzle.get_cell(cell_id);
                if cell.unwrap().num_open_neighbors(&self.puzzle) < min_open {
                    min_open = cell.unwrap().num_open_neighbors(&self.puzzle);
                    min_open_cell_id = cell_id;
                }
            }
            min_open_cell_id
        }
    }
}

// Begin solving the puzzle located in the given file
// Includes parsing the puzzle, creating the initial puzzle state, and recursivly solving the puzzle
// Does most of the work
fn solve_puzzle(filename: &str) {
    let path = Path::new(filename);

    // Verify valid extension
    // In the future, perhaps parse image files here
    let extension = path.extension().and_then(OsStr::to_str);
    if extension != Some("txt") {
        println!("At this time, puzzle files must be text files (.txt)!");
        return;
    }

    let display = path.display();
    println!("Solving the puzzle located at: {}\n", display);

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                           why.description()),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut input = String::new();
    match file.read_to_string(&mut input) {
        Err(why) => panic!("couldn't read {}: {}", display,
                           why.description()),
        Ok(_) => print!("{}:\n{}\n\n", display, input),
    }

    // Split the input based on lines
    let mut split_input: Vec<&str> = input.split("\n").collect();
    split_input.remove(0);

    // Check if the puzzle is a HEX puzzle
    let puzzle_type: String = input.chars().skip(0).take(input.find("\n").unwrap()-1).collect();
    let is_hex = puzzle_type == "HEX";

    // The initial puzzle
    let mut puzzle: puzzle::Puzzle = puzzle::Puzzle::new(is_hex);

    // Used to map locations of cells via the cell's id (as a usize, for reasons beyond my comprehension)
    // Used to create neighbor relationships later
    let mut cell_map: HashMap<String, usize> = HashMap::new();

    let mut row = 0; // Track the current row
    // Track the max height and width of the puzzle (note: this is because not all puzzles are squares and rectangles)
    let mut max_cell_row = 0;
    let mut max_cell_col = 0;
    // Nested for loops: iterate through each character in the input board
    // If it a cell character (capital letter, period, or asterisk, create a cell for it), update/create a flow
    for line in &split_input {
        let mut col = 0; // Track the current column
        for c in line.chars() {
            // Check if the character is a cell character
            if ALPHA_UPPER.contains(c) || c == '.' || c == '*' {
                let is_bridge = c == '*'; // Asterisks are bridges
                let is_endpoint = ALPHA_UPPER.contains(c);

                // Create the new cell
                let cell_id: cell::CellId = puzzle.new_cell(is_endpoint, None, is_bridge, is_hex);

                // Create a key for the map from the coordinates of the cell, and insert it into the map with the new cell id
                let mut key: String = String::from(col.to_string());
                key.push_str(row.to_string().as_ref());
                cell_map.insert(key, cell_id.index);

                // Update the max size variables
                max_cell_row = max(max_cell_row, row);
                max_cell_col = max(max_cell_col, col);

                // If the cell is an endpoint, either create a new flow if needed or update an existing one
                if ALPHA_UPPER.contains(c) {
                    let mut flow_id_1 = None;
                    let mut flow_exists = false;
                    let mut count = 0;

                    // Check if a flow with the current letter already exists
                    // If it does, update hte appropriate values
                    for flow in puzzle.flows.iter_mut() {
                        if flow.letter == c {
                            flow_exists = true;
                            flow.update_endpoint(1, Some(cell_id));
                            flow_id_1 = Some(flow::FlowId {index: count});
                        }
                        count += 1;
                    }

                    // If the flow doesn't exist, create a new one
                    if !flow_exists {
                        let flow_id = puzzle.new_flow(c);
                        let flow = puzzle.get_flow_mut(flow_id).unwrap();
                        flow_id_1 = Some(flow_id);
                        flow.update_endpoint(0, Some(cell_id));
                    }

                    // Update the new cell with the appropriate flow
                    puzzle.get_cell_mut(cell_id).unwrap().flow_id_1 = flow_id_1;
                }
            }
            col += 1;
        }
        row += 1;
    }

    row = 0;
    let mut neighbors = 0; // Count the number of neighbor relationships
    // Again, loop through all characters in the board configuration
    // This time, look for neighbor characters: '-', '|', '/', '\'
    // When one is found, update the appropriate cells
    for line in &split_input {
        let mut col = 0;
        let mut count = 0;
        for c in line.chars() {
            // If the character is a neighbor character, proceed
            if c == '-' || c == '/' || c == '\\' || c == '|' {
                // Get the coordinates of the two neighbors, and the neighbor relationship for each
                let (col1, row1, col2, row2, neighbor1, neighbor2) =
                    if c == '-' {
                        // On standard boards, the '-' character represents a LEFT-RIGHT relationship
                        // On hex boards, the character alternatingly refers to the BOTTOM_RIGHT-TOP_LEFT and TOP_RIGHT-BOTTOM_LEFT relationshiops
                        if is_hex {
                            // Handle alternating relationship
                            count+=1;
                            if count % 2 == 1 {
                                (col - 1, row, col + 1, row, neighbor::BOTTOM_RIGHT, neighbor::TOP_LEFT)
                            } else {
                                (col - 1, row, col + 1, row, neighbor::TOP_RIGHT, neighbor::BOTTOM_LEFT)
                            }
                        } else {
                            // If the current column is greater than the max width, this is a warped relationship. The neighbor is in the first column.
                            if col > max_cell_col {
                                (col - 1, row, 0, row, neighbor::RIGHT, neighbor::LEFT)
                            } else {
                                (col - 1, row, col + 1, row, neighbor::RIGHT, neighbor::LEFT)
                            }
                        }
                    } else if c == '/' {
                        (col - 1, row + 1, col + 1, row - 1, neighbor::TOP_RIGHT, neighbor::BOTTOM_LEFT)
                    } else if c == '\\' {
                        (col - 1, row - 1, col + 1, row + 1, neighbor::BOTTOM_RIGHT, neighbor::TOP_LEFT)
                    } else {
                        // This is the '|' relationship: TOP-BOTTOM
                        // If the row is greater than the max heigh, this is warped relationship. The neighbor is in the first row.
                        if row > max_cell_row {
                            (col, row - 1, col, 0, neighbor::BOTTOM, neighbor::TOP)
                        } else {
                            (col, row - 1, col, row + 1, neighbor::BOTTOM, neighbor::TOP)
                        }
                    };

                // Recreate the map keys for the two neighbors
                neighbors += 1;
                let mut key1: String = String::from(col1.to_string());
                key1.push_str(row1.to_string().as_ref());
                let mut key2: String = String::from(col2.to_string());
                key2.push_str(row2.to_string().as_ref());

                // Update the cells with information regarding their newly found neighbor
                puzzle.get_cell_mut(cell::CellId { index: *cell_map.get(&key1).unwrap() }).unwrap().set_neighbor(neighbor1, cell::CellId { index: *cell_map.get(&key2).unwrap() });
                puzzle.get_cell_mut(cell::CellId { index: *cell_map.get(&key2).unwrap() }).unwrap().set_neighbor(neighbor2, cell::CellId { index: *cell_map.get(&key1).unwrap() });
            }
            col += 1;
        }
        row += 1;
    }

    // Status info
    println!("Number of cells: {}", puzzle.num_cells());
    println!("Number of flows: {}", puzzle.num_flows());
    println!("Number of neighbors: {}\n\n", neighbors);

    // Create the initial puzzle state with no children
    let mut ps = puzzle_state::PuzzleState {puzzle: puzzle, children: Vec::new()};
    // Solve it. Just like that. It's done!
    let res = recursively_solve(&mut ps, 1);

    // Resplit the input, because why not?
    // Because Rust, that's why
    let mut split_input2: Vec<&str> = input.split("\n").collect();
    split_input2.remove(0);
    let mut cell = 0;
    // Loop over it again, printing everything out
    // Unless it is a cell character. Then replace it with the appropriate letter from the solved puzzle
    for line in &split_input2 {
        for c in line.chars() {
            if ALPHA_UPPER.contains(c) || c == '.' || c == '*' {
                // TODO: figure out how to display bridge solutions
                print!("{}", res.unwrap().puzzle.get_flow(res.unwrap().puzzle.get_cell(cell::CellId{index:cell}).unwrap().flow_id_1.unwrap()).unwrap().letter);
                cell +=1;
            } else {
                print!("{}", c);
            }
        }
        print!("\n");
    }

    // `file` goes out of scope, and then gets closed
}

// Actually solve the puzzle, unlike that other, slacker function
// Given a puzzle state, generate children and recursively solve these children until
// a complete puzzle state is found
fn recursively_solve(ps: &mut puzzle_state::PuzzleState, depth: usize) -> Option<&puzzle_state::PuzzleState> {
    //println!("Depth: {}, number solved: {}", depth, ps.puzzle.num_complete());
    if ps.is_complete() {
        // We're done!
        Some(ps)
    } else {
        // Generate the children
        ps.create_children();
        // Iterate through each
        // TODO see about sorting prior to iteration
        for child in ps.children.iter_mut() {
            // Solve the child
            let res = recursively_solve(child, depth+1);
            // If it is not None, return the child result
            if res.is_some() {
                return res;
            }
            // TODO see about killing children that return None. Might alleviate the memory issues :)
        }

        None
    }
}

// Handle arguments
// Basically, yell a the user if they did something wrong. It's really a one sided argument
// If only it could handle my arguments with the borrow checker
fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("Running sample puzzle...");
            let filename = "./puzzles/standard/Regular5x5_1.txt";
            solve_puzzle(filename);
        },
        2 => {
            let filename = &args[1];
            solve_puzzle(filename);
        },
        _ => {
            // show a help message
            println!("Enter either no arguments or the path to a puzzle to be solved!");
        }
    };
}
