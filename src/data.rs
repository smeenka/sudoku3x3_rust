use std::rc::Rc;
use std::sync::{Arc};
use crate::sudoku::{SudokuBoard, Row, RcSudokuCell };

use druid::{
    Selector, Lens, Data
};

pub const COMMAND_STEP: Selector<String>= Selector::new("sudoku.step");
pub const COMMAND_SOLVE: Selector<String> = Selector::new("sudoku.solve");
pub const COMMAND_INIT: Selector<String> = Selector::new("sudoku.init");
pub const COMMAND_SLOWMOTION: Selector<String> = Selector::new("sudoku.slowmotion");

pub const HEX_DIGITS:[char;CELL_SIZE] = ['1','2','3','4','5','6','7','8','9'];

pub const CELL_SIZE: usize = 9;
pub const CELL_ROW:  usize = 9;
pub const CELL_COL:  usize = 9;
pub const CELL_COUNT: usize = CELL_ROW * CELL_COL;



#[derive(Clone, Data, Lens)]
pub struct AppState {
    /// The number displayed. Generally a valid float.
    message: String,
    steps: usize,
    steps_s: String,
    start_count:usize,
    start_count_s:String,
    curr_count_s:String,
    solved: bool,
    pub board: Arc<SudokuBoard>,
}

impl AppState {
   pub fn new() -> AppState {
        let  mut board = SudokuBoard::new();
        let bref = board.wire();
        bref.init();
        AppState {
            // The number displayed. Generally a valid float.
            message: "Starting sudodku solver".to_string(),
            steps: 0,
            steps_s: "0".to_string(),
            start_count: 0,
            start_count_s:"0".to_string(),
            curr_count_s:"0".to_string(),
            solved: false,

            board: Arc::new(board)
        }
    }
    pub const lens_rows: ArcRowLens = ArcRowLens;
    pub const lens_cells: ArcCellLens = ArcCellLens;

    pub fn do_step(&mut self) {
        self.steps += 1;
        self.steps_s = format!("{}", self.steps);
        let board = &*self.board;
        board.resolve_step();
        self.curr_count_s= format!("now:{}", board.count_solved(false));
        board.show();
    }    
    pub fn do_restart(&mut self) {
        let board = &*self.board;
        self.steps = 0;
        self.steps_s = "0".to_string();
        self.start_count = 0;
        self.start_count_s= format!("start:{}", board.count_solved(true));
        self.curr_count_s = self.start_count_s.clone();
        //self.board.deref().init();
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