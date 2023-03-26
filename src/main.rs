use std::{env, thread, time};
extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

#[derive(Debug, Clone, Copy)]
enum Cell {
    Alive,
    Dead,
}

type Board = [[Cell; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

const BOARD_WIDTH: usize = 49;
const BOARD_HEIGHT: usize = 40;
const CELL_SIZE: usize = 16;

fn count_neighbours(board: &Board, row: usize, coll: usize) -> usize {
    let mut result: usize = 0;

    for delta_row in -1..=1 {
        for delta_coll in -1..=1 {
            let find_row = (row as i32 + delta_row + BOARD_HEIGHT as i32) as usize % BOARD_HEIGHT;
            let find_col = (coll as i32 + delta_coll + BOARD_WIDTH as i32) as usize % BOARD_WIDTH;

            if find_row == row && find_col == coll {
                continue;
            }

            let cell = board[find_row][find_col];

            match cell {
                Cell::Alive => result += 1,
                Cell::Dead => continue,
            }
        }
    }

    result
}

fn next(board: &Board) -> Board {
    let mut new_board = board_init();

    for (x, row) in board.into_iter().enumerate() {
        for (y, cell) in row.into_iter().enumerate() {
            let neighbours = count_neighbours(board, x, y);

            new_board[x][y] = match cell {
                Cell::Alive => {
                    if neighbours < 2 || neighbours > 3 {
                        Cell::Dead
                    } else {
                        Cell::Alive
                    }
                }
                Cell::Dead => {
                    if neighbours == 3 {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                }
            };
        }
    }

    return new_board;
}

fn board_init() -> Board {
    return [[Cell::Dead; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
}

///   |x|
///     |x|
/// |x|x|x|
///
fn fill_glider(board: &mut Board) {
    board[0][1] = Cell::Alive;
    board[1][2] = Cell::Alive;
    board[2][0] = Cell::Alive;
    board[2][1] = Cell::Alive;
    board[2][2] = Cell::Alive;
}

fn print_board(board: &Board) {
    for row in board {
        for cell in row {
            match cell {
                Cell::Alive => print!("X"),
                Cell::Dead => print!("."),
            }
        }
        println!();
    }
}

fn console() {
    let mut board = board_init();
    fill_glider(&mut board);

    let ten_millis = time::Duration::from_millis(100);
    loop {
        print_board(&board);
        print!("\x1b[{}A", BOARD_HEIGHT);
        print!("\x1b[{}D", BOARD_WIDTH);

        board = next(&mut board);

        thread::sleep(ten_millis);
    }
}

fn render_grid(
    canvas: &mut Canvas<Window>,
    rows: i32,
    colls: i32,
    cellsize: i32,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

    for i in 0..rows {
        canvas.draw_line(
            Point::new(0, i * cellsize),
            Point::new(colls * cellsize, i * cellsize),
        )?;
    }

    for i in 0..colls {
        canvas.draw_line(
            Point::new(i * cellsize, 0),
            Point::new(i * cellsize, rows * cellsize),
        )?;
    }

    Ok(())
}

fn render_cell(canvas: &mut Canvas<Window>, x: i32, y: i32, cellsize: u32) -> Result<(), String> {
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.fill_rect(Rect::new(x, y, cellsize, cellsize))?;

    Ok(())
}

fn graphical() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // the window is the representation of a window in your operating system,
    // however you can only manipulate properties of that window, like its size, whether it's
    // fullscreen, ... but you cannot change its content without using a Canvas or using the
    // `surface()` method.
    let window = video_subsystem
        .window(
            "Game of Life",
            (CELL_SIZE * BOARD_WIDTH) as u32,
            (CELL_SIZE * BOARD_HEIGHT) as u32,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    // the canvas allows us to both manipulate the property of the window and to change its content
    // via hardware or software rendering. See CanvasBuilder for more info.
    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    // clears the canvas with the color we set in `set_draw_color`.
    canvas.clear();
    // However the canvas has not been updated to the window yet, everything has been processed to
    // an internal buffer, but if we want our buffer to be displayed on the window, we need to call
    // `present`. We need to call this everytime we want to render a new frame on the window.
    canvas.present();

    let mut board = board_init();
    fill_glider(&mut board);

    let mut event_pump = sdl_context.event_pump()?;
    let mut frame: u32 = 0;
    let mut running = false;

    'running: loop {
        // get the inputs here
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    repeat: false,
                    ..
                } => {
                    running = !running;
                }
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    let row = (y / CELL_SIZE as i32) as usize;
                    let coll = (x / CELL_SIZE as i32) as usize;

                    let cell = board[row][coll];

                    board[row][coll] = match cell {
                        Cell::Alive => Cell::Dead,
                        Cell::Dead => Cell::Alive,
                    }
                }
                _ => {}
            }
        }

        // update the game loop here
        if frame >= 30 {
            board = next(&mut board);
            frame = 0;
        }

        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        canvas.clear();
        render_grid(
            &mut canvas,
            BOARD_HEIGHT as i32,
            BOARD_WIDTH as i32,
            CELL_SIZE as i32,
        )?;

        for (x, row) in board.into_iter().enumerate() {
            for (y, cell) in row.into_iter().enumerate() {
                match cell {
                    Cell::Alive => {
                        render_cell(
                            &mut canvas,
                            y as i32 * CELL_SIZE as i32,
                            x as i32 * CELL_SIZE as i32,
                            CELL_SIZE as u32,
                        )?;
                    }
                    Cell::Dead => {}
                }
            }
        }

        //this spell must be called each time
        canvas.present();

        if running {
            frame += 1;
        }
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "sdl2" {
        return graphical();
    }

    console();

    Ok(())
}
