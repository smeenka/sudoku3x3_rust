//use std::rc::{Rc};
//use std::sync::Arc;
use druid::*;
use std::collections::*;
use crate::sudoku_board::*;
use crate::data::*;

#[derive(Clone, PartialEq,Debug)]
pub enum GameState{
    Select,
    ManualInput,
    Stepping,
    Error,
    Solved,
}



#[derive(Clone,  Lens, Debug )]
pub struct SudokuState{
    step_count:usize, // current step count
    init_count:usize, // initial count of resolved cells
    curr_count:usize, // currentlly resolved cells
    game_state:GameState,
    printRowDetails:bool,
    printCellDetails:bool,
}


impl SudokuState {
    pub fn new () -> SudokuState {
        SudokuState {
            step_count:0,
            init_count:0,
            curr_count:0,
            game_state:GameState::Select,      
            printRowDetails:false,
            printCellDetails:false,
        }
    }
    pub fn reset(&mut self){
        self.step_count = 0;
        self.init_count = 0;
        self.curr_count = 0;      
        self.game_state = GameState::ManualInput;
    }
    pub fn select(&mut self){
        self.game_state = GameState::Select;
    }
    pub fn get_step_count(&self)-> usize { self.step_count }
    pub fn get_init_count(&self)-> usize { self.init_count }
    pub fn get_curr_count(&self)-> usize { self.curr_count }
    pub fn get_state(&self)-> GameState { self.game_state.clone() }

    pub fn do_count( &mut self, board:&SudokuBoard) {
        let counts = board.count_solved();
        self.init_count = counts.0;
        self.curr_count = counts.1;
    }


    pub fn resolve_step( &mut self, board:&SudokuBoard) -> GameState {
        self.step_count += 1;
        board.push();
        for square in board.all_logic_squares() {
            match self.reduce_square(square)  {
                Ok(_)   => self.game_state = GameState::Stepping,
                Err(_)  => self.game_state = GameState::Error,
            }
        }
        self.do_count(board);
        if self.curr_count == CELL_SIZE * CELL_SIZE {
            println!("Bingo!");
            self.game_state = GameState::Solved;
        }
        self.game_state.clone()
    }

    pub fn step_back(&mut self,  board:&SudokuBoard){
        board.pop();
        self.do_count(board);
        if self.step_count > 0 {
            self.step_count -= 1;
            self.game_state = GameState::Stepping;
        } 
    }

/********************************************************************************************************** */
    /** Solver logica  */



    pub fn reduce_square(&self, row_col_square: &dyn RowColSquare) -> Result<usize, String> {
        let cells = row_col_square.get_cells();

        // Step 1: reduce the possible cell values with the already solved ones
        //  get the resolved mask for all cells. A 1 on a bitpos means resolved.
        for this in 0..CELL_SIZE {
            let thiscell =  &cells[this];
            for other in 0..CELL_SIZE {
                if other != this {
                    let othercell =  &cells[other];
                    thiscell.reduce(othercell.get_resolved_mask())?;
                }
            }
        }
        // Step 2 the inverse of step 1
        // in step 1 for each given cell we investigate the possible values
        // in step 2 for each given value investigate the possible cells
        
        let mut possible_cells:Vec<usize> = vec![ 0;CELL_SIZE];   // Step 1 get the resolved mask for all cells. A 1 on a bitpos means resolved.

        for value in 0..CELL_SIZE {
            let value_mask = 1<< value;

            for n in 0.. CELL_SIZE{
                let cell_mask  = 1 << n;
                let cell = &cells[n];
                let mask = cell.get_unresolved_mask();

                if mask & value_mask == value_mask {
                    // ok this value could placed in  tnis cell
                    possible_cells[value] |= cell_mask;
                }
            }
        }
        // step 3 find loners (resolved cells hidden in the wood of unresolved bits
        for n in 0..CELL_SIZE {

            let mask = possible_cells[n];
            if mask.count_ones() == 0 {
                println!("Error: in reduce square {} zero bits left for value {}  ", row_col_square.get_id(), n+1);

                return Err(format!("For value {} no positions anymore",n+1));
            } else
            if mask.count_ones() == 1 {
                let index = mask.trailing_zeros() as usize;
                let cell = &cells[index];
                let value_mask = 1<< n;
                match cell.get_state() {
                    CellState::UnSolved(_) => {
                        println!("Found a loner hidden in  the bush {:?} value {}", cell.get_pos(), n +1);
                        cell.set_solved_value(value_mask);
                    },
                _  => ()
                } 
            } 
        }
    
        // step 4 make a hasmap of the unresolved masks, and count the twins
        let mut overall_twin_mask = 0;

        let mut twin_hash:HashMap<usize,usize> = HashMap::new();   
    
        // for each possible value masks count the amount
        for c in 0..CELL_SIZE {
            let cell = &cells[c];
            let unresolved = cell.get_unresolved_mask();
            let result = 
                    match twin_hash.get(&unresolved){
                        Option::Some(n)   => n + 1,
                        Option::None      => 1,
                        };
            twin_hash.insert(unresolved, result);
        }
        // now iterate over the map and find the twins
        for (key, val) in twin_hash.iter() {
            if key.count_ones () == 2 && *val == (2  as usize) {
                println!("Found a twin2 in {} (not yet resolved). Twin   mask {:09b}", row_col_square.get_id(), key  );
                overall_twin_mask |= key;
            }
        }
        if overall_twin_mask > 0{
            println!("Reduce {} due to found twin mask {:09b} ",  row_col_square.get_id(), overall_twin_mask);
            for c in 0..CELL_SIZE {
                let cell = &cells[c];
                cell.reduce(overall_twin_mask);
            }
        }
        // Show the results on the terminal
        if self.printRowDetails {
            print!("{:10} ", row_col_square.get_id());
            for n in 0..CELL_SIZE {
                print!(" {}:{:09b} ", n+1, possible_cells[n] );
            }
            println!();
        }
        /*
        if self.printCellDetails {
            print!("{:10} ", row_col_square.get_id());
            for (key, val) in count_hash.iter() {
                print!("-{:09b}:{:?}-",  key, val);
            }
            println!();
        }
        */
        Ok(0)
    }
    /********************************************************************************************************** */
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

   #[test]
    fn test_get_3x3_board() { 
        let  mut board = SudokuBoard::new();
        let bref = board.wire();
        bref.init();
        bref.show();
    }
    #[test]
    fn test_do_3_steps() { 
        let  mut board = SudokuBoard::new();
        let bref = board.wire();
        bref.init();
        bref.show();
        for _ in 0 .. 3 {
            bref.resolve_step().expect("something wrong");
            bref.show();
        }
    }
    #[test]
    fn test_check_board() { 
        let  mut board = SudokuBoard::new();
        let bref = board.wire();
        bref.init();
        bref.check_board()
    }
}

