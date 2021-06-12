use std::sync::Arc;
use druid::widget::prelude::*;
use druid::{
    AppLauncher, Color, Data,  RenderContext, Widget, WidgetExt,
    WindowDesc, Env, PaintCtx, LocalizedString, MenuItem, MenuDesc, ContextMenu,
    theme,
};
use druid::widget::{Either,Flex, Label, Painter, Button, TextBox, List};
use druid::piet::{FontFamily, Text, TextLayoutBuilder};

extern crate ini;

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
        .with_flex_child(
            Either::new(
                |data, _env| data.which,
                List::new(|| {
                    Label::new(|data: &String, _: &_| format!("Board: {}", data))
                        .center()
                        .background(Color::hlc(230.0, 50.0, 50.0))
                        .expand_width()
                        .padding(5.0)
                        .on_click(| ctx, data, _env| 
                            ctx.submit_command(COMMAND_SELECTED.with( data.to_string() ) ) ) 
                })
                .lens(AppState::board_list), 
                ui_build_board(&*board)
            ),10.0)
        .with_spacer(5.0)
        .with_child(ui_build_statusline() )
        .with_spacer(5.0)
        .controller(SudokuController {file:Option::None})
} // ui_builder

fn ui_build_menuitems() -> impl Widget<AppState> {
    Flex::row()
    .with_child(TextBox::new()
    .lens(AppState::selected)
        .on_click(|ctx, _data, _env| 
            ctx.submit_command(COMMAND_SELECT.with( "".to_string() )  ) 
        )   
    )
    .with_child(Button::new("Select board").on_click(|ctx, _data, _env| 
        ctx.submit_command(COMMAND_SELECT.with( "".to_string() )  ) )   
    )
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
                ctx.show_context_menu(ContextMenu::new( make_numberselect_menu(self.cell.clone()), mouse.pos));
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
        let mut t_color = Color::WHITE; 
        let mut t_size = 24.0;
        let mut offset = 10.0;
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
                        2 => tekst.push('\n'),
                        5 => tekst.push('\n'),
                        _ => ()
                    }
                };
                offset = 30.0;
                t_size = 16.0;
                t_color = Color::rgb8(0xEE, 0x22, 0x22);
            },
        };
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


fn make_numberselect_menu (cell:RcSudokuCell) -> MenuDesc<AppState> {
    let state = &cell.get_state();
    MenuDesc::empty()
        .append_if( MenuItem::new(LocalizedString::new("1") , 
            COMMAND_NUMBER.with( (cell.clone(), 1 ) ) ),
            ||  evaluate_cellstate(state, 1)
        )
        .append_if( MenuItem::new(LocalizedString::new("2") , 
            COMMAND_NUMBER.with( (cell.clone(), 2 ) ) ) ,
            ||  evaluate_cellstate(state, 2)
        )
        .append_if( MenuItem::new(LocalizedString::new("3") , 
            COMMAND_NUMBER.with( (cell.clone(), 3 ) ) ) ,
            ||  evaluate_cellstate(state, 3)
        )
        .append_if( MenuItem::new(LocalizedString::new("4") , 
            COMMAND_NUMBER.with( (cell.clone(), 4 ) ) ) ,
            ||  evaluate_cellstate(state, 4)
        )
        .append_if( MenuItem::new(LocalizedString::new("5") , 
            COMMAND_NUMBER.with( (cell.clone(), 5 ) ) ) ,
            ||  evaluate_cellstate(state, 5)
        )
        .append_if( MenuItem::new(LocalizedString::new("6") , 
            COMMAND_NUMBER.with( (cell.clone(), 6 ) ) ) ,
            ||  evaluate_cellstate(state, 6)
        )
        .append_if( MenuItem::new(LocalizedString::new("7") , 
            COMMAND_NUMBER.with( (cell.clone(), 7 ) ) ) ,
            ||  evaluate_cellstate(state, 7)
        )
        .append_if( MenuItem::new(LocalizedString::new("8") , 
            COMMAND_NUMBER.with( (cell.clone(), 8 ) ) ) ,
            ||  evaluate_cellstate(state, 8)
        )
        .append_if( MenuItem::new(LocalizedString::new("9") , 
            COMMAND_NUMBER.with( (cell.clone(), 9 ) ) ) ,
            ||  evaluate_cellstate(state, 9)
        )
}

fn evaluate_cellstate(state:&CellState, value:usize) -> bool {
    let mask = 1 << (value -1);
    match state {
        CellState::Solved(_,_) => false,
        CellState::UnSolved(m) => m & mask == mask 
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
    .with_child( Label::new(|data: &String, _env: &_| data.clone())
        .with_text_size(16.0)
        .lens(AppState::start_count_s)
        .padding(1.0)
    )
    .with_flex_spacer(10.0)
    .with_child( Label::new(|data: &String, _env: &_| data.clone() ) 
        .with_text_size(16.0)
        .lens(AppState::curr_count_s)
    )
    .with_flex_spacer(10.0)
    .with_child( Label::new(|data: &String, _env: &_| data.clone() ) 
        .with_text_size(16.0)
        .lens(AppState::steps_s)
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