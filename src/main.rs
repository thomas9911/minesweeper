extern crate structopt;
extern crate piston_window;
extern crate rand;
extern crate rusttype;

use piston_window::*;
use rusttype::Font;
use structopt::StructOpt;

mod tile;
use tile::{Tile, Tiles};

mod counter;
use counter::CountDown;


const FLAG_CHAR: &'static str = "F";
const BOMB_CHAR: &'static str = "B";
const MASK_CHAR: &'static str = "X";
const CLICK_SPEED: u32 = 100;

#[derive(StructOpt, Debug)]
#[structopt(name = "minesweeper")]
struct Opt{
    #[structopt(short = "w", long = "width", default_value = "1000")]
    width: u32,
    #[structopt(short = "h", long = "height", default_value = "500")]
    height: u32,
    #[structopt(short = "c", long = "cols", default_value = "20")]
    cols: u32,
    #[structopt(short = "r", long = "rows", default_value = "10")]
    rows: u32,
    #[structopt(short = "m", long = "mines", default_value = "10")]
    mines: u32,
}

fn main() {
    let opt = Opt::from_args();
    let mut mineboard = Board::new(opt.width, opt.height, opt.cols, opt.rows, opt.mines);
    while run_minesweeper(&mut mineboard){
        mineboard = Board::new(opt.width, opt.height, opt.cols, opt.rows, opt.mines);
    }
}

#[derive(Debug)]
struct Board{
    grid: grid::Grid,
    pub width: u32,
    pub height: u32,
    pub cols: u32,
    pub rows: u32,
    pub mine_count: u32,
    pub lost: bool,
    pub win: bool,
    tiles: Tiles,
    pub scale: f64,
    counter: CountDown
}

impl Board{
    pub fn new(width: u32, height: u32, cols: u32, rows: u32, mines: u32) -> Board{
        let scale1: f64 = width as f64 / cols as f64;
        let scale2: f64 = height as f64 / rows as f64;
        let scale = scale1.min(scale2);

        let mut tiles = Tiles::new(cols as usize, rows as usize);
        tiles.set_mines(mines);
        let grids = grid::Grid {
                cols: cols,
                rows: rows,
                units: scale,
            };

        let counter = CountDown::new(CLICK_SPEED);
        let win = false;
        let lost = false;

        Board{tiles, width, height, cols, rows, scale, counter, mine_count: mines, win, lost, grid: grids}
    }

    fn grid_coords_from_mouse_coords(&mut self, mouse_input: [u32; 2]) -> [u32; 2] {
        let x = mouse_input[0] as f64 / self.scale;
        let y = mouse_input[1] as f64 / self.scale;
        [x as u32, y as u32]
    }

    fn prepare_character(cell: &Tile) -> String {
        match cell.revealed {
            true => match cell.mine {
                true => BOMB_CHAR.to_string(),
                false => match cell.mines_around {
                    0 => " ".to_string(),
                    _ => cell.mines_around.to_string(),
                },
            },
            false => match cell.flagged {
                true => FLAG_CHAR.to_string(),
                false => MASK_CHAR.to_string(),
            },
        }
    }

    fn print_characters(
        &mut self,
        glyphs: &mut Glyphs,
        c: Context,
        g: &mut piston_window::G2d,
    ) {
        for row in self.tiles.tiles.iter() {
            for cell in row.iter() {
                let prepared_character = Board::prepare_character(&cell);
                let color = match prepared_character.as_str() {
                    FLAG_CHAR => [0.0, 0.0, 1.0, 1.0],
                    MASK_CHAR => [0.0, 0.0, 0.0, 1.0],
                    BOMB_CHAR => [1.0, 0.0, 1.0, 1.0],
                    _ => [1.0, 0.0, 0.0, 1.0],
                };
                text::Text::new_color(color, self.scale as u32)
                    .draw(
                        prepared_character.as_ref(),
                        glyphs,
                        &c.draw_state,
                        c.transform.trans(
                            self.scale * (cell.x as f64 + 0.15),
                            self.scale * (cell.y as f64 + 0.85),
                        ),
                        g,
                        ).unwrap();
            }
        };
    }

    fn check_lost(&mut self) -> bool {
        for row in self.tiles.tiles.iter() {
            for cell in row.iter() {
                if cell.mine & cell.revealed {
                    return true;
                }
            }
        }
        false
    }

    fn check_win_through_flags(&mut self) -> bool {
        for row in self.tiles.tiles.iter() {
            for cell in row.iter() {
                if cell.mine & !cell.flagged {
                    return false;
                }
            }
        }
        true
    }

    fn check_win_through_revealed_tiles(&mut self) -> bool {
        let mut count_revealed = 0;
        for row in self.tiles.tiles.iter() {
            for cell in row.iter() {
                if cell.revealed {
                    count_revealed += 1;
                }
            }
        }
        if (self.cols * self.rows - count_revealed) == self.mine_count {
            return true;
        }
        false
    }

    fn checks(&mut self){
        if self.check_lost() {
            self.lost = true;
        };
        if self.check_win_through_flags() | self.check_win_through_revealed_tiles() {
            self.win = true;
        };
    }

}

fn run_minesweeper(mineboard: &mut Board) -> bool {
    let mut mouse_coords: [u32; 2];
    let mut xy_coords: [u32; 2] = [0, 0];

    let mut window = make_window(mineboard.width, mineboard.height);
    let mut glyphs = match make_font(&window){
        Some(x) => x,
        _ => std::process::exit(0),
    };

    while let Some(e) = window.next() {
        mineboard.counter.tick();
        window.draw_2d(&e, |c, g| {
            let transform = c.transform.trans(mineboard.width as f64 / 4.0, mineboard.height as f64 / 2.0);
            clear([1.0; 4], g);

            mineboard.grid.draw(
                &Line::new([0.0, 0.0, 0.0, 1.0], 1.0),
                &c.draw_state,
                c.transform,
                g,
            );

            mineboard.print_characters(&mut glyphs, c, g);
            if mineboard.win & !mineboard.lost {
                text::Text::new_color([0.0, 0.0, 0.0, 1.0], mineboard.scale as u32)
                    .draw(
                        "WINNN!!!",
                        &mut glyphs,
                        &c.draw_state,
                        transform,
                        g,
                    ).unwrap();
            };
            if mineboard.lost {
                text::Text::new_color([0.0, 0.0, 0.0, 1.0], mineboard.scale as u32)
                    .draw(
                        "LOOSSSEE!!!",
                        &mut glyphs,
                        &c.draw_state,
                        transform,
                        g,
                    ).unwrap();
            };
        });

        if let Some(pos) = e.mouse_cursor_args() {
            mouse_coords = [pos[0] as u32, pos[1] as u32];
            xy_coords = mineboard.grid_coords_from_mouse_coords(mouse_coords);
        }
        if let Some(button) = e.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                if mineboard.counter.has_time_left() {
                    mineboard.tiles.reveal_around(xy_coords[0], xy_coords[1])
                } else {
                    let tile = &mut mineboard.tiles.tiles[xy_coords[0] as usize][xy_coords[1] as usize];
                    tile.reveal();
                }
                mineboard.counter.reset()
            }
            if button == Button::Mouse(MouseButton::Right) {
                let tile = &mut mineboard.tiles.tiles[xy_coords[0] as usize][xy_coords[1] as usize];
                tile.flip_flag();
            }
            if button == Button::Keyboard(Key::R){
                return true;
            }
        };
        mineboard.checks();
    }
    false
}

fn make_window(width: u32, height: u32) -> PistonWindow{
    WindowSettings::new("It\'s mine!", [width, height])
        .exit_on_esc(true)
        .build()
        .unwrap()
}

fn make_font(window: &PistonWindow) -> Option<Glyphs>{
    let font_data: &[u8] = include_bytes!("../assets/GlacialIndifference-Regular.ttf");
    let nice_font: Font<'static> = match Font::from_bytes(font_data) {
        Ok(x) => x,
        Err(x) => {
            println!("{}", x);
            return None;
        }
    };

    let factory = window.factory.clone();
    Some(Glyphs::from_font(nice_font, factory, TextureSettings::new()))
}