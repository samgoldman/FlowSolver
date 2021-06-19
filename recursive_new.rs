
fn recursive(puzzle: &Puzzle) -> Option<Puzzle> {
    if puzzle.is_complete() {
        return Some(puzzle.clone());
    }

    let children: Vec<Puzzle> = puzzle.create_children();

    for c  in 0..children.len() {
        let child: &Puzzle = children.get(c).unwrap();
        if child.is_solvable() == 1 || child.is_complete() {
            let res = recursive(child);
            if res.is_some() {
                return res;
            }
        }
    }

    None
}