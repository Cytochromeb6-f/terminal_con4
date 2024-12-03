use std::{fmt, io, time::Instant};

use terminal_con4::{Grid, analyze_minmax, analyze_bfs_mt};

use rand::seq::SliceRandom;

// Change this to true if there are display issues
const NEVER_CLEAR: bool = false;

fn clear_lines() {
    // Clears the terminal. Should work for windows

    if NEVER_CLEAR {
        return
    }
    
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

// Requests a single character from terminal input
fn input_char() -> Result<char, std::char::ParseCharError> {
    let mut input: String = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let len = input.len();
    let str = &input[len-3..len-2];
    
    str.parse::<char>()
}

// Requests an unsigned integer from terminal input
fn input_usize() -> Result<usize, std::num::ParseIntError> {
    let mut input: String = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let len = input.len();
    let str = &input[..len-2];
    
    str.parse::<usize>()
}


// A game where both sides are controlled by terminal input.
// Used for games with two human players
fn custom_game(keep_history: bool, l: usize, w: usize, h: usize) {

    let mut grid = Grid::new(l, w, h);

    while grid.turn() < (w*h) as u8 {
        if !keep_history {
            clear_lines()
        }
        println!("{grid}");

        match input_usize() {
            Ok(col) if col < grid.width() => {
                grid.play(col);
            },
            _ => continue
        }

        match grid.win_highlight() {
            win if win == 1 => {
                if !keep_history {clear_lines()}
                println!("\no won after {} turns!", grid.turn());
                println!("{}", grid);
                return
            },
            win if win == 2 => {
                if !keep_history {clear_lines()}
                println!("\nx won after {} turns!", grid.turn());
                println!("{}", grid);
                return
            }
            _ => ()
        };
    }
    println!("draw");
    println!("{}", grid);
    
}

// A game where one player is controlled by user input and the other by the computer.
fn adversarial_game(keep_history: bool, l: usize, w: usize, h: usize, cpu_player: u8, mut depth: u8, adaptive_depth: bool) {
    // cpu_player=1,2 specifies if the computer plays first or second

    // depth specifies how many layers of subsequent moves the computer will take into account

    // If adaptive_depth is true, then the depth will increase as the tree of possible moves
    // shinks over the course of the game

    let mut grid = Grid::new(l, w, h);
    // for col in [1,2,1,5,1,1,1,3, 1] {
    //     grid.play(col);
    // }

    while grid.turn() < (w*h) as u8 {
        if !keep_history {
            clear_lines()
        }
        println!("{grid}");
        
        if grid.player_to_move() == cpu_player {
            println!("Analyzing with depth = {depth}");
            let now = Instant::now();
            let (col, value) = analyze_minmax(grid.clone(), cpu_player, depth);
            let calc_time = now.elapsed().as_secs_f32();
            println!("The computer played in column {} (value: {:.4}) after {} seconds", col, value, calc_time);         
            grid.play(col);
            
            if adaptive_depth {
                // Increase the calculation depth if it the analysis took less than 1 second.
                if calc_time < 1. {
                    if calc_time > 0.1 {depth += 1}
                    else               {depth += 2}
                }
            }
            
        } else {
            match input_usize() {
                Ok(col) if col < grid.width() => {
                    grid.play(col);
                },
                _ => continue
            }
        }

        match grid.win_highlight() {
            win if win == cpu_player => {
                if !keep_history {clear_lines()}

                println!("\nThe computer won after {} turns!", grid.turn());
                println!("{}", grid);
                return
            },
            win if win == (3 - cpu_player) => {                   // 3-1 = 2, 3-2 = 1
                if !keep_history {clear_lines()}
                println!("\nYou won after {} turns!", grid.turn());
                println!("{}", grid);
                return
            }
            _ => ()
        };
    }
    println!("DRAW");
    println!("{}", grid);
    
}

fn computer_vs_itself(mut d_new: u8, mut d_old: u8, ramp_new: bool, ramp_old: bool, new_first: bool) {
    
    let l = 4;
    let w = 7;
    let h = 6;

    let mut grid = Grid::new(l, w, h);
    
    // for col in [2,3,3,2,3,3] {
    //     grid.play(col);
    // }

    let new_player;
    let new_disc;
    let old_disc;
    match new_first {
        true => {
            new_player = 1;
            new_disc = 'o';
            old_disc = 'x';
        },
        false => {
            new_player = 2;
            new_disc = 'x';
            old_disc = 'o';
        }
    };


    let now = Instant::now();
    while grid.turn() < (w*h) as u8 {
        println!("{grid}");
        
        if grid.player_to_move() == new_player {
            let now = Instant::now();

            // let legal = grid.legal_moves();
            // let col = *legal.choose(&mut rand::thread_rng()).unwrap();
            // let value = 0;
            
            let (col, value) = analyze_minmax(grid.clone(), new_player, d_new);
            
            let calc_time = now.elapsed().as_secs_f32();
            println!(
                "New algorithm ({}) analyzed with depth = {} and played in column {} (value: {:.4}) after {} seconds",
                new_disc, d_new, col, value, calc_time
            );
                   
            grid.play(col);
            if ramp_new {
                if calc_time < 1. {     // Increase the calculation depth if it takes less than 1 second
                    if calc_time > 0.1 {d_new += 1}
                    else               {d_new += 2}
                }
            }
        } else {
            let now = Instant::now();
            
            // let legal = grid.legal_moves();
            // let col = *legal.choose(&mut rand::thread_rng()).unwrap();
            
            let col = analyze_bfs_mt(grid.clone(), 3-new_player, d_old);
            
            let calc_time = now.elapsed().as_secs_f32();
            println!(
                "Old algorithm ({}) analyzed with depth = {} and played in column {} after {} seconds",
                old_disc, d_old, col, calc_time
            );
            
            grid.play(col);
            if ramp_old {
                if calc_time < 1. {     // Increase the calculation depth if it takes less than 1 second
                    if calc_time > 0.1 {d_old += 1}
                    else               {d_old += 2}
                }
            }
        }
        match grid.win_highlight() {
            win if win == new_player => {
                println!("new algorithm wins after {} turns", grid.turn());
                println!("{}", grid);
                return
            },
            win if win == 3 - new_player => {
                println!("old algorithm wins after {} turns", grid.turn());
                println!("{}", grid);
                return
            }
            _ => ()
        };
    }
    let elapsed = now.elapsed();
    println!("Draw after {} seconds",elapsed.as_secs_f32());
    println!("{}", grid);
    
}

struct Menu {
    current_page: u8,
    keep_history: bool,
    l: usize,
    w: usize,
    h: usize,
    game_mode: i8,
    start_depth: u8,
    adaptive_depth: bool,
}
impl Menu {
    fn new() -> Self {
        // Default settings
        Menu { current_page: 0, keep_history: true, l: 4, w: 7, h: 6, game_mode: 1, start_depth: 9, adaptive_depth: true}
        // Menu { current_page: 0, keep_history: false, l: 4, w: 7, h: 6, game_mode: 1, start_depth: 8, adaptive_depth: true}
    }


    fn begin(&self) {
        match self.game_mode {
            0 => custom_game(self.keep_history, self.l, self.w, self.h),
            -1 => adversarial_game(self.keep_history, self.l, self.w, self.h, 1, self.start_depth, self.adaptive_depth),
            1 => adversarial_game(self.keep_history, self.l, self.w, self.h, 2, self.start_depth, self.adaptive_depth),
            _ => panic!("Invalid game mode")
        }
    }

    fn input_options_str(&self) -> &str {

        match self.current_page {
            0 => return "\n Play: [p]    Edit setup: [s] \
                         \n",
            
            1 if (self.game_mode == 0) => {
                return "\nToggle keep history: [k]    Set win length: [l]    Set grid dimensions: [w/h]    Exit Setup: [e] \
                        \nSwitch game mode: [m]\n"
            },
            1 if (self.game_mode != 0) => {
                return "\nToggle keep history: [k]    Set win length: [l]    Set grid dimensions: [w/h]    Exit Setup: [e] \
                        \nSwitch game mode:    [m]    Switch start:   [t]    Set initial depth(>1): [d]    Toggle adaptive depth [a]"
            },
            _ => return "\nINVALID PAGE"
        }
    }

    fn run(&mut self) {
        loop {
            clear_lines();
            println!("{self}");                         // Show current settings and input options
            
            match self.current_page {
                // Start screen
                0 => match input_char() {
                    Ok('p') => {
                        self.begin();
                        break
                    },
                    Ok('s') => {
                        self.current_page = 1;
                        // self.setup_screen()
                    },
                    _ => ()
                }

                // Setup screen
                1 => match input_char() {
                    Ok('k') => self.keep_history = !self.keep_history,
                    Ok('l') => self.l = match input_usize() {
                        Ok(l) if (l > 0) => l,
                        _ => continue
                    },
                    Ok('w') => self.w = match input_usize() {
                        Ok(w) => w,
                        _ => continue
                    },
                    Ok('h') => self.h = match input_usize() {
                        Ok(h) => h,
                        _ => continue
                    },
                    Ok('e') => {self.current_page = 0},

                    // Cpu settings
                    Ok('m') => self.game_mode =  (self.game_mode + 1) % 2,              // -1,1 --> 0, 0 --> 1
                    Ok('t') => self.game_mode =  -self.game_mode,                       // -1 <--> 1, 0 --> 0
                    Ok('d') => self.start_depth = match input_usize() {
                        Ok(d) if (d > 1) => d as u8,
                        _ => continue
                    },
                    Ok('a') => self.adaptive_depth = !self.adaptive_depth,
                    _ => ()
                }
                _ => ()
            }
        }
    }
        
}
impl fmt::Display for Menu {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::from("Current setup");
        
        // Keep history
        output = format!("{output}\n     Keep history:       {}", match self.keep_history{true => "ON ", false => "OFF"});


        // Game mode
        output = format!("{output}           Game mode:        ");
        match self.game_mode {          
            0  => output = format!("{output}two players"),
            -1 => output = format!("{output}single player, cpu plays first"),
            1  => output = format!("{output}single player, cpu plays second"),
            _  => output = format!("{output}INVALID ({})", self.game_mode)
        }

        // Win condition
        output = format!("{output}\n     Length to win:      {}      ", self.l);
        
        // Start depth
        if self.game_mode != 0 {
            output = format!("{output}       Initial depth:    {}", self.start_depth);
        } 

        // Grid width and height
        output = format!("{output}\n     Grid dimensions:    {} x {} ", self.w, self.h);

        // Depth incrementation
        if self.game_mode != 0 {
            output = format!("{output}        Adaptive depth:   {}", match self.adaptive_depth{true => "ON", false => "OFF"});
        } 
        

        // Input options
        output = format!("{output}\n{}", self.input_options_str());
        write!(f, "{output}")
    }
}


fn main() {

    // TODO: Add alpha-beta pruning
   
    //  11: yes, 12: no |  https://www.helpfulgames.com/subjects/brain-training/connect-four.html

    // computer_vs_itself(8, 8, false, false, true);

    let mut menu = Menu::new();
    'play_again: loop {
        menu.run();

        loop {
            println!("\nPlay again? [y/n]");

            match input_char() {
                Ok('y') => break,
                Ok('n') => break 'play_again,
                _ => ()
            }
        }
    }


}
