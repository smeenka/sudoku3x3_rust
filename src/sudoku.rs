use std::rc::{Rc};
use std::cell::{RefCell};
//use std::sync::Arc;
use druid::{Data, Lens};
use crate::data::*;
use std::collections::*;

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
           value: CellState::UnSolved(CELL_RESET_MASK),
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
    /**
     *  If this cell is resolved return the the bitmask.
     */
    pub fn get_resolved_mask(&self) -> usize {
        // a 1 bit in the mask indicate a resolved cell
        match self.value {
            CellState::Solved(v,_)  => v, 
            _                      => 0,
        }
    }
    /**
     *  If this cell is resolved return the bitmask. 
     *  If this cell is NOT resolved return the possibities for this cell. 
     */
    pub fn get_unresolved_mask(&self) -> usize {
        match self.value {
            CellState::Solved(v,_) =>  v, 
            CellState::UnSolved(n)  =>  n, 
        }
    }
 
    // substract bits in in the possible bit masks. If current cell is now resolved return true
    pub fn reduce(&mut self, other_mask:usize)  -> Result<usize, String> {
        // substract the mask from the bits in this cell. If only one bit left, mark as solved
         
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
                        println!("Error: in reduce zero bits left for {:?}  incoming mask {:09b}", self.get_pos(), other_mask);
                        Err( format!("Error: in reduce zero bits left for {:?}  incoming mask {:09b}", self.get_pos(), other_mask) )
                    }else {
                        Ok(0)
                    }
                }
                
            },
        }        
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
    pub fn resolve_step( &self) ->Result<usize, String>{
        for r in 0..CELL_ROW {
            reduce_square( &self.rows[r])?;
        }
        for r in 0..CELL_ROW {
            reduce_square( &self.cols[r])?;
        }
        for r in 0..CELL_ROW {
            reduce_square( &self.squares[r] )?;
        }
        Ok(0)
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


fn reduce_square(row_col_square: &dyn RowColSquare) -> Result<usize, String> {
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
            println!("Error: in reduce square zero bits left for value {}  ", n+1);

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
            println!("Found a twin2 (not yet resolved). Twin   mask {:09b}", key  );
            overall_twin_mask |= key;
        }
    }
    if overall_twin_mask > 0{
        println!("Reduce due to found twin mask {:09b} ", overall_twin_mask);
        for c in 0..CELL_SIZE {
            let cell = &cells[c];
            //cell.reduce(overall_twin_mask);
        }
        
    }


 
   // Show the results on the terminal
    print!("{:10} ", row_col_square.get_id());
    for n in 0..CELL_SIZE {
        print!(" {}:{:09b} ", n+1, possible_cells[n] );
    }
    println!();

    /*
    print!("{:10} ", row_col_square.get_id());
    for (key, val) in count_hash.iter() {
        print!("-{:09b}:{:?}-",  key, val);
    }
    println!();
*/

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

