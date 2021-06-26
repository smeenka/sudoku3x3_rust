use std::rc::{Rc};
use std::cell::{RefCell};
//use std::sync::Arc;
use druid::{Data, Lens};
use crate::data::*;
//use std::collections::*;

const CELL_RESET_MASK:usize = 0x1FF; 

pub struct SudokuError {
}

#[derive(Clone, PartialEq)]
pub enum CellActor{
    StartValue,
    Resolved,
    Guessed(usize),
}


#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum CellState{
    Solved(usize, CellActor),   // uzize contains the positive bitmask for the resolved. There can be only one 1 in the mask
                                // count_ones == 1  0b001 == 1, 0b0010 == 2 0b01000 = 3 etc
    UnSolved(usize),            // usize contains the bitmask 1 means not yet resolved, 0 means resolved
    Error
}

#[derive(Clone )]
pub struct SudokuCell {
    pub value: CellState,
    row:usize,
    col:usize,
    idx:usize,
    stack:Vec<CellState>,
}

impl SudokuCell {
    pub fn new(r:usize,c:usize) -> SudokuCell {
        SudokuCell{
           value: CellState::UnSolved(CELL_RESET_MASK),
           row:r,
           col:c,
           idx: r*CELL_SIZE +c,
           stack:vec![],
        }
    }
    pub fn get_value(&self) -> String {
        // dereference the Rc into the CellState
        match self.value {
            CellState::Solved(v,_) => format!("{}",v.trailing_zeros() + 1 ),
            _ => String::from("-"),
        }
    }
    pub fn as_string(&self) -> String {
        // dereference the Rc into the CellState
        match self.value {
            CellState::Solved(v,_) => format!("{}",v.trailing_zeros() + 1 ),
            _ => String::from("-"),
        }
    }
    fn get_pos(&self) -> (usize,usize) {
        (self.row +1, self.col +1)
    }
    /**
     *  If this cell is resolved return the the bitmask else 0
     */
    pub fn get_resolved_mask(&self) -> usize {
        // a 1 bit in the mask indicate a resolved cell
        match self.value {
            CellState::Solved(v,_)  => v, 
            _                      => 0,
        }
    }
    /**
     *  If this cell is resolved return the bitmask (with one bit!). 
     *  If this cell is NOT resolved return the possibities for this cell. 
     */
    pub fn get_unresolved_mask(&self) -> usize {
        match self.value {
            CellState::Solved(v,_) =>  v, 
            CellState::UnSolved(n)  =>  n, 
            CellState::Error  =>  CELL_RESET_MASK, 
        }
    }
 
    // If this cell is NOT resolved substract bits in the possible bit masks.
    // If only one bit is left, mark as solved
    // Return a Result 
    pub fn reduce(&mut self, other_mask:usize)  -> Result<usize, String> {
        match self.value {
            CellState::Solved(_v,_) => return Ok(0),
            CellState::UnSolved(my_mask) => {  
                if my_mask != CELL_RESET_MASK &&  (my_mask & other_mask == my_mask) { 
                    Ok(0) 
                } else {
                    let new_mask = my_mask & !other_mask;
                    let nr_bits = new_mask.count_ones(); 
                    if nr_bits == 1 {
                        self.value = CellState::Solved( new_mask, CellActor::Resolved);  
                        println!("Solved in reduce cell {:?} value {} my mask {:09b}  other {:09b}",  self.get_pos(), self.as_string(), my_mask, other_mask  );
                    } else  if my_mask != new_mask {
                        self.value = CellState::UnSolved( new_mask)
                    };
                    if nr_bits == 0 {
                        let message = format!("Error: in reduce zero bits left for {:?}  incoming mask {:09b}", self.get_pos(), other_mask);
                        println!("{}", message);
                        self.value = CellState::Error;
                        Err( message )
                    }else {
                        Ok(0)
                    }
                }
            },
            CellState::Error => Err("Cell in Error state".to_string())
        }        
    }
    pub fn set_init_value(&mut self, v:usize)  {
        self.value = CellState::Solved( 1 << (v - 1) , CellActor::StartValue); 
    } 
    pub fn reset(&mut self)  {
        self.value = CellState::UnSolved(CELL_RESET_MASK); 
        self.stack = vec![];
    } 
    /**
     * Set the solved value, but only if the current mask is equal to the incoming value
     * return true if the value is set (masks are equal, false if not set)
     */
    pub fn set_solved_value(&mut self, mask:usize) -> bool  {
        match self.value {
            CellState::UnSolved(n)  => {
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
    // return (0,0) if cell is not resolved
    // return (1,1) if cell is resolved, but in initial state
    // return (0,1) if the cell is resolved due to stepping
    pub fn count_solved(&self) -> (usize,usize) {
        match &self.value {
            CellState::Solved(_,actor) => {
                match  actor {
                    CellActor::StartValue  => (1, 1),
                    _ => ( 0, 1 )
                }
            },
            _ => (0 , 0)
        }
    }

    fn push(&mut self){
        self.stack.push(self.value.clone());

    }
    fn pop(&mut self){
        if self.stack.len() > 0 {
            match self.stack.pop() {
                Some(v) => self.value = v,
                None    => ()
            }
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
 
    pub fn get_state(&self) -> CellState {
        // dereference the Rc 
        self.cell.borrow().value.clone()
    }
    pub fn reduce(&self, other: usize) -> Result<usize, String>   {
        self.cell.borrow_mut().reduce(other)
    }
    pub fn get_resolved_mask(&self) -> usize {
        self.cell.borrow().get_resolved_mask()
    }
    pub fn get_unresolved_mask(&self) -> usize {
        self.cell.borrow().get_unresolved_mask()
    }
    pub fn set_solved_value(&self, mask:usize ) -> bool  {
        // dereference the Rc into the CellState
        self.cell.borrow_mut().set_solved_value(mask)
    } 

    pub fn count_solved(&self) -> (usize,usize) {
        self.cell.borrow().count_solved()
    }

    pub fn reset(&self)  {
        self.cell.borrow_mut().reset();
    }
    pub fn pop(&self)  {
        self.cell.borrow_mut().pop();
    }
    pub fn push(&self)  {
        self.cell.borrow_mut().push();
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
    fn push(&self){
        for i in 0 .. CELL_ROW  * CELL_COL  {
            self.cells[i].push();
        }
    }
    fn pop(&self){
        for i in 0 .. CELL_ROW  * CELL_COL  {
            self.cells[i].pop();
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
    pub fn reset(&self) {         self.allcells.reset();    }
    pub fn pop(  &self) {         self.allcells.pop();     }
    pub fn push( &self) {         self.allcells.push();     }

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

    pub fn all_logic_squares(&self) -> Vec<&dyn RowColSquare> {
        let mut result:Vec<&dyn RowColSquare> = vec![];
        for row in &self.rows {
            result.push(row);
        }
        for col in &self.cols {
            result.push(col);
        }
        for square in &self.squares {
            result.push(square);
        }
        result
    }
    pub fn count_solved(&self) -> (usize, usize)  {
        let mut init_count = 0;
        let mut curr_count = 0;
        for cell in &self.allcells.cells{
            let counts = cell.count_solved();
            init_count += counts.0;
            curr_count += counts.1;
        }
        (init_count, curr_count)
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
    pub fn check_board( &self) {
        for r in 0..CELL_ROW {
            println!("---------------index {}", r);
            self.print_layout( &self.rows[r]);
            self.print_layout( &self.cols[r]);
            self.print_layout( &self.squares[r]);
        }
    }
    pub fn print_layout(&self, row_col_square: &dyn RowColSquare) {
        for cell in row_col_square.get_cells() {
            print!(" {:?} - ", cell.get_pos());
        }
        println!("");
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
        self.allcells.push();
    }
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

