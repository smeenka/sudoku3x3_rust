use druid::{
    widget::{Controller,  },
    Env, Event, EventCtx,  Widget,  Command, 
};
//use crate::sudoku::{SudokuBoard};
use ini::ini;


use crate::{
    data::*
};
use std::fs::File;

pub struct SudokuController{
    pub file:Option<File>,
}

impl<W: Widget<AppState>> Controller<AppState, W> for SudokuController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut AppState,  env: &Env)
    {
        match event {
            //Event::KeyDown(k_e) if HotKey::new(None, KbKey::Enter).matches(k_e) => {
            //    println!("Key down:{:?} child: " , k_e  );
            //    ctx.set_handled();
            //},
            Event::MouseDown(_me) => {
                //println!("Mouse down anykey {:?}" ,me.pos );
            }            
            Event::KeyDown(ke)  => {
                println!("Key Down {:?}" ,ke.key );
                data.autoselect();
                //ctx.set_handled();
            },
            Event::Command(cmd)  => {
                handle_commands(cmd, data);
                ctx.set_handled();

            }
            _ => {
                //println!("Event {:?}", event);
            }
        }
        child.event(ctx, event, data, env);
    }
}


//let y: Vec<_> = x.iter().filter(p).collect();

fn handle_commands(cmd: &Command, data: &mut AppState) {
    if  cmd.is(COMMAND_INIT)
    {
        let sel = cmd.get(COMMAND_INIT);
        println!("Received command Init with id  {:?}", sel   );
        data.do_restart();
        data.count_initial();
        data.message = "Select digits ..".into();
    } else 
    if  cmd.is(COMMAND_SELECT)
    {
        let sel = cmd.get(COMMAND_SELECT);
        println!("Received command Select with id  {:?}", sel   );
        data.which = true;
    } else 
    if  cmd.is(COMMAND_AUTOSELECT)
    {
        let sel = cmd.get(COMMAND_AUTOSELECT);
        println!("Received command Select with id  {:?}", sel   );
        data.autoselect();
    } else 
    if  cmd.is(COMMAND_SELECTED)
    {
        let sel = cmd.get(COMMAND_SELECTED);
        println!("Received command Select with id  {:?}", sel   );
        data.do_restart();
        data.selected = sel.unwrap().to_string();
        select_board(data);     
        data.count_initial();
        data.which = false;
    } else 
    if  cmd.is(COMMAND_STEP)
    {
        let sel = cmd.get(COMMAND_SOLVE);
        println!("Received command Solve with id  {:?}", sel   );
        data.do_step();
        data.count_current();
        data.message = "Stepping ..".into();
        data.which = false;

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
    } else
    if  cmd.is(COMMAND_NUMBER)
    {
        let sel = cmd.get(COMMAND_NUMBER).unwrap();
        println!("Received command Solve with id  {:?} from {:?}", sel.1, sel.0.get_value()   );
        sel.0.set_init_value(sel.1);
        data.do_step();
        data.count_initial();
    } 
}

pub fn load_file(data: &mut AppState)  {
    // Open the file in read-only mode (ignoring errors)
    let map = ini!(INI_FILE);

    for (key, _value) in &map {
        data.board_list.push_back(key.to_string());
    }
    data.selected = "".to_string();
    data.autoselect();
}
fn select_board(data: &mut AppState){
    // Open the file in read-only mode (ignoring errors)
    let map = ini!(INI_FILE);
    let board = & *data.board;             
    let sudoku = map.get(&data.selected).unwrap();

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
}
