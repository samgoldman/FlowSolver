extern crate time;

use std::cmp::max;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use time::Instant;

const NON_EXISTENT_CELL_ID: usize = 999;
const MAX_NEIGHBORS: usize = 6;

const SOLVABLE: i16 = 1;
const UNSOLVABLE_NO_CHILDREN: i16 = -1;
const UNSOLVABLE_DEAD_ENDS: i16 = -2;
const UNSOLVABLE_POOLS: i16 = -3;
const UNSOLVABLE_PATH_BLOCKED: i16 = -4;
const UNSOLVABLE_REGION: i16 = -5;

// All of the neighbor constants
const VERTICAL: usize = 0;
const HORIZONTAL: usize = 1;

// Structures and implementations related to flows
#[derive(Debug, Eq, Clone)]
pub struct Flow {
    pub id: usize,
    endpoints: [Option<CellId>; 2],
    letter: char,
}
impl PartialEq for Flow {
    fn eq(&self, _other: &Flow) -> bool {
        false
    }
}
impl Flow {
    // Update the endpoint at the given index to the given cellID
    // endpoint should be 0 or 1
    pub fn update_endpoint(&mut self, endpoint: usize, cell_id: CellId) {
        self.endpoints[endpoint] = Some(cell_id);
    }

    // Return true if this flow's endpoints are neighbors
    pub fn is_complete(&self, puzzle: &Puzzle) -> bool {
        puzzle
            .get_cell(self.get_endpoint(0))
            .unwrap()
            .is_neighbor(&self.get_endpoint(1))
    }

    pub fn get_endpoints(&self) -> [CellId; 2] {
        [self.endpoints[0].unwrap(), self.endpoints[1].unwrap()]
    }

    pub fn get_endpoint(&self, i: usize) -> CellId {
        self.get_endpoints()[i]
    }

    // Getter for attribute 'letter'
    pub fn get_letter(&self) -> char {
        self.letter
    }
}

#[derive(Debug, Eq, Clone, Copy)]
pub struct FlowId {
    pub index: usize,
}
impl PartialEq for FlowId {
    fn eq(&self, other: &FlowId) -> bool {
        self.index.eq(&other.index)
    }
}

// Structures and implementations related to cells
#[derive(Debug, Clone, Eq)]
pub struct Cell {
    pub is_endpoint: bool,
    pub flow_id: Option<FlowId>,
    pub neighbors: Vec<CellId>,
    pub is_hex: bool,
}
impl PartialEq for Cell {
    fn eq(&self, _other: &Cell) -> bool {
        false
    }
}
impl Cell {
    // Update the given neighbor
    pub fn add_neighbor(&mut self, cell: CellId) {
        self.neighbors.push(cell);
    }

    // Given a CellId, check if that cell in a neighbor of this cell
    pub fn is_neighbor(&self, other: &CellId) -> bool {
        self.neighbors
            .iter()
            .filter(|n| n.index == other.index)
            .count()
            > 0
    }

    pub fn is_occupied(&self) -> bool {
        self.flow_id.is_some()
    }

    // Return the number of neighboring cells that are not occupied
    pub fn num_open_neighbors(&self, puzzle: &Puzzle) -> usize {
        self.neighbors
            .iter()
            .filter(|n| !puzzle.get_cell(**n).unwrap().is_occupied())
            .count()
    }

    // Just the number of neighbors
    pub fn num_neighbors(&self) -> usize {
        self.neighbors.len()
    }
}

#[derive(Debug, Default, Eq, Copy, Clone)]
pub struct CellId {
    pub index: usize,
}
impl PartialEq for CellId {
    fn eq(&self, other: &CellId) -> bool {
        self.index.eq(&other.index)
    }
}

#[derive(Debug, Eq, Clone, PartialEq)]
pub struct Puzzle {
    cells: Vec<Cell>,
    pub flows: Vec<Flow>,
    pub is_hex: bool,
    pub print_string: String,
}

impl Ord for Puzzle {
    fn cmp(&self, other: &Puzzle) -> Ordering {
        let s = self.h();
        let o = other.h();
        s.cmp(&o)
    }
}

// 'PartialOrd' needs to be implemented as well.
impl PartialOrd for Puzzle {
    fn partial_cmp(&self, other: &Puzzle) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Puzzle {
    // Create a new puzzle
    pub fn new(is_hex: bool, print_string: String) -> Puzzle {
        Puzzle {
            cells: Vec::new(),
            flows: Vec::new(),
            is_hex,
            print_string,
        }
    }

    // Crate a new cell
    pub fn new_cell(&mut self, is_endpoint: bool, flow_id: Option<FlowId>, is_hex: bool) -> CellId {
        let next_index = self.num_cells();
        self.cells.push(Cell {
            is_endpoint,
            flow_id,
            neighbors: vec![],
            is_hex,
        });

        CellId { index: next_index }
    }

    pub fn num_cells(&self) -> usize {
        self.cells.len()
    }

    pub fn get_cell(&self, id: CellId) -> Option<&Cell> {
        self.cells.get(id.index)
    }

    pub fn get_cell_mut(&mut self, id: CellId) -> Option<&mut Cell> {
        self.cells.get_mut(id.index)
    }

    pub fn new_flow(&mut self, letter: char) -> FlowId {
        let next_index = self.num_flows();
        self.flows.push(Flow {
            id: next_index as usize,
            endpoints: [None; 2],
            letter,
        });

        FlowId { index: next_index as usize }
    }

    pub fn print_self(&self) {
        let mut split_input: Vec<&str> = self.print_string.split('\n').collect();
        split_input.remove(0);
        let mut bridge_addendum = String::new();
        let mut bridge_count = 0;
        let mut cell = 0;
        // Loop over it printing everything out
        // Unless it is a cell character. Then replace it with the appropriate letter from the solved puzzle
        for line in &split_input {
            for c in line.chars() {
                if c.is_ascii_uppercase() || c == '.' || c == '*' {
                    if c == '*' {
                        bridge_count += 1;
                        bridge_addendum = format!(
                            "{}\nBridge {}: Vertical is {}, horizontal is {}\n",
                            bridge_addendum,
                            bridge_count,
                            self.get_flow(
                                self.get_cell(CellId { index: cell })
                                    .unwrap()
                                    .flow_id
                                    .unwrap()
                            )
                            .unwrap()
                            .letter,
                            self.get_flow(
                                self.get_cell(CellId { index: cell + 1 })
                                    .unwrap()
                                    .flow_id
                                    .unwrap()
                            )
                            .unwrap()
                            .letter
                        );
                        cell += 1;
                        print!("{}", bridge_count);
                    } else {
                        let flow = self.get_cell(CellId { index: cell }).unwrap().flow_id;
                        if flow.is_some() {
                            print!("{}", self.get_flow(flow.unwrap()).unwrap().get_letter());
                        } else {
                            print!("{}", c);
                        }
                    }
                    cell += 1;
                } else {
                    print!("{}", c);
                }
            }
            println!();
        }
        println!("{}\n", bridge_addendum);
    }

    pub fn num_flows(&self) -> u64 {
        self.flows.len() as u64
    }

    pub fn get_flow(&self, id: FlowId) -> Option<&Flow> {
        self.flows.get(id.index)
    }

    pub fn get_flow_mut(&mut self, id: FlowId) -> Option<&mut Flow> {
        self.flows.get_mut(id.index)
    }

    pub fn num_complete(&self) -> u64 {
        self.flows
            .iter()
            .filter(|flow| flow.is_complete(self))
            .count() as u64
    }

    pub fn num_open_cells(&self) -> u64 {
        self.cells.iter().filter(|cell| !cell.is_occupied()).count() as u64
    }

    pub fn is_complete(&self) -> bool {
        self.num_complete() == self.num_flows() && self.num_open_cells() == 0
    }

    // Return a vector of all endpoints for flows that are not complete
    pub fn get_endpoints_for_incomplete_flows(&self) -> Vec<CellId> {
        let mut endpoints = Vec::new();

        // If the flow is incomplete, push its two endpoints onto the vector
        for flow in self.flows.iter().filter(|flow| !flow.is_complete(self)) {
            endpoints.push(flow.get_endpoint(0));
            endpoints.push(flow.get_endpoint(1));
        }

        endpoints
    }

    pub fn create_children(&self) -> Vec<Puzzle> {
        let endpoint_id = self.get_endpoint_to_extend();

        if endpoint_id.index == NON_EXISTENT_CELL_ID {
            return vec![];
        }

        let endpoint_cell = self.get_cell(endpoint_id).unwrap();
        let flow_id = endpoint_cell.flow_id.unwrap();
        let flow = self.get_flow(flow_id).unwrap();

        let endpoint_index = if flow.get_endpoint(0).index == endpoint_id.index {
            0
        } else {
            1
        };

        let mut children: Vec<Puzzle> = vec![];

        for n_id in endpoint_cell
            .neighbors
            .iter()
            .filter(|n_id| !self.get_cell(**n_id).unwrap().is_occupied())
        {
            let mut child = self.clone();

            let cell_to_move_to = child.get_cell_mut(*n_id).unwrap();

            cell_to_move_to.flow_id = Some(FlowId {
                index: flow_id.index,
            });
            cell_to_move_to.is_endpoint = true;

            child
                .get_cell_mut(child.get_flow(flow_id).unwrap().endpoints[endpoint_index].unwrap())
                .unwrap()
                .is_endpoint = false;

            let child_flow = child.get_flow_mut(flow_id).unwrap();
            child_flow.update_endpoint(endpoint_index, CellId { index: n_id.index });

            children.push(child);
        }

        children
    }

    pub fn num_possible_children(&self) -> u64 {
        let endpoint_id = self.get_endpoint_to_extend();
        // If there is no endpoint to extend, there are no possible children
        if endpoint_id.index == NON_EXISTENT_CELL_ID {
            return 0;
        }
        let endpoint_cell = self.get_cell(endpoint_id).unwrap();
        endpoint_cell.num_open_neighbors(self) as u64
    }

    // Basically, find the endpoint with the fewest open neighbors (possibilities) and return that one
    pub fn get_endpoint_to_extend(&self) -> CellId {
        let possible_endpoints = self.get_endpoints_for_incomplete_flows();
        let mut min_open = MAX_NEIGHBORS + 1; // Cannot have more open than the maximum number of neighbors

        // Default value in case no endpoints are found
        let mut min_open_cell_id = CellId {
            index: NON_EXISTENT_CELL_ID,
        };
        for cell_id in &possible_endpoints {
            let cell = self.get_cell(*cell_id);

            // If this endpoint has fewer open neighbors than the current minimum,
            // set the current minimum to this endpoint
            if cell.unwrap().num_open_neighbors(self) < min_open {
                min_open = cell.unwrap().num_open_neighbors(self);
                min_open_cell_id = *cell_id;
            }
        }

        // If no endpoints are found, return immediately
        // This would likely happen if all flows are complete, but there remain cells open
        if min_open_cell_id.index == NON_EXISTENT_CELL_ID {
            return min_open_cell_id;
        }

        // If there < 2 neighbors open for the selected endpoint,
        // Determine if there is a neighbor which only has one open neighbor. This is a forced move
        // Don't do this check if < 2 neighbors open, because that is already a forced move
        if min_open > 1 {
            for cell_id in &possible_endpoints {
                let cell = self.get_cell(*cell_id).unwrap();

                // Check each of the endpoint's neighbors
                for n_index in cell.neighbors.iter() {
                    let neighbor = self.get_cell(*n_index).unwrap();

                    // If the neighbor is open and has only one neighbor, the endpoint must be
                    // the next one to move, so return it
                    if !neighbor.is_occupied() && neighbor.num_open_neighbors(self) == 1 {
                        return *cell_id;
                    }
                }
            }
        }

        // Return the selected endpoint
        min_open_cell_id
    }

    // Is this puzzle solvable in its current state?
    // 1 if yes
    // <1 if no, corresponding to the reason (for statistics)
    pub fn is_solvable(&self) -> i16 {
        if self.num_possible_children() == 0 {
            return UNSOLVABLE_NO_CHILDREN;
        }

        if !self.is_hex {
            // I think these checks need tweaking for hex puzzles. Not sure though
            for i in 0..self.num_cells() {
                let cell = self.get_cell(CellId { index: i }).unwrap();

                // Detect dead ends - an empty cell connected only to one other empty cell and no endpoints
                // Any flow going into this would have no endpoints to connect to and no way to get out,
                // So it is impossible to solve
                if !cell.is_occupied() && cell.num_open_neighbors(self) == 1 {
                    let mut has_endpoint_neighbor = false;
                    for n_index in cell.neighbors.iter() {
                        if self.get_cell(*n_index).unwrap().is_endpoint {
                            has_endpoint_neighbor = true;
                            break;
                        }
                    }

                    if !has_endpoint_neighbor {
                        return UNSOLVABLE_DEAD_ENDS;
                    }
                }

                // Check for "pools" - when a flow doubles back on itself - these are illegal and generally pesky
                if cell.is_occupied() && cell.num_open_neighbors(self) <= 1 {
                    let mut same_flow_count = 0;
                    for n_index in cell.neighbors.iter() {
                        let neighbor = self.get_cell(*n_index).unwrap();
                        if neighbor.is_occupied()
                            && neighbor.flow_id.unwrap().index == cell.flow_id.unwrap().index
                        {
                            same_flow_count += 1;
                        }
                    }
                    if same_flow_count > 2 {
                        return UNSOLVABLE_POOLS;
                    }
                }
            }
        }

        // Idea for connected component analysis gotten from: https://mzucker.github.io/2016/08/28/flow-solver.html
        // Algorithm for connected component analysis is from wikipedia: https://en.wikipedia.org/wiki/Connected-component_labeling
        let mut preliminary_connected_component_sets: Vec<Vec<usize>> = vec![];
        let mut visited: Vec<usize> = vec![];

        // Create the connected components: basically each vector in the vector ccs contains the IDs of cells connected to each other
        for i in 0..self.num_cells() {
            if visited.contains(&i) {
                continue;
            }

            let mut queue: Vec<usize> = vec![i];
            // Loop through the queue of cells to visit
            while !queue.is_empty() {
                let curr_id = queue.pop().unwrap();

                if visited.contains(&curr_id) {
                    continue;
                }

                visited.push(curr_id);

                let curr_cell = self.get_cell(CellId { index: curr_id }).unwrap();

                // Occupied cells don't count as being in regions
                if curr_cell.is_occupied() {
                    continue;
                }

                // If a cell isn't added to an existing set, create a new region for it afterwards
                let mut added_to_set = false;
                // Check each of the neighbors
                for neighbor_id in curr_cell.neighbors.iter() {
                    // First, check if the neighbor is already in a region
                    // If it is, add this cell to that region
                    for ccs in preliminary_connected_component_sets.iter_mut() {
                        if ccs.contains(&neighbor_id.index) {
                            ccs.push(curr_id);
                            added_to_set = true;
                        }
                    }

                    // Also, if the neighbor hasn't already been visited and isn't in the queue, add it to the queue to be considered
                    if !visited.contains(&neighbor_id.index) && !queue.contains(&neighbor_id.index)
                    {
                        queue.push(neighbor_id.index);
                    }
                }

                if !added_to_set {
                    // If needed, create the region
                    preliminary_connected_component_sets.push(vec![curr_id]);
                }
            }
        }

        // Create the final regions, as there is a possibility that regions were not fully connected in the prior loop
        let mut connected_component_sets: Vec<Vec<usize>> = vec![];
        for i in 0..preliminary_connected_component_sets.len() {
            let set_i = preliminary_connected_component_sets.get(i).unwrap();
            let mut added_to_final = false;
            'j_loop: for set_j in connected_component_sets.iter_mut() {
                for id_j in set_j.iter() {
                    let cell_j = self.get_cell(CellId { index: *id_j }).unwrap();

                    for n in cell_j.neighbors.iter() {
                        if set_i.contains(&n.index) {
                            set_j.extend(set_i);
                            added_to_final = true;
                            break 'j_loop;
                        }
                    }
                }
            }

            if !added_to_final {
                connected_component_sets.push(set_i.clone());
            }
        }

        // Analyze the regions - Part 1: check if each region has at least one pair of
        // endpoints neighboring it (both endpoints neighbor at least one member of the region)
        'region_loop_1: for region in connected_component_sets.iter() {
            'flow_loop_1: for f in 0..self.num_flows() {
                let flow = self.get_flow(FlowId { index: f as usize }).unwrap();
                if flow.is_complete(self) {
                    continue;
                }

                let endpoint_0 = self.get_cell(flow.get_endpoint(0)).unwrap();
                let endpoint_1 = self.get_cell(flow.get_endpoint(1)).unwrap();

                let mut res = false;

                for n in endpoint_0.neighbors.iter() {
                    if region.contains(&n.index) {
                        res = true;
                    }
                }
                if !res {
                    continue 'flow_loop_1; // Region doesn't neighbor the first endpoint. Continue to next flow
                }

                for n in endpoint_1.neighbors.iter() {
                    if region.contains(&n.index) {
                        continue 'region_loop_1; // If we hit this point, the region contains both endpoints. Check next region
                    }
                }
            }
            return UNSOLVABLE_REGION;
        }

        // Analyze each flow: both endpoints must have a neighboring region in common, otherwise connecting them is impossible
        'flow_loop_2: for f in 0..self.num_flows() {
            let flow = self.get_flow(FlowId { index: f as usize }).unwrap();
            if flow.is_complete(self) {
                continue;
            }
            let endpoint_0 = self.get_cell(flow.get_endpoint(0)).unwrap();
            let endpoint_1 = self.get_cell(flow.get_endpoint(1)).unwrap();

            'region_loop_2: for region in connected_component_sets.iter() {
                let mut res = false;

                for n in endpoint_0.neighbors.iter() {
                    if region.contains(&n.index) {
                        res = true;
                    }
                }
                if !res {
                    continue 'region_loop_2; // This region doesn't neighbor the first endpoint. Check the next region
                }

                for n in endpoint_1.neighbors.iter() {
                    if region.contains(&n.index) {
                        continue 'flow_loop_2; // This region contains both endpoints. Check the next flow
                    }
                }
            }
            return UNSOLVABLE_PATH_BLOCKED;
        }
        SOLVABLE
    }

    // Magic numbers galore! (once upon a time)
    // Anyway, return the score of a board
    pub fn h(&self) -> u64 {
        // Modified from https://mzucker.github.io/2016/08/28/flow-solver.html (incorporates parts of g() and h() into one)
        1000 - self.num_open_cells() + self.num_complete() * 2 - self.num_possible_children()
    }
}

// Begin solving the puzzle located in the given file
// Includes parsing the puzzle, creating the initial puzzle state, and recursively solving the puzzle
// Does most of the work
fn solve_puzzle(filename: &str) {
    let path = Path::new(filename);

    // Verify valid extension
    let extension = path.extension().and_then(OsStr::to_str);
    if extension != Some("txt") {
        println!("At this time, puzzle files must be text files (.txt)!");
        return;
    }

    let display = path.display();
    println!("Solving the puzzle located at: {}\n", display);

    // Open the path in read-only mode, returns 'io::Result<File>'
    let mut file = match File::open(&path) {
        // The 'description' method of 'io::Error' returns a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns 'io::Result<usize>'
    let mut input = String::new();
    match file.read_to_string(&mut input) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => print!("{}:\n{}\n\n", display, input),
    }

    // Split the input based on lines
    let mut split_input: Vec<&str> = input.split('\n').collect();
    split_input.remove(0);

    // Check if the puzzle is a HEX puzzle
    let puzzle_type: String = input
        .chars()
        .skip(0)
        .take(input.find('\n').unwrap() - 1)
        .collect();
    let is_hex = puzzle_type == "HEX";

    // The initial puzzle
    let mut puzzle: Puzzle = Puzzle::new(is_hex, input.clone());

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
        if line.starts_with("//") {
            continue;
        }
        for (col, c) in line.chars().enumerate() {
            // Check if the character is a cell character
            if c.is_ascii_uppercase() || c == '.' || c == '*' {
                let is_bridge = c == '*'; // Asterisks are bridges
                let is_endpoint = c.is_ascii_uppercase();

                if !is_bridge {
                    // Create the new cell
                    let cell_id: CellId = puzzle.new_cell(is_endpoint, None, is_hex);

                    // Create a key for the map from the coordinates of the cell, and insert it into the map with the new cell id
                    let key: String = format!("{}-{}", col, row);
                    cell_map.insert(key, cell_id.index);

                    // If the cell is an endpoint, either create a new flow if needed or update an existing one
                    if c.is_ascii_uppercase() {
                        let mut flow_id_1 = None;
                        let mut flow_exists = false;

                        // Check if a flow with the current letter already exists
                        // If it does, update hte appropriate values
                        for (count, flow) in puzzle.flows.iter_mut().enumerate() {
                            if flow.get_letter() == c {
                                flow_exists = true;
                                flow.update_endpoint(1, cell_id);
                                flow_id_1 = Some(FlowId { index: count });
                            }
                        }

                        // If the flow doesn't exist, create a new one
                        if !flow_exists {
                            let flow_id = puzzle.new_flow(c);
                            let flow = puzzle.get_flow_mut(flow_id).unwrap();
                            flow_id_1 = Some(flow_id);
                            flow.update_endpoint(0, cell_id);
                        }

                        // Update the new cell with the appropriate flow
                        puzzle.get_cell_mut(cell_id).unwrap().flow_id = flow_id_1;
                    }
                } else {
                    // Bridges can't have a flow to set up, but do have an extra cell associated with them
                    let cell_id1: CellId = puzzle.new_cell(is_endpoint, None, is_hex);
                    let cell_id2: CellId = puzzle.new_cell(is_endpoint, None, is_hex);

                    let key1: String = format!("{}-{}--", col, row);
                    let key2: String = format!("{}-{}-|", col, row);
                    cell_map.insert(key1, cell_id1.index);
                    cell_map.insert(key2, cell_id2.index);
                }

                // Update the max size variables
                max_cell_row = max(max_cell_row, row);
                max_cell_col = max(max_cell_col, col);
            }
        }
        row += 1;
    }

    row = 0;
    let mut neighbors = 0; // Count the number of neighbor relationships
                           // Again, loop through all characters in the board configuration
                           // This time, look for neighbor characters: '-', '|', '/', '\'
                           // When one is found, update the appropriate cells
    for line in &split_input {
        if line.starts_with("//") {
            continue;
        }
        for (col, c) in line.chars().enumerate() {
            // If the character is a neighbor character, proceed
            if c == '-' || c == '/' || c == '\\' || c == '|' {
                // Get the coordinates of the two neighbors, and the neighbor relationship for each
                let (col1, row1, col2, row2, direction) = if c == '-' {
                    // If the current column is greater than the max width, this is a warped relationship. The neighbor is in the first column.
                    if col > max_cell_col {
                        (col - 1, row, 0, row, HORIZONTAL)
                    } else {
                        (col - 1, row, col + 1, row, HORIZONTAL)
                    }
                } else if c == '/' {
                    (col - 1, row + 1, col + 1, row - 1, HORIZONTAL) // These aren't really horizontal, but close enough, and it doesn't really matter
                } else if c == '\\' {
                    (col - 1, row - 1, col + 1, row + 1, HORIZONTAL)
                } else {
                    // If the row is greater than the max height, this is warped relationship. The neighbor is in the first row.
                    if row > max_cell_row {
                        (col, row - 1, col, 0, VERTICAL)
                    } else {
                        (col, row - 1, col, row + 1, VERTICAL)
                    }
                };

                // Recreate the map keys for the two neighbors
                neighbors += 1;
                let mut key1: String = format!("{}-{}", col1, row1);
                let mut key2: String = format!("{}-{}", col2, row2);

                if !cell_map.contains_key(&key1) {
                    key1 = if direction == VERTICAL {
                        format!("{}-{}-|", col1, row1)
                    } else {
                        format!("{}-{}--", col1, row1)
                    };
                }
                if !cell_map.contains_key(&key2) {
                    key2 = if direction == VERTICAL {
                        format!("{}-{}-|", col2, row2)
                    } else {
                        format!("{}-{}--", col2, row2)
                    };
                }

                puzzle
                    .get_cell_mut(CellId {
                        index: *cell_map.get(&key1).unwrap(),
                    })
                    .unwrap()
                    .add_neighbor(CellId {
                        index: *cell_map.get(&key2).unwrap(),
                    });
                puzzle
                    .get_cell_mut(CellId {
                        index: *cell_map.get(&key2).unwrap(),
                    })
                    .unwrap()
                    .add_neighbor(CellId {
                        index: *cell_map.get(&key1).unwrap(),
                    });
            }
        }
        row += 1;
    }

    // Status info
    println!("Number of cells: {}", puzzle.num_cells());
    println!("Number of flows: {}", puzzle.num_flows());
    println!("Number of neighbors: {}\n\n", neighbors);

    let start = Instant::now();
    // Solve it. Just like that. It's done!
    let res = greedy_best_first(puzzle);

    if let Some(r) = res {
        r.print_self();
        println!("Finished in {} seconds!", start.elapsed());
    } else {
        println!("Uh oh, no solution!");
        println!("Failed in {} seconds!", start.elapsed());
    }
}

// Not really sure if this is greedy best first any more, but I'm not changing the name now
// Solve the given PuzzleState, if possible. If not, return None
fn greedy_best_first(puzzle: Puzzle) -> Option<Puzzle> {
    let mut frontier: BinaryHeap<Puzzle> = BinaryHeap::new(); // Puzzles to consider
    frontier.push(puzzle);

    // Stats
    let mut frontier_max = 0;
    let mut states_visited: u64 = 0;
    let mut children_discarded: u64 = 0;
    let mut states_created = 1;
    let mut max_flows_completed: u64 = 0;
    let mut discarded_no_children: u64 = 0;
    let mut discarded_dead_end: u64 = 0;
    let mut discarded_pools: u64 = 0;
    let mut discarded_blocked: u64 = 0;
    let mut discarded_cc: u64 = 0;

    let mut avg_num_flows_complete = 0;
    let mut avg_num_cells_open = 0;

    let mut latest: Option<Puzzle> = None;

    while !frontier.is_empty() {
        states_visited += 1;
        frontier_max = max(frontier_max, frontier.len());

        let curr_state = frontier.pop().unwrap();

        max_flows_completed = max(max_flows_completed, curr_state.num_complete());
        avg_num_flows_complete += curr_state.num_complete();
        avg_num_cells_open += curr_state.num_open_cells();

        let mut children = curr_state.create_children();
        states_created += children.len();

        // Evaluate each child
        while !children.is_empty() {
            let child = children.pop().unwrap();

            // Yay! We're done! Print some stats and return
            if child.is_complete() {
                println!("---STATS AT {}---", states_visited);
                println!("States visited: {}\nMax Frontier Size: {}\nChildren Discarded: {}\nPercent Discarded: {}\nCurrent Frontier: {}\nStates created: {}",
                         states_visited, frontier_max, children_discarded, (children_discarded as f64 / states_created as f64), frontier.len(), states_created);
                println!(
                    "Num Cells Open: {}\nMax Flows Complete: {}",
                    child.num_open_cells(),
                    max_flows_completed
                );
                println!("Discard Stats:\n\tNum children: {}\n\tDead end: {}\n\tPools: {}\n\tBlocked Flow: {}\n\tCC Failed: {}\n",
                           (discarded_no_children as f64 / children_discarded as f64),
                           (discarded_dead_end as f64 / children_discarded as f64),
                           (discarded_pools as f64 / children_discarded as f64),
                           (discarded_blocked as f64 / children_discarded as f64),
                           (discarded_cc as f64 / children_discarded as f64));

                return Some(child);
            }
            let solvable_status = child.is_solvable(); // Determine if child is solvable
                                                       // If solvable, add it to the list to consider
            if solvable_status == 1 {
                frontier.push(child);
            } else {
                // Otherwise, update some stats and then forget about the child
                children_discarded += 1;
                if solvable_status == UNSOLVABLE_NO_CHILDREN {
                    discarded_no_children += 1;
                } else if solvable_status == UNSOLVABLE_DEAD_ENDS {
                    discarded_dead_end += 1;
                } else if solvable_status == UNSOLVABLE_POOLS {
                    discarded_pools += 1;
                } else if solvable_status == UNSOLVABLE_PATH_BLOCKED {
                    discarded_blocked += 1;
                } else if solvable_status == UNSOLVABLE_REGION {
                    discarded_cc += 1;
                }
            }
        }

        // Print some stats every so often to keep the user happy, and let them know that we're still chugging along
        let skip = 10000;
        if states_visited % skip == 0 {
            println!(
                "{}\t{}\t{:.4}\t{:.1}\t{}\t{:.4}",
                states_visited,
                frontier_max,
                (children_discarded as f64 / states_created as f64),
                avg_num_cells_open as f64 / skip as f64,
                max_flows_completed,
                avg_num_flows_complete as f64 / skip as f64
            );
            max_flows_completed = 0;
            avg_num_cells_open = 0;
            avg_num_flows_complete = 0;
        }

        latest = Some(curr_state);
    }

    // Never want to get here - if we did, the solver failed
    println!("---STATS AT {}---", states_visited);
    println!("States visited: {}\nMax Frontier Size: {}\nChildren Discarded: {}\nPercent Discarded: {}\nCurrent Frontier: {}\nStates created: {}",
             states_visited, frontier_max, children_discarded, (children_discarded as f64 / states_created as f64), frontier.len(), states_created);
    println!("Max Flows Complete: {}", max_flows_completed);
    println!("Discard Stats:\n\tNum children: {}\n\tDead end: {}\n\tPools: {}\n\tBlocked Flow: {}\n\tCC Failed: {}\n",
             (discarded_no_children as f64 / children_discarded as f64),
             (discarded_dead_end as f64 / children_discarded as f64),
             (discarded_pools as f64 / children_discarded as f64),
             (discarded_blocked as f64 / children_discarded as f64),
             (discarded_cc as f64 / children_discarded as f64));
    println!("Uh oh, no solution!");

    if let Some(l) = latest {
        println!("Latest configuration:");
        l.print_self();

        println!("Latest children:");
        let children = l.create_children();

        for child in children.iter() {
            child.print_self();
            println!("Solvable: {}\n", child.is_solvable());
        }
    }
    None
}

// Handle arguments
// Basically, yell at the user if they did something wrong. It's really a one sided argument
// If only it could handle my arguments with the borrow checker...
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            let filename = &args[1];
            solve_puzzle(filename);
        }
        _ => {
            println!("Enter the path to a puzzle to be solved!");
        }
    };
}
