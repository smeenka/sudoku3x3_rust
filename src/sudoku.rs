use std::rc::{Rc};
use std::cell::{RefCell};
//use std::sync::Arc;
use druid::{Data, Lens};
use crate::data::*;

const CELL_RESET_MASK:usize = 0x1FF; 

pub struct SudokuError {

}

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
    Twin2(usize, usize),        // this cell has a twin with the same mask, first parameter is the mask, second parameter idx of the twin
    //Twin3(usize, (RcSudokuCell,RcSudokuCell))                            
    UnSolved(usize),            // usize contains the bitmask 1 means not yet resolved, 0 means resolved
}

#[derive(Clone )]
pub struct SudokuCell {
    pub value: CellState,
    row:usize,
    col:usize,
    idx:usize,
}

impl SudokuCell {
    pub fn new(r:usize,c:usize) -> SudokuCell {
        SudokuCell{
           value: CellState::UnSolved(0x1FF),
           row:r,
           col:c,
           idx: r*CELL_SIZE +c,
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

    // if other cell is this cell do nothing
    // if this cell is resolved or a twin do nothing
    // if other cell is resolved, reduce the unsolved mask with the resolved value of the other
    // if other is not resolved and this one is not resolved check if we are a twin
    // if I am a twin and the other is a twin and we are twins, return our mask, else return CELL_RESET_MASK
    pub fn reduce(&mut self, other:&mut SudokuCell)  -> usize {
        let mut twin_mask = CELL_RESET_MASK;

        if self.idx == other.idx { return twin_mask }

        // substract the mask from the bits in this cell. If only one bit left, mark as solved
        match self.value {
            CellState::Solved(_v,_) => return twin_mask,
            CellState::Twin2(my_twin_mask, other_idx)  => {
                match other.value {
                    CellState::Solved(other_v,_) => {
                        // ok substract the other from this twin one
                        let new_mask = my_twin_mask & !other_v;
                        if new_mask.count_ones() == 1 {
                            self.value = CellState::Solved( new_mask, CellActor::Resolved);  
                            println!("Solved twin in reduce cell {:?} value {} my mask {:09b} incoming mask {:09b} resulting mask {:09b}",  self.get_pos(), self.as_string(), my_twin_mask ,other_v, new_mask) ;
                        } else {
                            self.value = CellState::UnSolved( new_mask);
                        }
                    },
                    _ => {
                        println!("Testing twin my index {} my twin other {} index twins  index {}", self.idx, other_idx, other.idx);
                        if other_idx == other.idx {
                            twin_mask = my_twin_mask;
                        }
                    }    
                _ => (),   
                }
            },
            CellState::UnSolved(self_mask) => {
                match other.value {
                    CellState::Solved(other_v,_) => {
                        // ok substract the other from this one
                        let new_mask = self_mask & !other_v;
                        if new_mask.count_ones() == 1 {
                            self.value = CellState::Solved( new_mask, CellActor::Resolved);  
                            println!("Solved in reduce cell {:?} value {} my mask {:09b} incoming mask {:09b} resulting mask {:09b}",  self.get_pos(), self.as_string(), self_mask ,other_v, new_mask) ;
                        } else {
                            self.value = CellState::UnSolved( new_mask);
                        }
                    },
                    CellState::Twin2(_, _)  => (),
                    CellState::UnSolved(other_mask) => {
                        if self_mask.count_ones() == 2 && self_mask == other_mask{
                            self.value =  CellState::Twin2( self_mask, other.idx);  
                            other.value = CellState::Twin2( self_mask, self.idx);  
                            println!("Found a twin: {:?} and  {:?} mask {:09b} ",  self.get_pos(), other.get_pos(), self_mask) ;
                        }
                    }, 
        
                }
            },
        };
        return twin_mask
    }



    pub fn set_init_value(&mut self, v:usize)  {
        self.value = CellState::Solved( 1 << (v - 1) , CellActor::StartValue); 
    } 
    pub fn reset(&mut self)  {
        self.value = CellState::UnSolved(CELL_RESET_MASK); 
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
 
    pub fn get_state(&self) -> CellState {
        // dereference the Rc 
        self.cell.borrow().value.clone()
    }
    pub fn reduce(&self, other: &RcSudokuCell)  -> usize {
        self.cell.borrow_mut().reduce(&mut other.cell.borrow_mut())
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
            reduce_square( &self.rows[r]);
        }
        for r in 0..CELL_ROW {
            reduce_square( &self.cols[r]);
        }
        for r in 0..CELL_ROW {
            reduce_square( &self.squares[r]);
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


fn reduce_square(row_col_square: &dyn RowColSquare) -> Result<i32, SudokuError> {
    let mut mask = 0;
    // Step 1 get the resolved mask for all cells. A 1 on a bitpos means resolved.
    let cells = row_col_square.get_cells();

    let mut twin_mask = CELL_RESET_MASK;

    // first reduce resolved values
    for me_index in 0..CELL_SIZE {
        for other_index in 0 ..CELL_SIZE {
            if me_index != other_index {
                twin_mask &= cells[me_index].reduce(&cells[other_index]);
            }
        }
    }
    if twin_mask != CELL_RESET_MASK {
        println!("Twin mask now {:09b}",twin_mask);
    }
    Ok(0)
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

