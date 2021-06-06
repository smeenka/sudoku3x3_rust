use std::rc::{Rc};
use std::cell::{RefCell};
use std::sync::Arc;
use druid::{Data, Lens};

use crate::data::*;

const MASK_VALUES:[usize;CELL_SIZE+1] = [0xFFFF, 0xFFFE, 0xFFFD,0xFFFB,0xFFF7,0xFFEF,0xFFDF,0xFFBF,0xFF7F,0xFEFF ] ;


#[derive(Clone)]
pub enum CellActor{
    StartValue,
    Resolved,
    Guessed(u8),
}


#[allow(dead_code)]
#[derive(Clone)]
pub enum CellState{
    Solved(u8, CellActor),
    UnSolved(usize),
    Locked2([u8;2]),
    Locked3([u8;3]),
}

#[derive(Clone)]
pub struct SudokuCell {
    pub value: CellState,
    row:usize,
    col:usize,
    message:String
}

impl SudokuCell {
    pub fn new(r:usize,c:usize) -> SudokuCell {
        SudokuCell{
           value: CellState::UnSolved(0x1FF),
           row:r,
           col:c,
           message:"init".to_string()
        }
    }
    pub fn get_value(&self) -> String {
        // dereference the Rc into the CellState
        match self.value {
            CellState::Solved(v,_) => v.to_string(),
            CellState::UnSolved(_) => String::from("-"),
            _ => String::from("x"),
        }
    }
    pub fn get_mask(&self) -> usize {
        // zero bits in the mask indicate resolved cells, one bits are unresolved cells
        match self.value {
            CellState::Solved(v,_) => {
                MASK_VALUES[v as usize]  // so value 1 will result in mask 0xFFFE
            },
            _ => 0xFFFF,
        }
    }
    pub fn resolve(&mut self, mask:usize)  {
        // substract the mask from the bits in this cell. If only one bit left, mark as solved
        let mut newvalue = self.value.clone();
        match self.value {
            CellState::Solved(_v,_) => (),
            CellState::UnSolved(n) => {
                let new_mask = n & mask;
                 
                if new_mask.count_ones() == 1 {
                    let mut cv = 0;
                    for n in 1 .. (CELL_SIZE + 1) {
                        if 1 << n  == new_mask {
                            cv = n;
                            break;
                        }
                    };  
                    self.message = format! ("solved{} ", cv);
                    newvalue = CellState::Solved( cv as u8 , CellActor::Resolved);  
                } else { 
                    if n != new_mask {
                        self.message = format! ("new mask{:3x} ", new_mask);
                    }
                    newvalue = CellState::UnSolved( new_mask); 
                }
            },
            _ => (),
        };
        self.value = newvalue;
    }
    pub fn set_value(&mut self, v:u8)  {
        // dereference the Rc into the CellState
        self.value = CellState::Solved(v, CellActor::StartValue);
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
#[derive(Clone, Lens, Data)]
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
    pub fn set_value(&self, v:u8)  {
        // dereference the Rc into the CellState
        self.cell.borrow_mut().set_value(v);
    } 
    pub fn get_state(&self) -> CellState {
        // dereference the Rc 
        self.cell.borrow().value.clone()
    }
    pub fn get_mask(&self) -> usize {
        self.cell.borrow().get_mask()
    }
    pub fn resolve(&self, mask:usize)  {
        self.cell.borrow_mut().resolve(mask);
    }
    pub fn get_message(&self)  -> String{
        self.cell.borrow().message.clone()
    }
    pub fn count_solved(&self, intitial:bool ) -> usize{
        self.cell.borrow().count_solved(intitial )
    }
}
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
}

#[derive(Clone, Data, Lens )]
pub struct Row {
    #[data(ignore)]
    pub cells:  Vec<RcSudokuCell>,
    pub message: String
}
impl Row {
    fn new ()-> Row {
        Row{ cells:vec![], message: "unwired".to_string()  }
    }
    pub fn wire(&mut self, r:usize,  allcells: &AllCells) {
        let startindex = r * CELL_SIZE;
        for n in 0..CELL_SIZE {
            let allcell  = &allcells.cells[startindex + n]; 
            self.cells.push(RcSudokuCell::new( &allcell.cell ));
            self.message = "Wired".to_string();
        }
    }

    fn resolve(&self) {
        let mut mask = 0xFFFF;
        for n in 0..CELL_SIZE {
            mask &= self.cells[n].get_mask();
        }
        for n in 0..CELL_SIZE {
            self.cells[n].resolve(mask);
        }
    }
}

#[derive(Clone, Data)]
pub struct Col {
    #[data(ignore)]
    cells:  Vec<RcSudokuCell>,
    message:String
}
impl Col {
    fn new ()-> Col {
        Col{ cells:vec![], message: "unwired".to_string()  }
    }
    pub fn wire(&mut self, c:usize,  allcells: &AllCells) {
        let startindex = c;
        for n in 0..CELL_SIZE {
            let allcell  = &allcells.cells[startindex +  n*CELL_SIZE]; 
            self.cells.push(RcSudokuCell::new( &allcell.cell ));
            self.message = "Wired".to_string();
        }
    }
    fn resolve(&self) {
        let mut mask = 0xFFFF;
        for n in 0..CELL_SIZE {
            mask &= self.cells[n].get_mask();
        }
        for n in 0..CELL_SIZE {
            self.cells[n].resolve(mask);
        }
    }
}

#[derive(Clone, Data)]
pub struct Square {
    #[data(ignore)]
    cells:  Vec<RcSudokuCell>,
    message: String
}
impl Square {
    fn new ()-> Square {
        Square{ cells:vec![], message: "unwired".to_string()  }
    }
    pub fn wire(&mut self, r:usize,  c:usize,  allcells: &AllCells) {
        for n in 0..CELL_SIZE {
            let ri =  (r + n/3) * CELL_SIZE;
            let ci =  c + n%3; 
            let alli = ri + ci;
            let allcell  = &allcells.cells[alli]; 
            self.cells.push(RcSudokuCell::new( &allcell.cell ));
            self.message = "Wired".to_string();
        }
    }
    fn resolve(&self) {
        let mut mask = 0xFFFF;
        for n in 0..CELL_SIZE {
            mask = mask & self.cells[n].get_mask();
        }
        for n in 0..CELL_SIZE {
            self.cells[n].resolve(mask);
        }
    }
}


#[derive(Clone, Data, Lens )]
pub struct SudokuBoard{
    #[data(ignore)]
    allcells:AllCells,
    pub message: String,
    #[data(ignore)]
    pub rows: Vec<Row>,
    #[data(ignore)]
    pub cols: Vec<Col>,
    #[data(ignore)]
    pub squares: Vec<Square>,
    pub step:usize,
}


impl SudokuBoard {
    pub fn new () -> SudokuBoard {
        println!("New board. Board size: {}:{}", CELL_ROW ,CELL_COL);

        SudokuBoard {
            allcells:AllCells::new(),
            message: String::from("initializing sudoku board instance"),
            rows: { let mut  rws  = vec![];
                    for _ in 0 .. CELL_ROW { 
                        rws.push( Row::new() ) ;
                    };
                    rws   
                  },
            cols: { let mut cols = vec![];
                    for _ in 0 .. CELL_ROW {
                        cols.push(Col::new() );
                    };
                    cols  
                  },
            squares: { let mut sq = vec![];
                    for _ in 0 .. CELL_ROW {
                        sq.push(Square::new());
                    };
                    sq   
                  },
            step: 0,
        }        
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

    fn init_cell( &self, r:usize , c:usize, v:u8){
        let cell = &self.rows[r].cells[c];
        let refcell = &*cell;
        refcell.set_value(v);
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
        println!("Resolve step {}" , self.step);
        for r in 0..CELL_ROW {
            self.rows[r].resolve();
        }
        for r in 0..CELL_COL {
            self.cols[r].resolve();
        }
        for r in 0..CELL_ROW {
            self.squares[r].resolve();
        }
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
            bref.resolve_step();
            bref.show();
        }
    }
}

