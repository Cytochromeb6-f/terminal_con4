use std::{fmt, thread, collections::{HashMap, VecDeque}};

#[derive(Clone)]
pub struct Grid {
    l: usize,
    w: usize,
    h: usize,
    vec: Vec<u8>,
    turn: u8,
}

impl Grid {
    // 0 means empty, 1 or 2 means the respective player
    // First index specifies row, with 0 being the bottom row
    // Second index specifies the column with 0 being the leftmost column
    pub fn new(l: usize, w: usize, h: usize) -> Self{
        Grid { l, w, h, vec: vec![0; w*h], turn: 0 }
    }

    pub fn width(&self) -> usize {
        self.w
    }

    pub fn turn(&self) -> u8 {
        self.turn
    }

    fn array(&self, i: usize, j: usize) -> u8 {
        self.vec[i*self.w + j]
    }

    fn array_set(&mut self, i: usize, j: usize, value: u8) {
        if (0..self.h).contains(&i) && (0..self.w).contains(&j) {
            self.vec[i*self.w + j] = value
        }
    }

    pub fn legal_moves(&self) -> Vec<usize> {
        let mut legal = Vec::new();
        for j in 0..self.w {
            if self.array(self.h-1, j) == 0 {
                legal.push(j)
            }
        }
        legal
    }
    pub fn illegal_moves(&self) -> Vec<usize> {
        let mut illegal = Vec::new();
        for j in 0..self.w {
            if self.array(self.h-1, j) != 0 {
                illegal.push(j)
            }
        }
        illegal
    }
    
    pub fn play(&mut self, col: usize) -> usize {
        // Returns the position where the played disc landed
        for row in 0..self.h {
            if self.array(row, col) == 0 {
                self.array_set(row, col, self.turn%2 + 1);
                self.turn += 1;
                return row
            }
        }
        // Returns an illegal position if the column was full
       self.h
    }

    pub fn n_legal_f64(&self) -> f64 {
        let mut n_legal = 0.;
        for j in 0..self.w {
            if self.array(self.h-1, j) == 0 {
                n_legal += 1.;
            }
        }
        n_legal
    }

    pub fn next_grids(&self) -> HashMap<[usize; 2], Grid> {
        let moves = self.legal_moves();
        let mut grids = HashMap::with_capacity(moves.len());
        
        for col in self.legal_moves() {
            let mut grid = self.clone();
            let row = grid.play(col);
            grids.insert([row, col], grid);
        }
        grids
    }
    pub fn win_fast(&self, row: usize, col: usize) -> u8 {
        let player = self.array(row, col);
        
        // Column
        if row >= self.l-1 {
            'column_check: {
                for i in 1..self.l {
                    if self.array(row-i, col) != player {
                        break 'column_check
                    }
                }
                return player
            }
        }
        
        // Row
        let mut line_len = 1;
        for j in 1..(self.w-col) {
            if self.array(row,col+j) == player {    // Rightward
                line_len += 1;
            }
            else {
                break
            }
        }
        for j in 1..=col {
            if self.array(row,col-j) == player {    // Leftward
                line_len += 1;
            }
            else {
                break
            }
        }
        if line_len >= self.l {
            return player
        }

        // Diagonal: /
        line_len = 1;
        for k in 1..(self.h-row).min(self.w-col) {
            if self.array(row+k,col+k) == player {    // Up-rightward
                line_len += 1;
            }
            else {
                break
            }
        }
        for k in 1..(row+1).min(col+1) {
            if self.array(row-k,col-k) == player {    // Down-leftward
                line_len += 1;
            }
            else {
                break
            }
        }
        if line_len >= self.l {
            return player
        }
        // Diagonal: \
        line_len = 1;
        for k in 1..(self.h-row).min(col+1) {
            if self.array(row+k,col-k) == player {    // Up-leftward
                line_len += 1;
            }
            else {
                break
            }
        }
        for k in 1..(row+1).min(self.w-col) {
            if self.array(row-k,col+k) == player {    // Down-rightward
                line_len += 1;
            }
            else {
                break
            }
        }
        if line_len >= self.l {
            return player
        }
        
        0
    }

    
    fn walk_highlight(&mut self, mut i: usize, mut j: usize, v_i: i8, v_j: i8) -> u8 {
        // Same as regular walk but returns the indices of the win-line so it can be highlighted
        let mut p1_line: Vec<(usize, usize)> = Vec::with_capacity(self.l);
        let mut p2_line: Vec<(usize, usize)> = Vec::with_capacity(self.l);
        while (0..self.h).contains(&i) && (0..self.w).contains(&j) {
            match self.array(i,j) {
                    1 => {p1_line.push((i,j)); p2_line.clear()},
                    2 => {p1_line.clear(); p2_line.push((i,j))},
                    _ => {p1_line.clear(); p2_line.clear()}
                }
            if p1_line.len() >= self.l {
                for (i,j) in p1_line {
                    self.array_set(i, j, 10*self.array(i, j))
                }
                return 1
            } else if p2_line.len() >= self.l {
                for (i,j) in p2_line {
                    self.array_set(i, j, 10*self.array(i, j))
                }
                return 2
            }
            i = ((i as i8) + v_i) as usize;
            j = ((j as i8) + v_j) as usize;
        }
        0
    }
    pub fn win_highlight(&mut self) -> u8 {
        // Rows
        for i in 0..self.h {
            match self.walk_highlight(i, 0, 0, 1) {
                w if w != 0 => return w, _ => ()
            };
        }
        // Columns
        for j in 0..self.w {
            match self.walk_highlight(0, j, 1, 0) {
                w if w != 0 => return w, _ => ()
            };
        }
        // Diagonals
        for i in 1..=(self.h-self.l) {
            match self.walk_highlight(i, 0, 1, 1) {
                w if w != 0 => return w, _ => ()    // Upward from left side, excluding the corner
            };
            match self.walk_highlight(i, self.w-1, 1, -1) {
                w if w != 0 => return w, _ => ()    // Upward from right side, excluding the corner
            };
        }
        for j in 0..=(self.w-self.l) {
            match self.walk_highlight(0, j, 1, 1) {
                w if w != 0 => return w, _ => ()    // Rightward from bottom row
            };
        }
        for j in (self.l-1)..(self.w) {
            match self.walk_highlight(0, j, 1, -1) {
                w if w != 0 => return w, _ => ()    // Leftward from bottom row
            };
        }
        0
    }
}
impl fmt::Display for Grid {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::with_capacity(3*self.w*self.h);
        for j in 0..self.w {
            output = format!("{output} {j} ")
        }
        for i in 0..self.h {
            output = format!("{output}\n");
            for j in 0..self.w {
                output = match self.array(self.h-i-1, j) {
                    0 => format!("{output}[ ]"),
                    1 => format!("{output}[o]"),
                    2 => format!("{output}[x]"),
                    10 => format!("{output} o "),   // Used when highlighting player 1 win
                    20 => format!("{output} x "),   // Used when highlighting player 2 win
                    _ => format!("{output}err")
                }
            }
        }
        write!(f, "{output}")
    }
}
pub struct Branch {
    // Structure used for BFS exploration
    root: Grid,
    queue: VecDeque<(f64, Grid)>,
    score: f64,
}
impl Branch {
    pub fn new(root_grid: Grid, queue_capacity: usize) -> Self {
        Branch { root: root_grid, queue: VecDeque::with_capacity(queue_capacity), score: 0. }
    }

    pub fn score(&self) -> f64{
        self.score
    }

    fn bfs(&mut self, player: u8, depth: u8) {
        let mut keep_pushing = true;

        let relevance = 1.;
        self.queue.push_back((relevance, self.root.clone()));

        while self.queue.len() > 0 {
            let (relevance, grid) = self.queue.pop_front().unwrap();

            for ([row, col], child) in grid.next_grids() {
                match child.win_fast(row, col) {
                    0 => {                      // No one wins
                        if keep_pushing {
                            if child.turn == self.root.turn + depth {
                                keep_pushing = false;
                                continue;
                            }
                            self.queue.push_back((relevance/grid.n_legal_f64(), child));
                        }
                    },
                    // One loss has the same magnitude as legal_moves.len() wins  
                    w if w == player => {           // The player that the analysis is done for wins
                        self.score += relevance/grid.n_legal_f64();
                    },
                    _ => {                              // The opponent wins
                        self.score -= relevance;       
                    }
                }
            }
        }
    }
}

pub fn analyze_bfs_mt(grid: Grid, player: u8, depth: u8) -> usize {
    // BFS version of analyze_mt() 
    // Makes one thread per top-level legal move
    let relevance = 1./grid.n_legal_f64();
    
    let mut handles = Vec::new();
    
    // Try all possible moves
    for ([row, col], branch_grid) in grid.next_grids() {

        let queue_capacity = (grid.n_legal_f64() as usize).pow((depth-1).into());
        
        // Spawn one thread per 1st level branch
        handles.push(thread::spawn(move || {

            match branch_grid.win_fast(row, col) {
                0 => {                              // No one wins
                    let mut branch = Branch::new(branch_grid, queue_capacity);
                    branch.bfs(player, depth-1);

                    return (col, relevance*branch.score)
                },
                // One loss has the same magnitude as legal_moves.len() wins  
                w if w == player => {           // The player that the analysis is done for wins
                    return (col, relevance);       
                },
                _ => panic!()       // This should be unreachable if the turn of grid corresponds to player
            }
        }));
    }
    
    let best_col = handles.into_iter()
        .map(|handle| handle.join().unwrap())
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(col, _)| col);
    
    best_col.unwrap()
    
}