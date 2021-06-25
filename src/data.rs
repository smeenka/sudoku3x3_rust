use std::sync::*;
//use std::vec::Vec;
use crate::sudoku_board::*;
use crate::sudoku_state::*;
//use std::borrow::Borrow;
use druid::im;
use druid::*;
use ini::ini;



pub const COMMAND_STEP: Selector<String>= Selector::new("sudoku.step");
pub const COMMAND_SOLVE: Selector<String> = Selector::new("sudoku.solve");
pub const COMMAND_INIT: Selector<String> = Selector::new("sudoku.init");
pub const COMMAND_SELECT: Selector<String> = Selector::new("sudoku.select");
pub const COMMAND_AUTOSELECT: Selector<String> = Selector::new("sudoku.autoselect");
pub const COMMAND_SELECTED: Selector<String> = Selector::new("sudoku.selected");
pub const COMMAND_BACK: Selector<String> = Selector::new("sudoku.back");
pub const COMMAND_NUMBER: Selector<(RcSudokuCell, usize)> = Selector::new("sudoku.number");

pub const INI_FILE:&str = "data/sudoku.ini";

pub const HEX_DIGITS:[char;CELL_SIZE] = ['1','2','3','4','5','6','7','8','9'];

pub const CELL_SIZE: usize = 9;
pub const CELL_ROW:  usize = 9;
pub const CELL_COL:  usize = 9;
pub const CELL_COUNT: usize = CELL_ROW * CELL_COL;


#[derive(Clone, Data, Lens)]
pub struct SelectState {
    selected: String,
   
    //pub board_list:im::Vec<String>
}


#[derive(Clone, Data, Lens)]
pub struct AppState {
    /// The number displayed. Generally a valid float.
    pub message: String,
    step_count: String,
    init_count:String,
    curr_count:String,
    pub which: bool,
    pub su_board: Arc<SudokuBoard>, // non mutable reference, for presentation only
    //pub select_window:SelectState,
    #[data(ignore)]
    pub su_state: SudokuState,   // owner of the mutable sudoku state 
    #[data(ignore)]
    pub board_list:im::Vector<String>,
    #[data(ignore)]
    pub autoselect_list:im::Vector<String>,
    pub selected: String,
}

impl AppState {
   pub fn new() -> AppState {
       let mut board = SudokuBoard::new();
       board.wire();
       AppState {
            message: "Select a board".to_string(),
            step_count: "0".to_string(),
            init_count:"0".to_string(),
            curr_count:"0".to_string(),
            which :true,
            su_board: Arc::new(board),
            su_state:SudokuState::new(),
            //select_window: SelectState {
            selected:"hard".into(),
            board_list: im::vector![],
            autoselect_list: im::vector![],
        }
    }
    pub fn init(&mut self) {
        self.load_file();
    }

    pub fn load_file(&mut self)  {
        // Open the file in read-only mode (ignoring errors)
        let map = ini!(INI_FILE);
    
        for (key, _value) in &map {
            self.board_list.push_back(key.to_string());
        }
        self.selected = "".to_string();
        self.autoselect();
    }
    pub fn select_board(&mut self){
        // Open the file in read-only mode (ignoring errors)
        self.which = false;
        let map = ini!(INI_FILE);
        let board = &*self.su_board;
        let sudoku = map.get(&self.selected).unwrap();
    
        for (key, value) in sudoku {
            if key.starts_with("row"){
                let rowc = key.chars().nth(3).unwrap();
                let row = rowc.to_digit(10).unwrap() - 1; 
                for col in 0..9 {
                    let valc = value.as_ref().unwrap().chars().nth(col).unwrap();
                    if '-' != valc {
                        let v = valc.to_digit(16).unwrap(); 
                        board.init_cell(row as usize, col, v as usize);     
                    }
                } 
            }
        }
        self.su_state.do_count(board);
        self.message = "Step .....".to_string();
    }
    
    //pub const lens_rows: ArcRowLens = ArcRowLens;
    //pub const lens_cells: ArcCellLens = ArcCellLens;

    // get a reference to the board 
    pub fn get_board_ref(&self) -> Arc<SudokuBoard> {
        self.su_board.clone()
    }

    pub fn do_step(&mut self) {
        let state = &mut self.su_state;
        let board = & *self.su_board;
        self.which = false;
        match state.resolve_step(board){
            GameState::Stepping     =>  self.message = "Stepping..".to_string(),
            GameState::Error        =>  self.message = "ERROR".to_string(),
            GameState::Resolved     =>  self.message = "RESOLVED!!".to_string(),            
            _                       =>  self.message = "unknown".to_string()
        }
        self.step_count = format!("{}", state.get_step_count());
        self.init_count= format!("{}",  state.get_init_count());
        self.curr_count= format!("{}",  state.get_curr_count());
        //board.show();
    }  
    pub fn do_step_back(&mut self) {
        let state = &mut self.su_state;
        let board = & *self.su_board;

        state.step_back(board);
        self.step_count = format!("{}", state.get_step_count());
        self.init_count= format!("{}",  state.get_init_count());
        self.curr_count= format!("{}",  state.get_curr_count());
        self.message = "Stepping..".to_string();
        
    }  
    pub fn do_restart(&mut self) {
        let state = &mut self.su_state;
        let board = & *self.su_board;
        board.reset();
        state.reset();
        self.step_count = format!("{}", state.get_step_count());
        self.init_count= format!("{}",  state.get_init_count());
        self.curr_count= format!("{}",  state.get_curr_count());
        self.which =false;
    }
    pub fn autoselect(&mut self) {
        let listref = &self.board_list;
        let result:im::Vector<_> = listref
            .into_iter()
            .filter(|s| s.contains(&self.selected) )
            .collect();
        self.autoselect_list = im::vector![];
        for s in result {
            self.autoselect_list.push_back(s.to_string());
        }
    }    
}

/*
pub struct ArcRowLens;

impl Lens<AppState, Vec<Row> >  for ArcRowLens {
    fn with<R, F: FnOnce(&Vec<Row>) -> R>(&self, data: &AppState, f: F) -> R {
        let board = &data.board;
        f(&board.rows)
    }
    fn with_mut<R, F: FnOnce(&mut Vec<Row>) -> R>(&self, _data: &mut AppState, f: F) -> R {
        let mut result = vec![];
        f(&mut result)
    }
}

pub struct ArcCellLens;

impl Lens<Row, Vec<RcSudokuCell>> for ArcCellLens {
    fn with<R, F: FnOnce(&Vec<RcSudokuCell>) -> R>(&self, data: &Row, f: F) -> R {
        f(&data.cells)
    }
    fn with_mut<R, F: FnOnce(&mut Vec<RcSudokuCell>) -> R>(&self, _data: &mut Row, f: F) -> R {
        let mut result = vec![];
        f(&mut result)
    }
}
*/
