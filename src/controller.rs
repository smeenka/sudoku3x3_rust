use druid::{
    widget::{Controller,  },
    Env, Event, EventCtx, HotKey, KbKey,  Widget,  Command
};

use crate::{
    data::*
};

pub struct SudokuController;

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
