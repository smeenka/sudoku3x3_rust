use std::sync::Arc;
use druid::widget::prelude::*;
use druid::*;
use druid::widget::*;
use druid::piet::*;
extern crate ini;

use sudoku3x3::{
    controller::*,
    sudoku_board::*,
    sudoku_state::*,
    data::*
};

pub fn main() {
    let mut app_state = AppState::new();
    app_state.init();
    let board = app_state.get_board_ref();
    let window = WindowDesc::new(ui_builder(board))
            .title("Sudoku 3x3")
            .window_size((600.0, 650.0));

    AppLauncher::with_window(window)
        //.log_to_console()
        .launch(app_state)

        .expect("launch failed");
}

fn ui_builder(board:Arc<SudokuBoard>) -> impl Widget<AppState> {
    Flex::column()
        .with_spacer(5.0)
        .with_child(ui_build_menuitems() )
        .with_spacer(5.0)
        .with_flex_child(
            Either::new(
                |data, _env| data.isSelectVisible(),
                build_autoselect(),
                ui_build_board(&*board)
            ),10.0)
        .with_spacer(5.0)
        .with_child(ui_build_statusline() )
        .with_spacer(5.0)
        .controller(SudokuController {file:Option::None})
} // ui_builder

fn build_autoselect() -> impl Widget<AppState>{
    Flex::column()
    .with_spacer(10.0)
    .with_child(
        Flex::row()
        .with_spacer(10.0)
        .with_child(
            Label::new("Filter:")
            .background(Color::rgb8(50, 10, 10))
            .fix_height(30.0)
        )
        .with_spacer(10.0)
        .with_child(TextBox::new()
            .lens(AppState::selected)
        )
        .fix_height(20.0)
        .align_left()
    )
    .with_spacer(10.0)
    .with_flex_child(
        Scroll::new(
            List::new(|| {
                Label::new(|data: &String, _: &_| format!("{}", data))
                    .background(Color::rgb8(10, 10, 10))
                    .expand_width()
                    .padding(5.0)
                    .on_click(| ctx, data, _env| 
                        ctx.submit_command(COMMAND_SELECTED.with( data.to_string() ) ) ) 
            })
            .lens(AppState::autoselect_list),
        ).vertical(),
        10.0 
    )


}



fn ui_build_menuitems() -> impl Widget<AppState> {
    Flex::row()
    .with_spacer(5.0)
    .with_child(Button::new("Select board")
        .disabled_if(|data:&AppState, _| data.isSelectBoardDisabled())    
        .on_click(|ctx, _data, _env| ctx.submit_command(COMMAND_SELECT.with( "".to_string() )  ) )
    )
    .with_flex_spacer(1.0)
    .with_child(Button::new("Restart")
        .on_click(|ctx, _data, _env|  ctx.submit_command(COMMAND_INIT.with( "".to_string() )  ) )   
    )
    .with_flex_spacer(1.0)
    .with_child(Button::new("Back")
        .disabled_if(|data:&AppState, _| data.isBackDisabled())    
        .on_click(|ctx, _data: &mut AppState, _env|  ctx.submit_command(COMMAND_BACK.with( "".to_string()  ) ) )  
    )
    .with_flex_spacer(1.0)
    .with_child(Button::new("Step")
        .disabled_if(|data:&AppState, _| data.isStepDisabled())    
        .on_click(|ctx, _data, _env|  ctx.submit_command(COMMAND_STEP.with( "".to_string()  ) ) )
    )
    .with_flex_spacer(1.0)
    .with_child (
        Button::new("Solve")
           .disabled_if(|data:&AppState, _| data.isSolveDisabled())    
           .on_click(| ctx, _data:&mut AppState, _env| ctx.submit_command(COMMAND_SOLVE.with( "".to_string() ) ) ) 
    ) 
    .with_flex_spacer(1.0)
    .with_child (
        Button::new("Save")
           .disabled_if(|data:&AppState, _| data.isSaveDisabled())    
           .on_click(| ctx, _data:&mut AppState, _env| ctx.submit_command(COMMAND_SAVE.with( "".to_string() ) ) ) 
    ) 
    .with_spacer(5.0)
}
/*
fn build_autocomplete<T:Data> (app_data:&AppState) ->impl Widget<T>{
    List::new(|| {
        Label::new(|data: &String, _: &_| format!("Board: {}", data))
            .center()
            .background(Color::hlc(230.0, 50.0, 50.0))
            .fix_height(200.0)
            .expand_width()
    })
    .lens(AppState::board_list)
}
*/
fn ui_build_board<T:Data> (board:&SudokuBoard) -> impl Widget<T> where Flex<AppState>: druid::Widget<T>{
    let mut column = Flex::column();

    let arcrows =  &board.rows;

    for c in 0..9 {
        //column.add_child(build_flex_row( &bref.rows[c]));
        column.add_flex_child(  build_row( &arcrows[c]  ), 1.0);
        match c {
            2 => column.add_default_spacer(),
            5 => column.add_default_spacer(),
            8 => column.add_default_spacer(),
            _ => ()
        }
    };
    column
}
fn build_row<T:Data>(row:&Row) -> impl Widget<T> where Flex<AppState>: druid::Widget<T> {
    let mut frow = Flex::row(); 
    for r in 0 .. 9 {    
        
        let arccells = &row.cells;
        let cell_widget = CellWidget::new(arccells[r].clone());
        frow.add_flex_child(cell_widget, 1.0 );
        match r {
            2 => {frow.add_default_spacer()},
            5 => {frow.add_default_spacer()},
            _ => ()
        }
    }
    frow
}//build_row

pub struct CellWidget{
    pub cell: RcSudokuCell,
}

impl CellWidget {
    fn new(cell:RcSudokuCell) -> CellWidget {
        CellWidget {
            cell: cell 
        }
    }
}

// If this widget has any child widgets it should call its event, update and layout
// (and lifecycle) methods as well to make sure it works. Some things can be filtered,
// but a general rule is to just pass it through unless you really know you don't want it.
impl Widget<AppState> for CellWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut AppState, _env: &Env) {
        match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                ctx.show_context_menu( make_numberselect_menu(self.cell.clone()), mouse.pos);
            },
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppState,
        _env: &Env,
    ) -> Size {
        // BoxConstraints are passed by the parent widget.
        // This method can return any Size within those constraints:
        // bc.constrain(my_size)
        //
        // To check if a dimension is infinite or not (e.g. scrolling):
        // bc.is_width_bounded() / bc.is_height_bounded()
        //
        // bx.max() returns the maximum size of the widget. Be careful
        // using this, since always make sure the widget is bounded.
        // If bx.max() is used in a scrolling widget things will probably
        // not work correctly.
        if bc.is_width_bounded() | bc.is_height_bounded() {
            let size = Size::new(100.0, 100.0);
            bc.constrain(size)
        } else {
            bc.max()
        }
    }

    // The paint method gets called last, after an event flow.
    // It goes event -> update -> layout -> paint, and each method can influence the next.
    // Basically, anything that changes the appearance of a widget causes a paint.
    fn paint(&mut self, ctx: &mut PaintCtx, _data: &AppState, env: &Env) {
        // Clear the whole widget with the color of your choice
        // (ctx.size() returns the size of the layout rect we're painting in)
        // Note: ctx also has a `clear` method, but that clears the whole context,
        // and we only want to clear this widget's area.
        let size = ctx.size();

        // Rectangles: the path for practical people
        let bounds = ctx.size().to_rect();
        let rounded = bounds.to_rounded_rect(5.0);

        ctx.stroke(rounded, &env.get(druid::theme::PRIMARY_DARK), 1.0);        
        ctx.fill(rounded, &env.get(theme::BACKGROUND_LIGHT));
        
        let mut tekst = "".into();
        let mut t_size = 24.0;
        let mut offset = 10.0;
        let mut t_color = Color::rgb8(0xEE, 0xEE, 0xEE);
        match self.cell.get_state() {
            CellState::Solved(v, actor)   => {
                    t_color = match actor { 
                        CellActor::StartValue => Color::rgb8(0xEE, 0xEE, 0xEE),
                        _                     => Color::rgb8(0x22, 0x80, 0xEE),
                    };
                    tekst = format!("{}",v.trailing_zeros()+1 );
            },
            CellState::UnSolved(u) => { 
                let mut shifted = u;
                for c in 0..CELL_SIZE {
                    tekst.push(  if  shifted & 0x1 == 1 { HEX_DIGITS[c] }   else  {' ' } );
                    tekst.push(' ');
                    shifted = shifted >> 1;
                    match c {
                        2 => tekst += "               \n" ,
                        5 => tekst += "               \n",
                        _ => ()
                    }
                };
                offset = 30.0;
                t_size = 16.0;
                t_color = Color::rgb8(0xEE, 0x22, 0x22);
            },
            CellState::Error => { 
                tekst = "Error".to_string();
                offset = 15.0;
                t_size = 20.0;
                t_color = Color::rgb8(0xFF, 0x00, 0x00);
            }
        }
        // This is the builder-style way of drawing text.
        let text = ctx.text();
        let layout = text
            .new_text_layout(tekst)
            .font(FontFamily::SERIF, t_size)
            .text_color(t_color)
            .build()
            .unwrap();
        ctx.draw_text(&layout, (size.width/2.0 -offset, size.height/2.0 - offset));
    }
} // CellWidget impl


fn make_numberselect_menu (cell:RcSudokuCell) -> Menu<AppState> {
    let state = cell.get_state();
    let mut menu = Menu::empty();

    for i in 1 ..10 {
        // do the clone outside the closure, clone inside the closure will take the original with into the closure
        let cellclone = cell.clone();
        menu = menu.entry(  
            MenuItem::new(LocalizedString::new( HEX_STRS[i] ) )
                .on_activate( move |_, data:&mut AppState, _| {
                    cellclone.set_init_value(i);
                    data.do_reduce();
                })    
                .enabled( evaluate_cellstate(&state, i) )
            );
    }
    menu
}

    

fn evaluate_cellstate(state:&CellState, value:usize) -> bool {
    let mask = 1 << (value -1);
    match state {
        CellState::Solved(_,_) => false,
        CellState::UnSolved(m)  => m & mask == mask, 
        CellState::Error       => false

    }
} 

fn ui_build_statusline() -> impl Widget<AppState> {
    Flex::row()
    .with_spacer(5.0)
    .with_child( Label::new(|data: &String, _env: &_| data.clone())
        .with_text_size(16.0)
        .lens(AppState::selected)
        .padding(1.0)
    )
    .with_flex_spacer(10.0)
    .with_child( Label::new(|data: &SudokuState, _env: &_| format!("{}", data.get_init_count() ) ) 
        .with_text_size(16.0)
        .lens(AppState::su_state)
    )
    .with_flex_spacer(10.0)
    .with_child( Label::new(|data: &SudokuState, _env: &_| format!("{}", data.get_curr_count() ) ) 
        .with_text_size(16.0)
        .lens(AppState::su_state)
    )
    .with_flex_spacer(10.0)
    .with_child( Label::new(|data: &SudokuState, _env: &_| format!("{}", data.get_step_count() ) ) 
        .with_text_size(16.0)
        .lens(AppState::su_state)
    )
    .with_flex_spacer(10.0)
    .with_child( Label::new(|data: &String, _env: &_| data.clone())
        .with_text_size(16.0)
        .lens(AppState::message)
        .padding(1.0)
    )
    .with_spacer(5.0)
}

#[warn(bare_trait_objects)]
fn _status_label<T:Data> ( text: Box<dyn Fn(&T, &Env) -> String + 'static>)  -> impl Widget<T>  {
    let painter = Painter::new(|ctx, _, env| {
        let bounds = ctx.size().to_rect().inset(-1.0);
        let rounded = bounds.to_rounded_rect(5.0);

        ctx.fill(rounded, &env.get(theme::BACKGROUND_LIGHT));
        ctx.stroke(rounded, &Color::rgb8(0x80, 0x80, 0x80), 1.0);

        if ctx.is_active() {
            ctx.fill(rounded, &Color::rgb8(0x71, 0x71, 0x1));
        }
    });
    Label::new( text)
        .with_text_size(16.0)
        .with_text_color(Color::WHITE)
        .center()
        .background(painter)
        .expand()
}