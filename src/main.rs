extern crate piston_window;
extern crate rusttype;
extern crate rand;

use piston_window::*;
use rusttype::Font;

mod tile;
use tile::{Tile, Tiles};

mod counter;
use counter::CountDown;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 500;
const COLS: u32 = 20;
const ROWS: u32 = 10;
const MINES: u32 = 15;

const FLAG_CHAR: &'static str = "F";
const BOMB_CHAR: &'static str = "B";
const MASK_CHAR: &'static str = "X";

const SCALE1: f64 = WIDTH as f64 / COLS as f64;
const SCALE2: f64 = HEIGHT as f64 / ROWS as f64;

fn get_scale() -> f64{
    SCALE1.min(SCALE2)
}

fn main(){
    println!("Start {}x{}: with scale {}", WIDTH, HEIGHT, get_scale());
    println!("Board {}x{}: with {} mines", COLS, ROWS, MINES);
    run_minesweeper();
}

fn run_minesweeper() {
    let mut tiles = Tiles::new(COLS as usize, ROWS as usize);
    tiles.set_mines(MINES);
    let mut win = false;
    let mut lost = false;
    let mut counter = CountDown::new(100); // double click sensitivity 50 - 150

    let mut window: PistonWindow =
        WindowSettings::new("It\'s mine!", [WIDTH, HEIGHT])
        .exit_on_esc(true).build().unwrap();
    let font_data: &[u8] = include_bytes!("../assets/GlacialIndifference-Regular.ttf");
    let nice_font: Font<'static> = match Font::from_bytes(font_data){
        Ok(x) => x,
        Err(x) => {println!("{}", x); return}
    };

    let factory = window.factory.clone();
    let mut glyphs = Glyphs::from_font(nice_font, factory, TextureSettings::new());
    // let mut glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();
    let scale = get_scale();
    let gridss = grid::Grid{cols: COLS, rows: ROWS, units: scale};
    let mut mouse_coords: [u32; 2];
    let mut xy_coords: [u32; 2] = [0,0];

    while let Some(e) = window.next() {
        counter.tick();
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);

            gridss.draw(&Line::new([0.0,0.0,0.0,1.0], 1.0), &c.draw_state, c.transform, g);

            for row in tiles.tiles.iter(){
                for cell in row.iter(){
                    print_character(cell, scale, &mut glyphs, c, g)
                }
            };
            if win & !lost{
                text::Text::new_color([0.0, 0.0, 0.0, 1.0], scale as u32).draw(
                            "WINNN!!!",
                            &mut glyphs,
                            &c.draw_state,
                            c.transform.trans(WIDTH as f64 /4.0, HEIGHT as f64 /2.0), g
                        ).unwrap();
            }
            if lost{
                text::Text::new_color([0.0, 0.0, 0.0, 1.0], scale as u32).draw(
                            "LOOSSSEE!!!",
                            &mut glyphs,
                            &c.draw_state,
                            c.transform.trans(WIDTH as f64 /4.0, HEIGHT as f64 /2.0), g
                        ).unwrap();
            }
        });
        if let Some(pos) = e.mouse_cursor_args() {
                mouse_coords = [pos[0] as u32, pos[1] as u32];
                xy_coords = grid_coords_from_mouse_coords(mouse_coords);
            }
        if let Some(button) = e.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                if counter.has_time_left(){
                    tiles.reveal_around(xy_coords[0], xy_coords[1])
                } else {
                    let tile = &mut tiles.tiles[xy_coords[0] as usize][xy_coords[1] as usize];
                    tile.reveal();
                }
                counter.reset()
            }
            if button == Button::Mouse(MouseButton::Right){
                let tile = &mut tiles.tiles[xy_coords[0] as usize][xy_coords[1] as usize];
                tile.flip_flag();
            }
        };
        if check_lost(&tiles){
            lost = true;
        };
        if check_win_through_flags(&tiles) | check_win_through_revealed_tiles(&tiles){
            win = true;
        };
    }
}


fn grid_coords_from_mouse_coords(mouse_input: [u32; 2]) -> [u32; 2]{
    let x = mouse_input[0] as f64 / get_scale();
    let y = mouse_input[1] as f64 / get_scale();
    [x as u32, y as u32]
}

fn prepare_character(cell: &Tile) -> String{
    match cell.revealed{
        true => match cell.mine{
            true => BOMB_CHAR.to_string(),
            false => match cell.mines_around{
                0 => " ".to_string(),
                _ => cell.mines_around.to_string()
            }
        },
        false => match cell.flagged {
            true => FLAG_CHAR.to_string(),
            false => MASK_CHAR.to_string()
        }
    }
}

fn print_character(cell: &Tile, scale: f64, glyphs: &mut Glyphs, c: Context, g: &mut piston_window::G2d){
    let prepared_character = prepare_character(&cell);
    let color = match prepared_character.as_str(){
        FLAG_CHAR => [0.0, 0.0, 1.0, 1.0],
        MASK_CHAR => [0.0, 0.0, 0.0, 1.0],
        BOMB_CHAR => [1.0, 0.0, 1.0, 1.0],
        _ => [1.0, 0.0, 0.0, 1.0]
    };
    text::Text::new_color(color, scale as u32)
                    .draw(
                        prepared_character.as_ref(),
                        glyphs,
                        &c.draw_state,
                        c.transform.trans(scale*(cell.x as f64 + 0.15), scale*(cell.y as f64 + 0.85)),
                        g
                    ).unwrap();
}


fn check_lost(board: &Tiles) -> bool{
    for row in board.tiles.iter(){
        for cell in row.iter(){
            if cell.mine & cell.revealed{
                return true
            }
        }
    }
    false
}

fn check_win_through_flags(board: &Tiles) -> bool{
    for row in board.tiles.iter(){
        for cell in row.iter(){
            if cell.mine & !cell.flagged{
                return false
            }
        }
    }
    true
}

fn check_win_through_revealed_tiles(board: &Tiles) -> bool{
    let mut count_revealed = 0;
    for row in board.tiles.iter(){
        for cell in row.iter(){
            if cell.revealed{
                count_revealed+=1;
            }
        }
    }
    if (COLS*ROWS - count_revealed) == MINES{
        return true
    }
    false
}
