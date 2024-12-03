use std::{fmt, thread, collections::{HashMap, VecDeque}, hash::{DefaultHasher, Hash, Hasher}};




#[derive(Clone)]
pub struct Grid {
    l: usize,       // Length of disc-line required to win
    w: usize,
    h: usize,
    vec: Vec<u8>,   // 0: empty [ ], 1: player 1 [o], 2: player 2 [x], 10: player 1 highlighted, 20: player 2 highlighted
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

    pub fn player_to_move(&self) -> u8 {
        return self.turn%2 + 1
    }

    fn array(&self, i: usize, j: usize) -> u8 {
        self.vec[i*self.w + j]
    }

    fn array_set(&mut self, i: usize, j: usize, value: u8) {
        if (0..self.h).contains(&i) && (0..self.w).contains(&j) {
            self.vec[i*self.w + j] = value
        }
    }

    // Gives a vector with the indices of all non-full columns 
    pub fn legal_moves(&self) -> Vec<usize> {
        let mut legal = Vec::new();
        for j in 0..self.w {
            if self.array(self.h-1, j) == 0 {
                legal.push(j)
            }
        }
        legal
    }
    
    // Gives number of legal moves available
    fn n_legal_f64(&self) -> f64 {
        // Faster than running self.legal_moves.len()
        // Used for scaling backpropagated scores in the move tree

        let mut n_legal = 0.;
        for j in 0..self.w {
            if self.array(self.h-1, j) == 0 {
                n_legal += 1.;
            }
        }
        n_legal
    }

    
    // Plays in a disc in  given column
    pub fn play(&mut self, col: usize) -> usize {
        // Returns the position where the played disc landed
        for row in 0..self.h {
            if self.array(row, col) == 0 {
                self.array_set(row, col, self.player_to_move());
                self.turn += 1;
                return row
            }
        }

        // Returns an illegal position if the column was full
        self.h
    }

    
    // Gives all possible grid states that can be reached with one move 
    fn next_grids(&self) -> HashMap<[usize; 2], Grid> {
        let legal_moves = self.legal_moves();
        let mut grids = HashMap::with_capacity(legal_moves.len());
        
        for col in legal_moves {
            let mut grid = self.clone();
            let row = grid.play(col);
            grids.insert([row, col], grid);
        }
        grids
    }


    // Checks if any player has l discs in a line. Only looks at lines containing the coordinate (row, col)
    // Returns whether player 1, player 2 or neither (0) has won.
    // Used when exploring the move tree to check if a disc played at (row, col) results in victory.
    fn win_fast(&self, row: usize, col: usize) -> u8 {
        let player = self.array(row, col);
        
        // Vertical line
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
        
        // Horizontal line
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
        
        return 0
    }

    
    // Starts at (i,j) and walks with step velocity (v_i,v_j) until it hits a wall.
    // If an l-length continuous line of discs of the same type is found, then the discs
    // in that line will be highlighted. Returns which player won or 0 if no one won.
    fn walk_highlight(&mut self, mut i: usize, mut j: usize, v_i: i8, v_j: i8) -> u8 {
        let mut p1_line: Vec<(usize, usize)> = Vec::with_capacity(self.l);
        let mut p2_line: Vec<(usize, usize)> = Vec::with_capacity(self.l);
        while (0..self.h).contains(&i) && (0..self.w).contains(&j) {
            match self.array(i,j) {
                1 => {p1_line.push((i,j)); p2_line.clear()},
                2 => {p1_line.clear(); p2_line.push((i,j))},
                _ => {p1_line.clear(); p2_line.clear()}
            }

            // Highlights by changing 1 --> 10, 2 --> 20. 
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


    // Checks if any player has won and highlights the winning line.
    // Slower than self.win_fast() but checks the whole grid. 
    pub fn win_highlight(&mut self) -> u8 {
        // Horizontal lines
        for i in 0..self.h {
            match self.walk_highlight(i, 0, 0, 1) {
                win if win != 0 => return win, _ => ()
            };
        }
        // Vertical lines
        for j in 0..self.w {
            match self.walk_highlight(0, j, 1, 0) {
                win if win != 0 => return win, _ => ()
            };
        }
        // Diagonals
        for i in 1..=(self.h-self.l) {
            match self.walk_highlight(i, 0, 1, 1) {
                win if win != 0 => return win, _ => ()    // Upward from left side, excluding the corner
            };
            match self.walk_highlight(i, self.w-1, 1, -1) {
                win if win != 0 => return win, _ => ()    // Upward from right side, excluding the corner
            };
        }
        for j in 0..=(self.w-self.l) {
            match self.walk_highlight(0, j, 1, 1) {
                win if win != 0 => return win, _ => ()    // Rightward from bottom row
            };
        }
        for j in (self.l-1)..(self.w) {
            match self.walk_highlight(0, j, 1, -1) {
                win if win != 0 => return win, _ => ()    // Leftward from bottom row
            };
        }
        0
    }

    
}
impl fmt::Display for Grid {
    // Graphical representation of the grid. 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::with_capacity(3*self.w*self.h);
        for j in 0..self.w {
            output = format!("{output} {j} ")
        }
        match self.player_to_move() {
            1 => output = format!("{output}  (o) "),
            2 => output = format!("{output}  (x) "),
            _ => ()
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

impl Hash for Grid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vec.hash(state);
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}


// Structure used for BFS exploration of the move tree
struct Branch {
    root: Grid,
    queue: VecDeque<(f64, Grid)>,
    score: f64,
}
impl Branch {
    pub fn new(root_grid: Grid, queue_capacity: usize) -> Self {
        Branch { root: root_grid, queue: VecDeque::with_capacity(queue_capacity), score: 0. }
    }

    // Determines the score of this branch by searching through all possible combinations
    // of moves to a given depth. The score is increased when paths to own victory is found
    // and decreased when a paths to enemy victory is found. 
    fn bfs(&mut self, protagonist: u8, depth: u8) {
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
                    w if w == protagonist => {           // The protagonist wins
                        self.score += relevance/grid.n_legal_f64();
                    },
                    _ => {                              // The other player wins
                        self.score -= relevance;       
                    }
                }
            }
        }
    }
}



// Determines the best possible move for a given player, based on a given search depth.
// 
// The depth should be greater than 1. Unresonably large depth causes memory allocation errors.
// Uses multithreading. Makes one Branch from each top-level legal move and runs each in its own thread. 
// 
// Returns the index of the column whose branch has the highest score
// 
// Randomness is caused by internal reordering of the hashMaps when they are cloned.
pub fn analyze_bfs_mt(grid: Grid, protagonist: u8, depth: u8) -> usize {
    
 
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
                    branch.bfs(protagonist, depth-1);

                    return (col, relevance*branch.score)
                },
                // One loss has the same magnitude as legal_moves.len() wins  
                w if w == protagonist => {           // The protagonist wins
                    return (col, relevance);       
                },
                _ => panic!()       // This should be unreachable since the analysis is only run during the protagonist's turn
            }
        }));
    }
    
    let best_col = handles.into_iter()
        .map(|handle| handle.join().unwrap())
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(col, _)| col);
    
    best_col.unwrap()
    
}








#[derive(Clone)]
pub struct Node {
    // Structure used for minmax exploration
    grid: Grid,
    horizontal_threat_lines: Vec<f64>,
}
#[allow(dead_code)]
impl Node {
    fn new(grid: Grid) -> Self {
        let horizontal_threat_lines = vec![0.; grid.h*(grid.w-grid.l+1)];
        Node {grid, horizontal_threat_lines}
    }

    fn create_children(&self) -> HashMap<[usize; 2], Node> {
        // Gives new nodes for all possible grid states that can be reached with one move 
        let legal_moves = self.grid.legal_moves();
        let mut children = HashMap::with_capacity(legal_moves.len());
        
        for col in legal_moves {
            let mut child = self.clone();
            let row = child.grid.play(col);
            children.insert([row, col], child);
        }
        children
    }


    // Horizontal threats are stored between turns since the are easy to update without looking at the whole grid.
    // A threat_line is a contiguous line of disc positions with length grid.l.
    // An empty threat_line has level 0 if it contains discs of both types.
    // A threat_line which contains discs of both types always has level f64::INFINITY but is counted as 0.
    // A threat_line that contains D discs from player 1 has level +D.
    // A threat_line that contains D discs from player 2 has level -D.

    // The threat_lines are adressed by their leftmost disc position
    fn update_threat_lines(&mut self, i: usize, j: usize) {
        let player = self.grid.array(i, j);

        let th_w: usize = self.grid.w - self.grid.l + 1;


        // Horizontal threat_lines
        let th_i = i;
        let mut th_j = j;
        for _ in 0..self.grid.l {
            if th_j < th_w {
                match player {
                    1 => {
                        match self.horizontal_threat_lines[th_i*th_w + th_j] {
                            level if level.is_infinite() => continue,
                            level if (level < 0.) => {
                                self.horizontal_threat_lines[th_i*th_w + th_j] = f64::INFINITY
                            },
                            _  => {
                                self.horizontal_threat_lines[th_i*th_w + th_j] += 1.
                            },
                        }
                    },
                    2 => {
                        match self.horizontal_threat_lines[th_i*th_w + th_j] {
                            level if level.is_infinite() => continue,
                            level if (level > 0.) => {
                                self.horizontal_threat_lines[th_i*th_w + th_j] = f64::INFINITY
                            },
                            _ => {
                                self.horizontal_threat_lines[th_i*th_w + th_j] -= 1.
                            }
                        }
                    },
                    _ => panic!()       // This should be unreachable since (row,col) should not be empty
                }
            }
            if th_j==0 {
                break
            }
            th_j -= 1;
        }
    }

    fn get_value(&mut self, depth: u8, protagonist: u8, row: usize, col: usize, transp_table: &mut HashMap<u64, f64>) -> f64 {
        // Get the value of this node from the values of its children recursively
        // Also marks who won if this is a winning state
        
        // (row,col) is the position where the last disc was placed on the grid 
        // protagonist denotes wich player the analysis is done for
        
        // Get cached value is this state has been seen before.
        let state_id = calculate_hash(&self.grid);
        match transp_table.get(&state_id) {
            Some(stored_value) => return *stored_value,
            _ => ()
        }

        let mut value: f64;
        match self.grid.win_fast(row, col) {
            0 => {                                  // No one wins

                if depth == 0 {                                     // Depth limit reached
                    value = self.heuristic1(protagonist);      
                }      

                else if self.grid.n_legal_f64() == 0. {             // Game over (draw)
                    value = 0.
                }
                
                else if self.grid.player_to_move() == protagonist {      // The protagonist's turn
                    
                    value = f64::NEG_INFINITY;                             
                    
                    // Clone the grid and try all possible moves
                    for ([row, col], mut child) in self.create_children() {
                        child.update_threat_lines(row, col);

                        let child_value = child.get_value(depth-1, protagonist, row, col, transp_table);
                        
                        // Keep the maximal value
                        if child_value > value {    
                            value = child_value
                        }
                    }
                }
                else {                                                  // The other player's turn
                    value = f64::INFINITY;

                    // Clone the grid and try all possible moves
                    for ([row, col], mut child) in self.create_children() {
                        child.update_threat_lines(row, col);

                        let child_value = child.get_value(depth-1, protagonist, row, col, transp_table);
                        
                        // Keep the minimal value
                        if child_value < value {    
                            value = child_value
                        }
                    }
                }
            },
            w if w == protagonist => {          // The protagonist wins
                value = 3e6;
            },         
            _ => {                                  // The other player wins
                value = -3e6;
            }                                  
        }

        transp_table.insert(state_id, value);
        return value
    }

    // The length of the longest diagonal line that player would get if it placed a disk at (row,col).
    // Returns zero if (row,col) is not empty.
    fn diag_potential (&self, row: usize, col: usize, player: u8) -> f64 {
        if self.grid.array(row, col) != 0{
            return 0.
        }
        
        // Forward slash direction: /
        let mut fs_line_len: f64 = 1.;                                    
        for k in 1..(self.grid.h-row).min(self.grid.w-col) {
            if self.grid.array(row+k,col+k) == player {    // Up-rightward
                fs_line_len += 1.;
            }
            else {
                break
            }
        }
        for k in 1..(row+1).min(col+1) {
            if self.grid.array(row-k,col-k) == player {    // Down-leftward
                fs_line_len += 1.;
            }
            else {
                break
            }
        }
        // Backslash direction: \
        let mut bs_line_len: f64 = 1.;
        for k in 1..(self.grid.h-row).min(col+1) {
            if self.grid.array(row+k,col-k) == player {    // Up-leftward
                bs_line_len += 1.;
            }
            else {
                break
            }
        }
        for k in 1..(row+1).min(self.grid.w-col) {
            if self.grid.array(row-k,col+k) == player {    // Down-rightward
                bs_line_len += 1.;
            }
            else {
                break
            }
        }
        return fs_line_len.max(bs_line_len)
    }

    // Horizontal and diagonal threats on rows with prefered parity.
    // Player 1 wants threats in even rows, player 2 in odd rows. The bottom row is row 0.
    // Only works if self.h is even.
    fn heuristic1(&self, protagonist: u8) -> f64 {
        if self.grid.h%2 == 1 {
            panic!();
            // return 0.
        }
        let mut score = 0.;

        for i in 0..self.grid.h {

            let correct_parity_disc = (i%2 + 1) as u8;

            let sign = match correct_parity_disc {
                player if player == protagonist => 1.,
                _ => -1.
            };

            let mut row_score = 0.;
            
            let th_w: usize = self.grid.w - self.grid.l + 1;

            // Horizontal threats
            let th_i = i;
            for th_j in 0..th_w {
                // There are (grid.w-grid.l+1) different ways to win horizontally in every row
                let threat_level = self.horizontal_threat_lines[th_i*th_w + th_j];
                
                if threat_level.is_infinite() {
                    continue
                }
                else if threat_level >= (self.grid.l as f64) -1. {
                    row_score += 1e2
                }
                else {
                    row_score += threat_level
                }
            }
            
            
            // Diagonal threats
            for j in 0..self.grid.w {
                let diag_pot = self.diag_potential(i, j, correct_parity_disc);
                
                if diag_pot >= self.grid.l as f64 {
                    row_score += 1e2
                }
                else {
                    row_score += diag_pot
                }
            }

            // Lower threats are worth more 
            score += sign*row_score/(1.+i as f64)
        }
        
        return score
    }

}
pub fn analyze_minmax(grid: Grid, protagonist: u8, depth: u8) -> (usize, f64) {
    // This version does not save earlier work

    // Will play the move with the highest score 
    // Randomness is caused by internal reordering of the hashMaps when they are cloned
    
    let mut best_col = grid.width();  // This is an illegal move but should always be overridden.
    // let capacity = grid.l.pow(depth as u32);
    let root_node = Node::new(grid);

    // let mut transp_table: HashMap<u64, f64> = HashMap::with_capacity(capacity);
    let mut transp_table: HashMap<u64, f64> = HashMap::new();
    
    
    let mut best_value = f64::NEG_INFINITY;
    let mut best_immediate_value = f64::NEG_INFINITY;
    for ([row, col], mut child) in root_node.create_children() {
        child.update_threat_lines(row, col);

        let child_value = child.get_value(depth-1, protagonist, row, col, &mut transp_table);
        let child_immediate_value = child.heuristic1(protagonist);

        if child_value >= best_value {
            if (child_value == best_value) && (child_immediate_value <= best_immediate_value) {
                continue
            }
            best_immediate_value = child_immediate_value;
            best_value = child_value;
            best_col = col;
        }
    }
    
    
    return (best_col, best_value)
}