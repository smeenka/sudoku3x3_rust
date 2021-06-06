use std::rc::Rc;
use std::sync::Arc;
use druid::{
    AppLauncher, Color, Data, Lens,  LensExt,  RenderContext, Widget, WidgetExt,
    WindowDesc, Env,
    theme
};
use druid::widget::{CrossAxisAlignment, Flex, Label, List, Painter, Button};

use sudoku3x3::{
    controller::{SudokuController, },
    sudoku::{SudokuBoard, Row, RcSudokuCell, CellState, CellActor },
    data::*
};

pub fn main() {
    let app_state = AppState::new();
    let board = app_state.board.clone();
    let window = WindowDesc::new(move || ui_builder(board))
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
        .with_flex_child( ui_build_board(&*board),  20. )
        .with_spacer(5.0)
        .with_child(ui_build_statusline() )
        .with_spacer(5.0)
        .controller(SudokuController)
} // ui_builder

fn ui_build_menuitems() -> impl Widget<AppState> {
    Flex::row()
    .with_child(Button::new("Restart").on_click(|ctx, _data, _env| 
        ctx.submit_command(COMMAND_INIT.with( "".to_string() )  ) )   
    )
    .with_flex_spacer(1.0)
    .with_child(Button::new("Step").on_click(|ctx, _data, _env| 
            ctx.submit_command(COMMAND_STEP.with( "".to_string()  ) ) )
    )
    .with_flex_spacer(1.0)
    .with_child(Button::new("Slowmotion").on_click(|ctx, _data: &mut AppState, _env| 
        ctx.submit_command(COMMAND_SLOWMOTION.with( "".to_string()  ) ) )  
    )
    .with_flex_spacer(1.0)
    .with_child(Button::new("Solve").on_click(| ctx, _data, _env| 
        ctx.submit_command(COMMAND_SOLVE.with( "".to_string() ) ) ) 
    )  
    .with_flex_spacer(1.0)
}

fn ui_build_board<T:Data> (board:&SudokuBoard) -> impl Widget<T> {
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
fn build_row<T:Data>(row:&Row) -> impl Widget<T> {
    let mut frow = Flex::row(); 
    for r in 0 .. 9 {    
        
        let arccells = &row.cells;

        frow.add_flex_child(  build_cell(arccells[r].clone() ), 1.0);
        match r {
            2 => {frow.add_default_spacer()},
            5 => {frow.add_default_spacer()},
            _ => ()
        }
    }
    frow
}//build_row
    
fn build_cell<T:Data> (cell: RcSudokuCell) -> impl Widget<T> {
    let mut color = Color::WHITE;
    let mut size = 24.0;
    match cell.get_state() {
        CellState::Solved(_, _actor)   => {
            color = Color::YELLOW;
            size = 24.0;
        },
        CellState::UnSolved(_u) => { 
            color = Color::Rgba32(0xF0F0F0F0);
            size = 14.0;
        },
    }


    active_label(color, size, Box::new(move |&_, &_| {
        match cell.get_state() {
            CellState::Solved(v, _actor)   => {
                    format!("{}",v)
            },
            CellState::UnSolved(u) => { 
                let mut result = String::new();
                let mut shifted = u;
                for c in 0..CELL_SIZE {
                    result.push(  if  shifted & 0x1 == 1 { HEX_DIGITS[c] }   else  {' ' } );
                    result.push(' ');
                    shifted = shifted >> 1;
                    match c {
                        2 => result.push('\n'),
                        5 => result.push('\n'),
                        _ => ()
                    }
                };
                format!("{}",result)
            },
        }
    }))             
} //build_cell



/*
    Flex::row()
        .with_child( 
            List::new( build_row) 
            //.horizontal()
            .lens(AppState::lens_rows)
        )
    .with_child( 
            Label::new(|data: &String, _env: &_| data.clone())
                .with_text_size(16.0)
                .lens(Row::row_message)
                .padding(1.0)
        )
    
}
*/
/*
fn build_cell() -> impl Widget<Rc<SudokuRefCell>> {
    Label::new(|data: &String, _env: &_| data.clone())
        .with_text_size(16.0)
 //       .lens(SudokuRefCell::message)
        .padding(1.0)
    
    .with_child(List::new(build_row)
        .with_spacing(5.)
        .padding(10.)
        .lens(AppState::SudokuBoard::RowsX);

    Flex::row()
    .with_child(
        List::new(todo_item)
        .with_spacing(5.)
        .padding(10.)
        .lens(AppState::todos);
        
        Label::new("To Be Done") )
    .with_child( Label::new(|data: &String, _env: &_| data.clone())
    .with_text_size(16.0)
    .lens(AppState::message) )
    .padding(1.0)
*/

fn ui_build_statusline() -> impl Widget<AppState> {
    Flex::row()
    .with_spacer(10.0)
    .with_child( Label::new(|data: &String, _env: &_| data.clone())
        .with_text_size(16.0)
        .lens(AppState::start_count_s)
        .padding(1.0)
    )
    .with_spacer(10.0)
    .with_child( Label::new(|data: &String, _env: &_| data.clone() ) 
        .with_text_size(16.0)
        .lens(AppState::curr_count_s)
    )
    .with_spacer(10.0)
    .with_child( Label::new(|data: &String, _env: &_| data.clone() ) 
        .with_text_size(16.0)
        .lens(AppState::steps_s)
    )
    .with_spacer(10.0)
    .with_child( Label::new(|data: &String, _env: &_| data.clone())
        .with_text_size(16.0)
        .lens(AppState::message)
        .padding(1.0)
    )
    .with_flex_spacer(10.0)
}



#[warn(bare_trait_objects)]
fn active_label<T:Data> ( color:Color, size:f64, text: Box<dyn Fn(&T, &Env) -> String + 'static>)  -> impl Widget<T>  {
    let painter = Painter::new(|ctx, _, env| {
        let bounds = ctx.size().to_rect();

        ctx.fill(bounds, &env.get(theme::BACKGROUND_LIGHT));

        if ctx.is_hot() {
            ctx.stroke(bounds.inset(-0.5), &Color::WHITE, 1.0);
        }

        if ctx.is_active() {
            ctx.fill(bounds, &Color::rgb8(0x71, 0x71, 0x1));
        }
    });
    Label::new( text)
        .with_text_size(size)
        .with_text_color(color)
        .center()
        .background(painter)
        .expand()
        .on_click( |ctx, _data: &mut T, _env| {
            println!("Clicked label");
            ctx.request_paint();
            ctx.request_update();
        })
    
}    


/*
fn digit_button(digit: u8) -> impl Widget<AppState> {
    let painter = Painter::new(|ctx, _, env| {
        let bounds = ctx.size().to_rect();

        ctx.fill(bounds, &env.get(theme::BACKGROUND_LIGHT));

        if ctx.is_hot() {
            ctx.stroke(bounds.inset(-0.5), &Color::WHITE, 1.0);
        }

        if ctx.is_active() {
            ctx.fill(bounds, &Color::rgb8(0x71, 0x71, 0x71));
        }
    });

    Label::new(format!("{}", digit))
        .with_text_size(24.)
        .center()
        .background(painter)
        .expand()
        .on_click(move |_ctx, data: &mut AppState, _env| print!("Clicked"))
}
*/