use std::rc::{Rc};
use std::cell::{RefCell};
//use std::sync::Arc;
use druid::{Data, Lens};
use crate::data::*;

const CELL_RESET_MASK:usize = 0x1FF; 

#[derive(Clone)]
pub enum CellActor{
    StartValue,
    Resolved,
    Guessed(usize),
}


#[allow(dead_code)]
#[derive(Clone)]
pub enum CellState{
    Solved(usize, CellActor),   // uzize contains the positive bitmask for the resolved. There can be only one 1 in the mask
                                // count_ones == 1  0b001 == 1, 0b0010 == 2 0b01000 = 3 etc
    UnSolved(usize),            // usize contains the bitmask 1 means not yet resolved, 0 means resolved
}

#[derive(Clone )]
pub struct SudokuCell {
    pub value: CellState,
    row:usize,
    col:usize,
}

impl SudokuCell {
    pub fn new(r:usize,c:usize) -> SudokuCell {
        SudokuCell{
           value: CellState::UnSolved(0x1FF),
           row:r,
           col:c,
        }
    }
    pub fn get_value(&self) -> String {
        // dereference the Rc into the CellState
        match self.value {
            CellState::Solved(v,_) => format!("{}",v.trailing_zeros() + 1 ),
            CellState::UnSolved(_) => String::from("-"),
        }
    }
    pub fn as_string(&self) -> String {
        // dereference the Rc into the CellState
        match self.value {
            CellState::Solved(v,_) => format!("{}",v.trailing_zeros() + 1 ),
            CellState::UnSolved(_) => String::from("-"),
        }
    }
    fn get_pos(&self) -> (usize,usize) {
        (self.row +1, self.col +1)
    }
    /**
     *  If this cell is resolved return the inverse of the bitmask. So if the value is 4
     *  the returned value is &(1<<3) = 0xFFFFF7 
     */
    pub fn get_resolved_mask(&self) -> usize {
        // a 1 bit in the mask indicate a resolved cell, one bits are unresolved cells
        match self.value {
            CellState::Solved(v,_) => v, 
            _                      => 0,
        }
    }
    /**
     *  If this cell is resolved return the bitmask. So if the value is 4 the returned value is &(1<<3) = 8 
     *  If this cell is NOT resolved return the possibities for this cell. Each 1 in the mask is a possible value
     */
    pub fn get_unresolved_mask(&self) -> usize {
        match self.value {
            CellState::Solved(v,_) =>  v, 
            CellState::UnSolved(n) =>  n, 
        }
    }
    // substract bits in in the possible bit masks. If current cell is now resolved return true
    pub fn substract(&mut self, mask:usize) -> bool  {
        // substract the mask from the bits in this cell. If only one bit left, mark as solved
        let result =
        match self.value {
            CellState::Solved(_v,_) => false,
            CellState::UnSolved(n) => {
                let inverted = !mask;
                let new_mask = n & inverted;
                if new_mask.count_ones() == 1 {
                    self.value = CellState::Solved( new_mask, CellActor::Resolved);  
                    println!("Solved in substract cell {:?} value {} my mask {:09b} incoming mask {:09b} ressulting mask {:09b}",  self.get_pos(), self.as_string(), n ,mask, new_mask) ;
                    true
                } else { 
                    if n != new_mask {
                        self.value = CellState::UnSolved( new_mask);
                    }
                    false
                }
            },
        };
        result
    }
    pub fn set_init_value(&mut self, v:usize)  {
        self.value = CellState::Solved( 1 << (v - 1) , CellActor::StartValue); 
    } 
    pub fn reset(&mut self)  {
        self.value = CellState::UnSolved(CELL_RESET_MASK); 
    } 

    /**
     * Set the solved value, but only if the current mask is equal to the incoming value
     * return true if the value is set (masks are equal, false if not set)
     */
    pub fn set_solved_value(&mut self, mask:usize) -> bool  {
        match self.value {
            CellState::UnSolved(n) => {
                if (n & mask)  == mask {
                    self.value = CellState::Solved(mask, CellActor::Resolved);
                    println!("Resolved cell {:?} value {}", self.get_pos(), self.as_string());
                    return true;
                }
                false
            }
            _ => false,
        }
    } 


    // if solved return 1 else return zero. If intitial is true return only begin situation, else the current situation
    pub fn count_solved(&self, intitial:bool ) -> usize {
        match &self.value {
            CellState::Solved(_,actor) => 
                match actor {
                    CellActor::StartValue => 1 ,
                    _ if intitial => 0,
                    _  => 1,
                } 
            _ => 0
        }
    }
} // cell


/**
 * The Rc is easy clonable and makes more than one reference possible to the sudoku cell
 * The RefCell is for inner mutability
 */
#[derive(Clone, Lens, Data )]
pub struct RcSudokuCell {
    cell: Rc<RefCell<SudokuCell>>
}
impl RcSudokuCell {
    pub fn new(refcell:&Rc<RefCell<SudokuCell>>) -> RcSudokuCell {
        RcSudokuCell{
           cell : refcell.clone()
        }
    }
    pub fn get_value(&self) -> String {
        //From the refcell borrow the pointer to the sudoku cell
        self.cell.borrow().get_value()
    } 
    pub fn set_init_value(&self, v:usize)  {
        // dereference the Rc into the CellState
        self.cell.borrow_mut().set_init_value(v);
    } 
    pub fn set_solved_value(&self, mask:usize ) -> bool  {
        // dereference the Rc into the CellState
        self.cell.borrow_mut().set_solved_value(mask)
    } 
    pub fn get_state(&self) -> CellState {
        // dereference the Rc 
        self.cell.borrow().value.clone()
    }
    pub fn get_resolved_mask(&self) -> usize {
        self.cell.borrow().get_resolved_mask()
    }
    pub fn get_unresolved_mask(&self) -> usize {
        self.cell.borrow().get_unresolved_mask()
    }
    pub fn substract(&self, mask:usize) -> bool  {
        self.cell.borrow_mut().substract(mask)
    }
    pub fn count_solved(&self, intitial:bool ) -> usize{
        self.cell.borrow().count_solved(intitial )
    }
    pub fn reset(&self)  {
        self.cell.borrow_mut().reset();
    }
    pub fn get_pos(&self) -> (usize,usize) {
        self.cell.borrow().get_pos() 
    }
    pub fn as_string(&self) -> String {
        self.cell.borrow().as_string() 
    }
}
/**
 * AllCells is the owner of the sudoku refcells. All rows, cols or squares  have a copy of the Rc of the RcSudokuCell
*/
#[derive(Clone)]
pub struct AllCells {
    cells:  Vec<RcSudokuCell>,
}

impl AllCells {
    fn new ()-> AllCells {
        let mut cells = vec![];
        for r in 0 .. CELL_ROW  {
            for c in 0 .. CELL_COL  {
                let cell = SudokuCell::new(r, c);
                let refcell = RefCell::new(cell);
                let rccell = Rc::new(refcell);
                cells.push(RcSudokuCell::new(&rccell) );
            }
        }
        AllCells{ cells:cells}
    }
    // note the the self is immutable!
    pub fn reset (&self) {
        for i in 0 .. CELL_ROW  * CELL_COL  {
            self.cells[i].reset();
        }
    }
}

pub trait RowColSquare  {
    fn get_cells(&self) -> &Vec<RcSudokuCell>;
    fn get_id(&self) -> &String;
}

#[derive(Clone, Data, Lens )]
pub struct Row {
    #[data(ignore)]
    pub cells:  Vec<RcSudokuCell>,
    id: String,
}

impl RowColSquare for Row {
    fn get_cells(& self) -> & Vec<RcSudokuCell> {
        return &self.cells;
    }
    fn get_id(&self) -> &String{
        return &self.id;
    }
}


impl Row {
    fn new (i:usize)-> Row {
        Row{ cells:vec![], id:format!("Row: {}", i + 1)  }
    }
    pub fn wire(&mut self, r:usize,  allcells: &AllCells) {
        let startindex = r * CELL_SIZE;
        for n in 0..CELL_SIZE {
            let allcell  = &allcells.cells[startindex + n]; 
            self.cells.push(RcSudokuCell::new( &allcell.cell ));
        }
    }
}

#[derive(Clone, Data)]
pub struct Col {
    #[data(ignore)]
    cells:  Vec<RcSudokuCell>,
    id: String,
}
impl RowColSquare for Col {
    fn get_cells(&self) -> &Vec<RcSudokuCell> {
        return &self.cells;
    }
    fn get_id(&self) -> &String{
        return &self.id;
    }
}

impl Col {
    fn new (i:usize)-> Col {
        Col{ cells:vec![], id:format!("Col: {}", i + 1)  }
    }
    pub fn wire(&mut self, c:usize,  allcells: &AllCells) {
        let startindex = c;
        for n in 0..CELL_SIZE {
            let allcell  = &allcells.cells[startindex +  n*CELL_SIZE]; 
            self.cells.push(RcSudokuCell::new( &allcell.cell ));
        }
    }
}

#[derive(Clone, Data)]
pub struct Square {
    #[data(ignore)]
    cells:  Vec<RcSudokuCell>,
    id: String,

}
impl RowColSquare for Square {
    fn get_cells(&self) -> &Vec<RcSudokuCell> {
        return &self.cells;
    }
    fn get_id(&self) -> &String{
        return &self.id;
    }
}

impl Square {
    fn new (i:usize)-> Square {
        Square{ cells:vec![],  id:format!("Square: {}", i + 1)  }
    }
    pub fn wire(&mut self, r:usize,  c:usize,  allcells: &AllCells) {
        for n in 0..CELL_SIZE {
            let ri =  (r + n/3) * CELL_SIZE;
            let ci =  c + n%3; 
            let alli = ri + ci;
            let allcell  = &allcells.cells[alli]; 
            self.cells.push(RcSudokuCell::new( &allcell.cell ));
        }
    }
}


#[derive(Clone,  Lens )]
pub struct SudokuBoard{
    allcells:AllCells,
    pub rows: Vec<Row>,
    pub cols: Vec<Col>,
    pub squares: Vec<Square>,
}


impl SudokuBoard {
    pub fn new () -> SudokuBoard {
        println!("New board. Board size: {}:{}", CELL_ROW ,CELL_COL);

        SudokuBoard {
            allcells:AllCells::new(),
            rows: { let mut  rws  = vec![];
                    for i in 0 .. CELL_ROW { 
                        rws.push( Row::new(i) ) ;
                    };
                    rws   
                  },
            cols: { let mut cols = vec![];
                    for i in 0 .. CELL_ROW {
                        cols.push(Col::new(i) );
                    };
                    cols  
                  },
            squares: { let mut sq = vec![];
                    for i in 0 .. CELL_ROW {
                        sq.push(Square::new(i));
                    };
                    sq   
                  },
        }        
    }
    pub fn reset(&self){
        self.allcells.reset();
    }

    // replace all dummy rc's to the actual reference
    pub fn wire(& mut self) -> &SudokuBoard {
        for r in 0 ..CELL_ROW  {
            self.rows[r].wire(r, &self.allcells);
        }
        for c in 0 ..CELL_COL  {
            self.cols[c].wire(c, &self.allcells);
        }
        self.squares[0].wire(0,0, &self.allcells);
        self.squares[1].wire(0,3, &self.allcells);
        self.squares[2].wire(0,6, &self.allcells);
        self.squares[3].wire(3,0, &self.allcells);
        self.squares[4].wire(3,3, &self.allcells);
        self.squares[5].wire(3,6, &self.allcells);
        self.squares[6].wire(6,0, &self.allcells);
        self.squares[7].wire(6,3, &self.allcells);
        self.squares[8].wire(6,6, &self.allcells);
        self
    }

    /*
|   |   |   |
| 7 |   |9  |
|5  |9  |  2|
-------------
|   |1 6|4  |
|461|   |   |
|   |  5|  6|
-------------
| 86| 4 |3  |
|9 2| 1 |  8|
|3  |  8|   |
-------------
*/    
    pub fn init(& self){

        self.init_cell( 1, 1,  7);
        self.init_cell( 1, 7,  9);
        self.init_cell( 2, 0,  5);
        self.init_cell( 2, 3,  9);
        self.init_cell( 2, 8,  2);
        self.init_cell( 3, 3,  1);
        self.init_cell( 3, 5,  6);
        self.init_cell( 3, 6,  4);
        self.init_cell( 4, 0,  4);
        self.init_cell( 4, 1,  6);
        self.init_cell( 4, 2,  1);
        self.init_cell( 5, 5,  5);
        self.init_cell( 5, 8,  6);
        self.init_cell( 6, 1,  8);
        self.init_cell( 6, 2,  6);
        self.init_cell( 6, 4,  4);
        self.init_cell( 6, 6,  3);
        self.init_cell( 7, 0,  9);
        self.init_cell( 7, 2,  2);
        self.init_cell( 7, 4,  1);
        self.init_cell( 7, 8,  8);
        self.init_cell( 8, 0,  3);
        self.init_cell( 8, 5,  8);
    }

    pub fn init_cell( &self, r:usize , c:usize, v:usize){
        let cell = &self.rows[r].cells[c];
        let refcell = &*cell;
        refcell.set_init_value(v);
    }

    pub fn count_solved(&self, initial:bool) -> usize {
        let mut  count = 0;
        for cell in &self.allcells.cells{
            count += cell.count_solved(initial);
        }
        count
    }
    pub fn show( &self){
        for r in 0..CELL_SIZE{
            let row = &self.rows[r];
            for c in 0..CELL_SIZE{
                let cell = &row.cells[c];
                print!(" {}", cell.get_value());
                match c {
                    2 => print!(" |"),
                    5 => print!(" |"),
                    8 => println!(""),
                    _ => ()
                }
            }
            match r {
                2 => println!("-------|-------|---------"),
                5 => println!("-------|-------|---------"),
                _ => ()
            }
        }
    }
    pub fn resolve_step( &self) {
        for r in 0..CELL_ROW {
            solver_next_step( &self.rows[r]);
        }
        for r in 0..CELL_ROW {
            solver_next_step( &self.cols[r]);
        }
        for r in 0..CELL_ROW {
            solver_next_step( &self.squares[r]);
        }
    }
    pub fn check_board( &self) {
        for r in 0..CELL_ROW {
            println!("---------------index {}", r);
            print_layout( &self.rows[r]);
            print_layout( &self.cols[r]);
            print_layout( &self.squares[r]);
        }
    }
}
/********************************************************************************************************** */
/** Solver logica  */

fn print_layout(row_col_square: &dyn RowColSquare) {
    for cell in row_col_square.get_cells() {
        print!(" {:?} - ", cell.get_pos());
    }
    println!("");
}


fn solver_next_step(row_col_square: &dyn RowColSquare) {
    let mut mask = 0;
    // Step 1 get the resolved mask for all cells. A 1 on a bitpos means resolved.
    let cells = row_col_square.get_cells();

    loop {
        let mut changed = false;
        for n in 0..CELL_SIZE {
            mask |= cells[n].get_resolved_mask();
        }
        // the mask does contain for each resolved cell a 1
        // Substract the already resolved values from the array of possible values. 
        // if only one bit is left the function will mark the cell as resolved
        for n in 0..CELL_SIZE {
            changed = changed || cells[n].substract(mask);
        }
        if !changed {break;}
    } ; 

    // count the possible locations  for each value
    let mut option_count  = [0; CELL_SIZE];

    for row in 0..CELL_SIZE {
        let cell = &cells[row];
        let mut mask = cell.get_unresolved_mask();
        for bitpos in 0..CELL_SIZE {
            // if bit is one, there is a possible location found for this value
            if (mask & 1) == 1 {
                option_count[bitpos] += 1
            }
            mask = mask >> 1;
        }
    }
    
    // for each position in the option_count check value 1
    for row in 0..CELL_SIZE {
        if option_count[row] == 1 {
            let mask = 1 << row;
            // again iterate over the cells and find the cell with the current mask
            for c in 0..CELL_SIZE {
                let cell = &cells[c];
                match cell.get_state() {
                    CellState::UnSolved(n) => {
                        if n & mask == mask {
                            println!("Would advice for cell {:?} value {}", cell.get_pos(), row +1);
                            if (7,5) == cell.get_pos() {
                                println!("bingo");
                            }
                            cell.set_solved_value(mask);
                            break;
                        }
                    }
                    _ => (),
                }
            }
        }
    }
     
    // Show the results on the terminal
    print!("{:10} ", row_col_square.get_id());
    for n in 0..CELL_SIZE {
        print!("{} ", option_count[n]);
    }
    println!();

}
/********************************************************************************************************** */


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
            bref.resolve_step();
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

