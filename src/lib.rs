use std::{collections::{HashMap, VecDeque}, fmt, hash::{DefaultHasher, Hash, Hasher}, thread::{self}};




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

    fn read(&self, i: usize, j: usize) -> u8 {
        self.vec[i*self.w + j]
    }

    fn set(&mut self, i: usize, j: usize, value: u8) {
        if (0..self.h).contains(&i) && (0..self.w).contains(&j) {
            self.vec[i*self.w + j] = value
        }
    }

    // Gives a vector with the indices of all non-full columns 
    pub fn legal_moves(&self) -> Vec<usize> {
        let mut legal = Vec::new();
        for j in 0..self.w {
            if self.read(self.h-1, j) == 0 {
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
            if self.read(self.h-1, j) == 0 {
                n_legal += 1.;
            }
        }
        n_legal
    }

    
    // Plays in a disc in  given column
    pub fn play(&mut self, col: usize) -> usize {
        // Returns the position where the played disc landed
        for row in 0..self.h {
            if self.read(row, col) == 0 {
                self.set(row, col, self.player_to_move());
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
        let player = self.read(row, col);
        
        // Vertical line
        if row >= self.l-1 {
            'column_check: {
                for i in 1..self.l {
                    if self.read(row-i, col) != player {
                        break 'column_check
                    }
                }
                return player
            }
        }
        
        // Horizontal line
        let mut line_len = 1;
        for j in 1..(self.w-col) {
            if self.read(row,col+j) == player {    // Rightward
                line_len += 1;
            }
            else {
                break
            }
        }
        for j in 1..=col {
            if self.read(row,col-j) == player {    // Leftward
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
            if self.read(row+k,col+k) == player {    // Up-rightward
                line_len += 1;
            }
            else {
                break
            }
        }
        for k in 1..(row+1).min(col+1) {
            if self.read(row-k,col-k) == player {    // Down-leftward
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
            if self.read(row+k,col-k) == player {    // Up-leftward
                line_len += 1;
            }
            else {
                break
            }
        }
        for k in 1..(row+1).min(self.w-col) {
            if self.read(row-k,col+k) == player {    // Down-rightward
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
            match self.read(i,j) {
                1 => {p1_line.push((i,j)); p2_line.clear()},
                2 => {p1_line.clear(); p2_line.push((i,j))},
                _ => {p1_line.clear(); p2_line.clear()}
            }

            // Highlights by changing 1 --> 10, 2 --> 20. 
            if p1_line.len() >= self.l {
                for (i,j) in p1_line {
                    self.set(i, j, 10*self.read(i, j))
                }
                return 1
            } else if p2_line.len() >= self.l {
                for (i,j) in p2_line {
                    self.set(i, j, 10*self.read(i, j))
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
                output = match self.read(self.h-i-1, j) {
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
                    w if w == protagonist => {          // The protagonist wins
                        self.score += relevance/grid.n_legal_f64();
                    },
                    _ => {                                  // The other player wins
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
struct ThreatMap {
    w: usize,
    h: usize,
    horizontal1: Vec<f64>,
    horizontal2: Vec<f64>,
    fwrd_slash1: Vec<f64>,
    fwrd_slash2: Vec<f64>,
    back_slash1: Vec<f64>,
    back_slash2: Vec<f64>,
}

impl ThreatMap {
    // Contains 6 grids that store the threat level of empty disc positions on the grid .
    // The grids are indexed by threat shape: '-', '/', '\'; and player: 1, 2.  
    // All grid positions start with threat level 1.
    // Non-empty grid positions have have threat level 0.

    fn new(grid: &Grid) -> Self {
        let w = grid.w;
        let h = grid.h;
        
        ThreatMap { w, h, horizontal1: vec![1.; w*h], horizontal2: vec![1.; w*h],
                          fwrd_slash1: vec![1.; w*h], fwrd_slash2: vec![1.; w*h],
                          back_slash1: vec![1.; w*h], back_slash2: vec![1.; w*h]
        }
    }


    fn read(&self, i: usize, j: usize, threat_shape: char, player: u8) -> f64 {
        match threat_shape {
            '-' => {
                match player {
                    1 => return self.horizontal1[i*self.w + j],
                    2 => return self.horizontal2[i*self.w + j],
                    _ => panic!()
                }
            }
            '/' => {
                match player {
                    1 => return self.fwrd_slash1[i*self.w + j],
                    2 => return self.fwrd_slash2[i*self.w + j],
                    _ => panic!()
                }
            }
            '\\' => {
                match player {
                    1 => return self.back_slash1[i*self.w + j],
                    2 => return self.back_slash2[i*self.w + j],
                    _ => panic!()
                }
            }
            _ => panic!()
        }
    }

    fn increment(&mut self, i: usize, j: usize, threat_shape: char, player: u8) {
        if (0..self.h).contains(&i) && (0..self.w).contains(&j) {
            match threat_shape {
                '-' => {
                    match player {
                        1 => self.horizontal1[i*self.w + j] += 1.,
                        2 => self.horizontal2[i*self.w + j] += 1.,
                        _ => panic!()
                    }
                }
                '/' => {
                    match player {
                        1 => self.fwrd_slash1[i*self.w + j] += 1.,
                        2 => self.fwrd_slash2[i*self.w + j] += 1.,
                        _ => panic!()
                    }
                }
                '\\' => {
                    match player {
                        1 => self.back_slash1[i*self.w + j] += 1.,
                        2 => self.back_slash2[i*self.w + j] += 1.,
                        _ => panic!()
                    }
                }
                _ => panic!()
            }
        }
    }
    fn nullify(&mut self, i: usize, j: usize, threat_shape: char, player: u8) {
        if (0..self.h).contains(&i) && (0..self.w).contains(&j) {
            match threat_shape {
                '-' => {
                    match player {
                        1 => self.horizontal1[i*self.w + j] = 0.,
                        2 => self.horizontal2[i*self.w + j] = 0.,
                        _ => panic!()
                    }
                }
                '/' => {
                    match player {
                        1 => self.fwrd_slash1[i*self.w + j] = 0.,
                        2 => self.fwrd_slash2[i*self.w + j] = 0.,
                        _ => panic!()
                    }
                }
                '\\' => {
                    match player {
                        1 => self.back_slash1[i*self.w + j] = 0.,
                        2 => self.back_slash2[i*self.w + j] = 0.,
                        _ => panic!()
                    }
                }
                _ => panic!()
            }
        }
    }

    // Update the threatmap after player plays in (i,j)
    fn update_with(&mut self, row: usize, col: usize, grid: &Grid) {
        let player = grid.read(row, col);
        
        // Horizontal: -
        self.nullify(row, col, '-', player);

        let mut enclosure_extent_right = 0;
        for k in 1..=grid.l {                               // Rightward
            if k == self.w-col {
                enclosure_extent_right = k-1;
                break;                                  // Stops if it hits the wall
            }
            match grid.read(row,col+k) {
                0 => {
                    if k != grid.l {
                        self.increment(row, col+k, '-', player);
                    }
                },
                disc if disc == player => {
                    enclosure_extent_right = k-1
                },
                disc if disc == 3-player => break,
                _ => panic!()
            }
        }
        let mut enclosure_extent_left = 0;
        for k in 1..=grid.l {                               // Leftward
            if k == col+1 {
                enclosure_extent_left = k-1;
                break;                                  // Stops if it hits the wall
            }
            match grid.read(row,col-k) {
                0 => {
                    if k != grid.l {
                        self.increment(row, col-k, '-', player);
                    }
                },
                disc if disc == player => {
                    enclosure_extent_left = k-1
                },
                disc if disc == 3-player => break,
                _ => panic!()
            } 
        }
        // Nullify enclosed disc positions for the enemy
        for j in col-enclosure_extent_left..=col+enclosure_extent_right {
            self.nullify(row, j, '-', 3-player);
        }
        

        // Diagonal: /
        self.nullify(row, col, '/', 1);
        self.nullify(row, col, '/', 2);

        let mut enclosure_extent_up_right = 0;
        for k in 1..=grid.l {                               // Up-rightward
            if k == (self.h-row).min(self.w-col) {
                enclosure_extent_up_right = k-1;
                break;                                  // Stops if it hits the wall
            }
            match grid.read(row+k,col+k) {
                0 => {
                    if k != grid.l {
                        self.increment(row+k, col+k, '/', player);
                    }
                },
                disc if disc == player => {
                    enclosure_extent_up_right = k-1
                },
                disc if disc == 3-player => break,
                _ => panic!()
            }
        }
        // Nullify enclosed disc positions for the enemy
        for k in 1..=enclosure_extent_up_right {
            self.nullify(row+k, col+k, '/', 3-player);
        }

        let mut enclosure_extent_down_left = 0;
        for k in 1..=(row+1).min(col+1) {                   // Down-leftward
            if k == (row+1).min(col+1) {
                enclosure_extent_down_left = k-1;
                break;                                  // Stops if it hits the wall
            }
            match grid.read(row-k,col-k) {
                0 => {
                    if k != grid.l {
                        self.increment(row-k, col-k, '/', player);
                    }
                },
                disc if disc == player => {
                    enclosure_extent_down_left = k-1
                },
                disc if disc == 3-player => break,
                _ => panic!()
            }
        }
        // Nullify enclosed disc positions for the enemy
        for k in 1..=enclosure_extent_down_left {
            self.nullify(row-k, col-k, '/', 3-player);
        }
        
        
        // Diagonal: \
        self.nullify(row, col, '\\', 1);
        self.nullify(row, col, '\\', 2);

        let mut enclosure_extent_up_left = 0;
        for k in 1..=grid.l {                               // Up-leftward
            if k == (self.h-row).min(col+1) {
                enclosure_extent_up_left = k-1;
                break;                                  // Stops if it hits the wall
            }
            match grid.read(row+k,col-k) {   
                0 => {
                    if k != grid.l {
                        self.increment(row+k, col-k, '\\', player);
                    }
                },
                disc if disc == player => {
                    enclosure_extent_up_left = k-1
                },
                disc if disc == 3-player => break,
                _ => panic!()
            } 
        }
        // Nullify enclosed disc positions for the enemy
        for k in 1..=enclosure_extent_up_left {
            self.nullify(row+k, col-k, '\\', 3-player);
        }

        let mut enclosure_extent_down_right = 0;
        for k in 1..=grid.l {                               // Down-rightward
            if k == (row+1).min(self.w-col) {
                enclosure_extent_down_right = k-1;
                break;                                  // Stops if it hits the wall
            }
            match grid.read(row-k,col+k) {
                0 => {
                    if k != grid.l {
                        self.increment(row-k, col+k, '\\', player);
                    }
                },
                disc if disc == player => {
                    enclosure_extent_down_right = k-1
                },
                disc if disc == 3-player => break,
                _ => panic!()
            }
        }
        // Nullify enclosed disc positions for the enemy
        for k in 1..=enclosure_extent_down_right {
            self.nullify(row+k, col+k, '\\', 3-player);
        }
    }
}



#[derive(Clone)]
pub struct Node {
    // Structure used for minmax exploration
    grid: Grid,
    threat_map: ThreatMap,
}

impl Node {
    fn new(grid: Grid) -> Self {
        let threat_map = ThreatMap::new(&grid);
        
        Node {grid, threat_map}
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

    
    
    fn update_threat_map(&mut self, row: usize, col: usize) {
        self.threat_map.update_with(row, col, &self.grid);
    }
    // Horizontal and diagonal threats on rows with prefered parity.
    // Player 1 wants threats in even rows, player 2 in odd rows. The bottom row is row 0.
    // Only works if self.h is even.
    fn heuristic(&self, protagonist: u8) -> f64 {
        if self.grid.h%2 == 1 {
            panic!();
        }
        let mut score = 0.;

        for i in 0..self.grid.h {

            let correct_parity_disc = (i%2 + 1) as u8;

            let sign = match correct_parity_disc {
                player if player == protagonist => 1.,
                _ => -1.
            };

            let mut row_score = 0.;
            
            for j in 0..self.grid.w {
                for threat_shape in ['-', '/', '\\'] {
                    row_score += self.threat_map.read(i, j, threat_shape, correct_parity_disc).powi(2)
                }
            }
            
            // Lower threats are worth more 
            score += sign*row_score/(1.+i as f64)
        }
        
        return score
    }
    fn get_value_alpha_beta(&mut self, depth: u8, protagonist: u8, row: usize, col: usize, 
                            parent_alpha: f64, parent_beta: f64, transp_table: &mut HashMap<u64, (f64, i8)>) -> f64 {
        // Get the value of this node from the values of its children recursively
        // 
        // protagonist denotes wich player the analysis is done for
        // (row,col) is the position where the last disc was placed on the grid 
        // Transposition table for alpha-beta pruning minmax search.
        // 
        // Second tuple entry in transposition table is the value type:
        //      0 means exact value,
        //      -1 means alpha value,
        //      +1 means beta value.
        
        let mut alpha = parent_alpha;
        let mut beta = parent_beta;

        // Get cached value if this state has been seen before.
        let state_id = calculate_hash(&self.grid);
        match transp_table.get(&state_id) {
            Some((stored_value, stored_type))  => {
                match stored_type {
                    -1 => alpha = alpha.max(*stored_value),         // Alpha value
                    0 => return *stored_value,                      // Exact value
                    1 => beta = beta.max(*stored_value),            // Beta value
                    _ => panic!()
                }
                if alpha >= beta {
                    return *stored_value
                }
            },
            _ => ()
        }

        let mut value_type = 0;

        let mut value: f64;
        match self.grid.win_fast(row, col) {
            0 => {                                  // No one wins

                if depth == 0 {                                     // Depth limit reached
                    value = self.heuristic(protagonist);      
                }      

                else if self.grid.n_legal_f64() == 0. {             // Game over (draw)
                    value = 0.
                }
                
                else if self.grid.player_to_move() == protagonist {      // The protagonist's turn
                    
                    value = f64::NEG_INFINITY;                             
                    
                    // Clone the grid and try all possible moves.
                    for ([row, col], mut child) in self.create_children(){
                        child.update_threat_map(row, col);
                        let child_value = child.get_value_alpha_beta(depth-1, protagonist, row, col, 
                                                                          alpha, beta, transp_table);
                        
                        // Keep the maximal value
                        value = value.max(child_value);
                        alpha = alpha.max(value);

                        if beta <= alpha {
                            value_type = -1;                // Store value type: alpha
                            break                           // Beta prune
                        }
                    }
                }
                else {                                                  // The other player's turn
                    value = f64::INFINITY;

                    // Clone the grid and try all possible moves.
                    for ([row, col], mut child) in self.create_children() {
                        child.update_threat_map(row, col);
                        let child_value = child.get_value_alpha_beta(depth-1, protagonist, row, col, 
                                                                          alpha, beta, transp_table);
                        
                        // Keep the minimal value
                        value = value.min(child_value);
                        beta = beta.min(value);

                        if beta <= alpha {
                            value_type = 1;                 // Store value type: beta
                            break                           // Alpha prune
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

        transp_table.insert(state_id, (value, value_type));
        return value
    }
}

pub fn analyze_alphabeta(grid: Grid, protagonist: u8, depth: u8) -> (usize, f64) {
    // Will play the move with the highest value 

    let mut best_col = grid.width();  // This is an illegal move but should always be overridden.
    
    let root_node = Node::new(grid);

    let mut transp_table: HashMap<u64, (f64, i8)> = HashMap::new();
    
    
    let mut best_value = f64::NEG_INFINITY;
    let mut best_immediate_value = f64::NEG_INFINITY;
    for ([row, col], mut child) in root_node.create_children() {
        child.update_threat_map(row, col);

        let child_value = child.get_value_alpha_beta(depth-1, protagonist, row, col, 
                                                          f64::NEG_INFINITY, f64::INFINITY,
                                                          &mut transp_table);
        let child_immediate_value = child.heuristic(protagonist);

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