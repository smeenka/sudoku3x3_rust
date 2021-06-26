use druid::{
    widget::{Controller,  },
    Env, Event, EventCtx,  Widget,  Command, 
};
//use crate::sudoku::{SudokuBoard};
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
        data.message = "Select digits ..".into();
    } else 
    if  cmd.is(COMMAND_SELECT)
    {
        let sel = cmd.get(COMMAND_SELECT);
        data.show_select();     
        println!("Received command Select with id  {:?}", sel   );
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
        data.select_board();     
    } else 
    if  cmd.is(COMMAND_STEP)
    {
        let sel = cmd.get(COMMAND_SOLVE);
        println!("Received command Solve with id  {:?}", sel   );
        data.do_step();

    } else 
    if  cmd.is(COMMAND_BACK)
    {
        println!("Received command Step Back");
        data.do_step_back();
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
    } 
}


