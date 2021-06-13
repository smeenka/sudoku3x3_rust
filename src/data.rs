use std::sync::*;
use std::vec::Vec;
use crate::sudoku::*;
//use std::borrow::Borrow;
use druid::im;
use druid::*;

pub const COMMAND_STEP: Selector<String>= Selector::new("sudoku.step");
pub const COMMAND_SOLVE: Selector<String> = Selector::new("sudoku.solve");
pub const COMMAND_INIT: Selector<String> = Selector::new("sudoku.init");
pub const COMMAND_SELECT: Selector<String> = Selector::new("sudoku.select");
pub const COMMAND_AUTOSELECT: Selector<String> = Selector::new("sudoku.autoselect");
pub const COMMAND_SELECTED: Selector<String> = Selector::new("sudoku.selected");
pub const COMMAND_SLOWMOTION: Selector<String> = Selector::new("sudoku.slowmotion");
pub const COMMAND_NUMBER: Selector<(RcSudokuCell, usize)> = Selector::new("sudoku.number");

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
    steps: usize,
    steps_s: String,
    start_count:usize,
    start_count_s:String,
    curr_count_s:String,
    solved: bool,
    pub which: bool,
    pub board: Arc<SudokuBoard>,
    //pub select_window:SelectState,
    #[data(ignore)]
    pub board_list:im::Vector<String>,
    #[data(ignore)]
    pub autoselect_list:im::Vector<String>,
    pub selected: String,
}

impl AppState {
   pub fn new() -> AppState {
        let  mut board = SudokuBoard::new();
        board.wire();
        AppState {
            // The number displayed. Generally a valid float.
            message: "Select a board".to_string(),
            steps: 0,
            steps_s: "0".to_string(),
            start_count: 0,
            start_count_s:"0".to_string(),
            curr_count_s:"0".to_string(),
            solved: false,
            which :true,
            board: Arc::new(board),
            //select_window: SelectState {
            selected:"".into(),
            board_list: im::vector![],
            autoselect_list: im::vector![],
            //}

        }
    }
    //pub const lens_rows: ArcRowLens = ArcRowLens;
    //pub const lens_cells: ArcCellLens = ArcCellLens;

    pub fn do_step(&mut self) {
        self.steps += 1;
        self.steps_s = format!("{}", self.steps);
        let board = &*self.board;
        board.resolve_step();
        //board.show();
        self.which = false;
    }    
    pub fn do_restart(&mut self) {
        let board = &*self.board;
        self.steps = 0;
        self.steps_s = "0".to_string();
        board.reset();
        self.which =false;
    }
    pub fn count_initial(&mut self) {
        self.count_current();
        let board = &*self.board;
        self.start_count = board.count_solved(true);
        self.start_count_s= format!("{}", self.start_count);

    }    
    pub fn count_current(&mut self) {
        let board = &*self.board;
        self.curr_count_s= format!("{}", board.count_solved(false));
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


pub struct ArcRowLens;

impl Lens<AppState, Vec<Row> >  for ArcRowLens {
    fn with<R, F: FnOnce(&Vec<Row>) -> R>(&self, data: &AppState, f: F) -> R {
        let board = &*data.board;
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