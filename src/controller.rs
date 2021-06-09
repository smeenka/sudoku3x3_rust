use druid::{
    widget::{Controller,  },
    Env, Event, EventCtx, HotKey, KbKey,  Widget,  Command
};
use crate::sudoku::{SudokuBoard};
use ini::ini;


use crate::{
    data::*
};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct SudokuController{
    pub file:Option<File>,
}

impl<W: Widget<AppState>> Controller<AppState, W> for SudokuController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppState,  env: &Env)
    {
        match event {
            Event::KeyDown(k_e) if HotKey::new(None, KbKey::Enter).matches(k_e) => {
                println!("Key down:{:?} child: " , k_e  );
                //ctx.set_handled();
            },
            Event::KeyDown(_k_e)  => {
                println!("Key down anykey "   );
                //ctx.set_handled();
            },
            Event::Command(cmd)  => {
                handle_commands(cmd, data);
                ctx.request_update();
                //ctx.set_handled();
            }
            _ => {
                //println!("Event {:?}", event);
            }
        }
        child.event(ctx, event, data, env);
    }
}



fn handle_commands(cmd: &Command, data: &mut AppState) {
    if  cmd.is(COMMAND_INIT)
    {
        let sel = cmd.get(COMMAND_INIT);
        println!("Received command Init with id  {:?}", sel   );
        data.do_restart();
        load_file(&*data.board);
    } else 
    if  cmd.is(COMMAND_STEP)
    {
        let sel = cmd.get(COMMAND_SOLVE);
        println!("Received command Solve with id  {:?}", sel   );
        data.do_step();
    } else 
    if  cmd.is(COMMAND_SLOWMOTION)
    {
        let sel = cmd.get(COMMAND_SLOWMOTION);
        println!("Received command Step with id  {:?}", sel   );
    } else 
    if  cmd.is(COMMAND_SOLVE)
    {
        let sel = cmd.get(COMMAND_SOLVE);
        println!("Received command solve with id  {:?}", sel   );
    } 

}


                /*
                on_click(|_ctx, data: &mut AppState, _env| {
                println!("Clicked step");
                data.message = "step".to_string();
                let boardref = &*data.board;
                boardref.resolve_step();
                _ctx.request_paint();
                _ctx.request_layout();
            */    

enum LoadState{
    START,
    LOAD,
    END
}
        

fn load_file(data: &SudokuBoard)  {             
    // Open the file in read-only mode (ignoring errors)
    let map = ini!("data/sudoku.ini");

    for (header, section) in &map {
        println!("=========== Header: {}", header);
        for (key, value) in section {
            //println!("{}:{:?}", key, value );
        }
    }
    let sudoku1 = map.get(&"sudoku1".to_string()).unwrap();
    for (key, value) in sudoku1 {
        if key.starts_with("row"){
            let rowc = key.chars().nth(3).unwrap();
            let row = rowc.to_digit(10).unwrap() - 1; 
            for col in 0..9 {
                let valc = value.as_ref().unwrap().chars().nth(col).unwrap();
                if '-' != valc {
                    let v = valc.to_digit(16).unwrap(); 
                    println!("{} {} {}",row,col,v);
                    data.init_cell(row as usize, col, v as usize);     
                }
            } 
        }
        

    }
}
    /*   
    let state:LoadState = LoadState::START;
    let filename = "data/easy.txt";
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file); 
    let mut ri = 0;    
    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for (index, line) in reader.lines().enumerate() {
        state = match state {
            LoadState::START  if line.unwrap().starts_with("start") =>LoadState::LOAD,
            LoadState::LOAD  if line.unwrap().starts_with("END") =>LoadState::END,
            LoadState::LOAD  if line.unwrap().starts_with("|") => {
                data.init_line(&line.unwrap(), ri);
                ri += 1;
                LoadState::LOAD
            },
            _  => LoadState::END 
        }
    }
 */   
