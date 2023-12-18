use std::{fmt, io, time::Instant};

use terminal_con4::{Grid, analyze_bfs_mt};

fn custom_game(l: usize, w: usize, h: usize) {

    let mut grid = Grid::new(l, w, h);


    let mut input = String::new();

    while grid.turn() < (w*h) as u8 {
        println!("{grid}");

        io::stdin().read_line(&mut input).expect("Failed to read line");
        
        let len = input.len();
        let col_str = &input[len-3..len-2];
        let col_res = col_str.parse::<usize>();

        if col_res.is_ok(){
            let col = col_res.unwrap();
            if (0..grid.width()).contains(&col) {
                grid.play(col);
            }
        }

        match grid.win_highlight() {
            win if win == 1 => {
                println!("o wins after {} turns", grid.turn());
                println!("{}", grid);
                return
            },
            win if win == 2 => {
                println!("x wins after {} turns", grid.turn());
                println!("{}", grid);
                return
            }
            _ => ()
        };
    }
    println!("draw");
    println!("{}", grid);
    
}

#[allow(dead_code)]
fn adversarial_game(l: usize, w: usize, h: usize, cpu_is_first: bool, mut depth: u8) {

    let mut grid = Grid::new(l, w, h);

    let cpu_player = match cpu_is_first{
        true => 1,
        false => 2
    };

    let mut input: String = String::new();
    

    while grid.turn() < (w*h) as u8 {
        println!("{grid}");
        
        if grid.turn()%2 == cpu_player-1 {
            println!("Analyzing with depth = {depth}");
            let now = Instant::now();
            let col = analyze_bfs_mt(grid.clone(), cpu_player, depth);
            let calc_time = now.elapsed().as_secs_f32();
            println!("The computer played in column {} after {} seconds", col, calc_time);         
            grid.play(col);
            
            if calc_time < 1. {     // Increase the calculation depth if it takes less than 1 second
                if calc_time > 0.1 {depth += 1}
                else               {depth += 2}
            }
            
        } else {
            io::stdin().read_line(&mut input).expect("Failed to read line");
            let len = input.len();

            let col_str = &input[len-3..len-2]; 
            let col_res = col_str.parse::<usize>();
            
            if col_res.is_ok(){
                let col = col_res.unwrap();
                if (0..grid.width()).contains(&col) {
                    grid.play(col);
                }
            }

        }
        match grid.win_highlight() {
            win if win == 1 => {
                println!("o wins after {} turns", grid.turn());
                println!("{}", grid);
                return
            },
            win if win == 2 => {
                println!("x wins after {} turns", grid.turn());
                println!("{}", grid);
                return
            }
            _ => ()
        };
    }
    println!("draw");
    println!("{}", grid);
    
}

fn input_char() -> Result<char, std::char::ParseCharError> {
    let mut input: String = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let len = input.len();
    let str = &input[len-3..len-2];
    
    str.parse::<char>()
}

fn input_u8() -> Result<u8, std::num::ParseIntError> {
    let mut input: String = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    let len = input.len();
    let str = &input[..len-2];
    
    str.parse::<u8>()
}

struct Menu {
    l: usize,
    w: usize,
    h: usize,
    game_mode: u8,
    start_depth: u8,

}
impl Menu {
    fn new() -> Self {
        Menu { l: 4, w: 7, h: 6, game_mode: 0, start_depth: 6}
    }


    fn play(&self) {
        match self.game_mode {
            0 => adversarial_game(self.l, self.w, self.h, false, self.start_depth),
            1 => adversarial_game(self.l, self.w, self.h, true, self.start_depth),
            2 => custom_game(self.l, self.w, self.h),
            _ => panic!("Invalid game mode")
        }
    }

    fn start(&mut self) {
        loop {
            println!("{self}");                         // Show current settings
            println!("\n Play: [p]    Edit Setup: [s]");
            match input_char() {
                Ok('p') => {
                    self.play();
                    break
                },
                Ok('s') => {
                    self.setup()
                },
                _ => ()
            }
        }
    }

    fn setup(&mut self) {
        loop {
            println!("{self}");                         // Show current settings
            println!("\n Cycle game mode: [m]    Set win length [l]    Set grid size: [w] [h]    Set cpu start depth: [d]    Exit Setup: [e]");
            match input_char() {
                Ok('m') => self.game_mode =  (self.game_mode + 1) % 3,
                Ok('d') => self.start_depth = match input_u8() {
                  Ok(d) => d,
                  _ => continue
                },
                Ok('l') => self.l = match input_u8() {
                  Ok(l) => l as usize,
                  _ => continue
                },
                Ok('w') => self.w = match input_u8() {
                  Ok(w) => w as usize,
                  _ => continue
                },
                Ok('h') => self.h = match input_u8() {
                  Ok(h) => h as usize,
                  _ => continue
                },
                Ok('e') => break,
                _ => ()
            }
        }
    }
        
}
impl fmt::Display for Menu {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::from("\nCurrent setup ");
        

        

        // Game mode
        output = format!("{output} \n     Game mode: ");
        match self.game_mode {          
            0 => output = format!("{output}single player, you start"),
            1 => output = format!("{output}single player, cpu starts"),
            2 => output = format!("{output}two players"),
            _ => output = format!("INVALID")
        }

        // Grid dimensions and win condition
        output = format!("{output} \n     Connect N: {}", self.l);
        output = format!("{output} \n     Grid size: {} x {}", self.w, self.h);

        // Cpu settings
        output = format!("{output} \n Cpu start depth: {}", self.start_depth);
        // output = format!("{output} \n     Grid size: {}x{}", self.w, self.h);

        write!(f, "{output}")
    }
}


fn main() {

    println!("\nWelcome to connect 4");

    let mut menu = Menu::new();
    menu.start();
}
