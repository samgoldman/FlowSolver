# Flow Free Solver

There are two versions of this solver:
### Python
0. Located in /old_python (because it's older)
1. Only supports standard Flow puzzles (no bridges, hexes, warps)
2. Will scan puzzle screenshots
3. Because of (2), requires the python OpenCV module to be installed
4. Filenames of puzzles must be hardcoded in (I know, I'm bad)
5. Lacks pretty much any commenting
6. Has been tested with 5x5, 7x7, and 9x9 puzzles. Other test screenshots are included, but result in recursion errors (too deep)
7. General design principle:
    1. Convert puzzle to a 2D array
    2. Starting with the flow with the fewest options, extend that flow for each option and recursively solve the new board
    3. If reach an impossible to solve board, return and continue with other children
8. There are some optimizations, such as favoring children which stay on the most direct route, checking for isolated cells, favoring children with the most flows complete, etc.
### Rust
0. Located everywhere else (prebuilt windows exe is in ./build)
1. Supports standard, hex, warp and bridge puzzles
2. Will not scan puzzle screenshots (yes, you have to do some of the work, see section below)
3. Does not require the python OpenCV modules (it's not even in python, why would it need a python module)
4. A single filename must be passed when running flow_free_solver_rust.exe
5. Some comments here and there
6. Has been tested on:
    1. 12x12 standard boards (that one take 20 seconds)
    2. 14x14 warps (ranges from 2 seconds to ~10 hours - yeah, that's a wild range. These are the most tested because I often used this project to solve problems I'm stuck on)
    3. Small hexes (note: has been tested on a large hex, with so far inconclusive results. I'd test it on others, but hex puzzles are a PAIN to convert into the specified format)
    4. Small bridges (just haven't gotten around to testing many)
7. General design principle: Rust is simultaneously awesome and the bane of my existence
    1. Convert each puzzle to a series of cells
    2. Link each cell with its designated neighbors
    3. Starting with the endpoint with the fewest options, generate all possible children based on that endpoint
    4. Add the children (after checking if they are solvable) to a max heap, which uses a heuristic to determine the order they should be visited in
        1. The heuristic prioritizes puzzles with fewer open cells, fewer children, and more flows solved
    5. Loop through each puzzle on the heap with steps 3-5 until a complete puzzle is found
    
## File Format for Rust Puzzle Input
1. The file must be a .txt file
2. The first line of the file must be `STANDARD`, `BRIDGES`, `HEX`, or `WARPS`
3. Every subsequent line describes the puzzle:
    1. Each cell must be represented by one of the following:
        1. `.`: empty cell (square or hex)
        2. `*`: empty bridge
        3. `[A-Z]`: a flow endpoint with the corresponding letter. Must be either 0 or 2 of each letter.
    2. Cells are connected by neighbor characters:
        1. `|`: top to bottom
        2. `\\`: bottom right to top left (single backslash)
        3. `/`: bottom left to top right
        4. `-`:
            1. Square puzzles: left to right
            2. Hex puzzles: alternating bottom right to top left and bottom left to top right
        5. Note: in the case of warps, the warps should be designated by a `-` or `|` after the last row or column

Notes:
1. Cells may be skipped: puzzles do not need to be be perfect rectangles
2. Walls may be created by omitting the appropriate neighbor characters
   
### Standard Example:
```aidl
STANDARD
A-.-B-.-C
| | | | |
.-.-D-.-E
| | | | |
.-.-.-.-.
| | | | |
.-B-.-C-.
| | | | |
.-A-D-E-.
```

### Bridges Example:
```aidl
BRIDGES
  A-B-C
  | | |
A-.-.-.-C
| | | | |
D-.-*-.-D
| | | | |
E-.-B-.-E
  | | |
  .-.-.
```

### Warps Example:
```aidl
WARPS
.-.-.-.-.-.-.-
| | | | | | |
B-.-.-.-B-.-D
| | | | | | |
.-.-F-.-A-.-.
| | | | | | |
.-C-.-.-.-.-.
| | | | | | |
.-.-.-C-D-.-.
| | | | | | |
.-.-A-.-F-.-E
| | | | | | |
.-.-.-.-E-.-.-
|           |
```
Note the extra pipes and dashes extruding from the right and bottom, connecting the cells at the far right and left/top and bottom

### Hexes Example:
```aidl
HEX
  A   B
 /|\ /|\
C-.-.-.-D
|/|\|/|\|
.-.-.-.-E
|/|\|/|\|
.-A-.-D-.
|/|\|/|\|
.-C-B-E-.
```

Note that these are not laid out like hexes. Hex puzzles must be flattened into rows. The (shitty) white lines in the image below show how rows are formed. Due to this, the `-` neighbor character alternates which neighbor relationship it refers to. Also note how in the top row, there are cells skipped. This is acceptable for all types of puzzles.

<img width="360" height="740" src="https://raw.githubusercontent.com/samgoldman/flowsolver/master/puzzles/hex/Classic5x5_1.jpg" />
