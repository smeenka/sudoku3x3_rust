use std::sync::*;
//use std::vec::Vec;
use crate::sudoku_board::*;
use crate::sudoku_state::*;
//use std::borrow::Borrow;
use druid::im;
use druid::*;
use ini::ini;
use std::fs::*;
use std::io::*;
//use std::io::buffered::BufferedWriter;



pub const COMMAND_STEP: Selector<String>= Selector::new("sudoku.step");
pub const COMMAND_SOLVE: Selector<String> = Selector::new("sudoku.solve");
pub const COMMAND_SAVE: Selector<String> = Selector::new("sudoku.save");
pub const COMMAND_INIT: Selector<String> = Selector::new("sudoku.init");
pub const COMMAND_SELECT: Selector<String> = Selector::new("sudoku.select");
pub const COMMAND_SELECTED: Selector<String> = Selector::new("sudoku.selected");
pub const COMMAND_BACK: Selector<String> = Selector::new("sudoku.back");
pub const COMMAND_NUMBER: Selector<(RcSudokuCell, usize)> = Selector::new("sudoku.number");

pub const INI_FILE:&str = "data/sudoku.ini";

pub const HEX_DIGITS:[char;CELL_SIZE] = ['1','2','3','4','5','6','7','8','9'];
pub const HEX_STRS:[&'static str;CELL_SIZE + 1] = ["0","1","2","3","4","5","6","7","8","9"];

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
    pub su_board: Arc<SudokuBoard>, // non mutable reference, for presentation only
    //pub select_window:SelectState,
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
            su_board: Arc::new(board),
            su_state:SudokuState::new(),
            selected:"".into(),
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
    pub fn save_file(&mut self)  {
        // Open the path in read-only mode, returns `io::Result<File>`
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(INI_FILE).unwrap();

        writeln!(file,"");    
        writeln!(file,";Added by Rust Sudoku.");    
        writeln!(file,"[{}]",self.selected);    
        for row in &self.su_board.rows {
            write!(file,"{}=",row.get_id());    
            for cell in &row.cells{
                write!(file,"{}",cell.get_value());    
            }
            writeln!(file,"");    
        }
        writeln!(file,";xxx");    
    }
 
    pub fn select_board(&mut self){
        // Open the file in read-only mode (ignoring errors)
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
        self.su_state.reduce_step(board);
        self.message = "Rightclick for manual select".to_string();
    }
    
    //pub const lens_rows: ArcRowLens = ArcRowLens;
    //pub const lens_cells: ArcCellLens = ArcCellLens;

    // get a reference to the board 
    pub fn get_board_ref(&self) -> Arc<SudokuBoard> {
        self.su_board.clone()
    }
    pub fn show_select(&mut self) {
        self.su_state.select();
        self.message = "Select a board ..".into();
    }
    pub fn exec_save(&mut self) {
        if GameState::Select == self.su_state.get_state(){
            self.save_file();
            let board = &*self.su_board;
            self.su_state.play(board);
            self.message = "Start stepping ..".into();
        }else {
            self.su_state.select();
            self.message = "Choose  unique name ..".into();
        }
    }
    pub fn isSelectBoardDisabled(&self) -> bool {
        let gamestate = self.su_state.get_state();
        ! ( GameState::Select == gamestate  || GameState::ManualInput == gamestate)
    }
    pub fn isBackDisabled(&self) -> bool {
        let gamestate = self.su_state.get_state();
        GameState::Select == gamestate
    }
    pub fn isStepDisabled(&self) -> bool {
        let gamestate = self.su_state.get_state();
        ! (GameState::ManualInput == gamestate || GameState::Stepping == gamestate)
    }
    pub fn isSolveDisabled(&self) -> bool {
        let gamestate = self.su_state.get_state();
        // ! (GameState::ManualInput == gamestate || GameState::Stepping == gamestate )
        true
    }
    pub fn isSelectVisible(&self) -> bool {
        let gamestate = self.su_state.get_state();
        GameState::Select == gamestate 
    }
    pub fn isSaveDisabled(&self) -> bool {
        let gamestate = self.su_state.get_state();
        GameState::Select == gamestate && self.autoselect_list.len() > 0
    }

    pub fn do_step(&mut self) {
        let state = &mut self.su_state;
        let board = & *self.su_board;
        match state.resolve_step(board){
            GameState::Stepping     =>  self.message = "Stepping..".to_string(),
            GameState::Error        =>  self.message = "ERROR".to_string(),
            GameState::Solved       =>  self.message = "RESOLVED!!".to_string(),            
            GameState::ManualInput  =>  self.message = "Manual Input".to_string(),            
            _                       =>  self.message = "unknown".to_string()
        }
        //board.show();
    }  
    pub fn do_step_back(&mut self) {
        let state = &mut self.su_state;
        let board = & *self.su_board;

        state.step_back(board);
        self.message = "Stepping..".to_string();
    }  

    pub fn do_reduce(&mut self) {
        let state = &mut self.su_state;
        let board = & *self.su_board;
        match state.reduce_step(board){
            GameState::Error        =>  self.message = "ERROR".to_string(),
            _                       =>  self.message = "Manual Input".to_string(),            
        }
        //board.show();
    }  
    pub fn do_restart(&mut self) {
        let state = &mut self.su_state;
        let board = & *self.su_board;
        board.reset();
        state.reset();
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
