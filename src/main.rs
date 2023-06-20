use rand::prelude::SliceRandom;

use std::fs::{self, File};
use std::path::Path;
use std::io::{self, Write};

#[derive(Default)]
struct Stack<T: Default + Copy + Clone> {
    items: Vec<T>,
}

impl<T: Default + Copy + Clone> Stack<T> {
    pub fn push(&mut self, val: T) {
        self.items.push(val);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[derive(Default, Copy, Clone)]
struct Cell {
    pub row: usize,
    pub col: usize,
    pub visited: bool,
}

impl Cell {
    pub fn ind(&self) -> usize {
        self.row * MAZE_SIZE + self.col
    }
}

#[derive(Copy, Clone, PartialEq)]
enum NeighborDir {
    Center,
    North,
    South,
    West,
    East,
}

#[derive(Default)]
enum WallKind {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Default)]
struct Wall {
    start: Cell,
    target: Cell,
    kind: WallKind
}

const MAZE_SIZE: usize = 10;

#[derive(Default)]
struct Env {
    grid: [[Cell; MAZE_SIZE]; MAZE_SIZE],
    removed_walls: Vec<Wall>,
}

impl Env {
    fn init() -> Self {
        let mut this = Self::default();
        for r in 0..MAZE_SIZE {
            for c in 0..MAZE_SIZE {
                this.grid[r][c] = Cell {
                    row: r,
                    col: c,
                    visited: false,
                };
            }
        }
        this.removed_walls = vec![];
        this
    }
}

fn in_bound(val: i32, low: i32, high: i32) -> bool {
    (val >= low) && (val < high)
}

fn unvisited_neighbors(grid: &[[Cell; MAZE_SIZE]; MAZE_SIZE], row: usize, col: usize) -> NeighborDir {
    let mut rng = rand::thread_rng();
    let mut directions = [
        NeighborDir::North,
        NeighborDir::South,
        NeighborDir::East,
        NeighborDir::West,
    ];
    // Shuffle the order in which neighboring cells are 'checked'
    directions.shuffle(&mut rng);
    let mut new_row;
    let mut new_col;

    for el in directions.iter() {
        // Reset values for the next direction
        new_row = row as i32;
        new_col = col as i32;

        match el {
            NeighborDir::North => new_row -= 1,
            NeighborDir::South => new_row += 1,
            NeighborDir::West => new_col -= 1,
            NeighborDir::East => new_col += 1,
            NeighborDir::Center => unreachable!(),
        };

        if in_bound(new_row, 0, MAZE_SIZE as i32) &&
            in_bound(new_col, 0, MAZE_SIZE as i32) &&
            !grid[new_row as usize][new_col as usize].visited {
            return *el;
        }
    }
    return NeighborDir::Center;
}

fn remove_wall(walls: &mut Vec<Wall>, start: Cell, target: Cell) {
    assert!(start.ind() != target.ind());
    let row_diff = (start.row as i32) - (target.row as i32);
    let col_diff = (start.col as i32) - (target.col as i32);
    let kind = if row_diff != 0 {
        WallKind::Vertical
        // Original: WallKind::Horizontal
    } else {
        // Original: WallKind::Vertical
        WallKind::Horizontal
    };
    if row_diff > 0 || col_diff > 0 {
        walls.push(Wall { target, start, kind });
    } else if row_diff < 0 || col_diff < 0 {
        walls.push(Wall { start, target, kind });
    }
}

fn gen_maze(env: &mut Env) {
    // Initial random row and col
    let mut row = rand::random::<usize>() % MAZE_SIZE;
    let mut col = rand::random::<usize>() % MAZE_SIZE;
    let mut current = env.grid[row][col];
    // Mark current cell as visited
    env.grid[row][col].visited = true;
    
    // Initialize a separate stack
    let mut stack = Stack::<Cell>::default();
    // Push random initial cell to the stack
    stack.push(current);

    while stack.len() > 0 {
        // Pop cell from the stack
        current = stack.pop().unwrap();
        // Update `row` and `col` to the current cell's
        row = current.row;
        col = current.col;
        // Get the direction of a random unvisited neighbor
        let unvisited = unvisited_neighbors(&env.grid, row, col);
        // If unvisited neighbor is center that means all of the current cell's neighbors are visited
        if unvisited == NeighborDir::Center { continue; }
        // Push current cell to the stack
        stack.push(current);

        let mut target_row = row;
        let mut target_col = col;
        match unvisited {
            NeighborDir::North => target_row -= 1,
            NeighborDir::South => target_row += 1,
            NeighborDir::West => target_col -= 1,
            NeighborDir::East => target_col += 1,
            NeighborDir::Center => unreachable!(),
        }
        let target = env.grid[target_row][target_col];
        // Remove wall between current and target cell
        remove_wall(&mut env.removed_walls, current, target);
        // Mark target cell as visited
        env.grid[target_row][target_col].visited = true;
        stack.push(target);
    }
}

const SOLID_COLOR: u32 = 0x32A852;
const OPEN_COLOR: u32 = 0x0;
// const OPEN_COLOR: u32 = 0x2856A1;

const OPEN_PATH_SIZE: u32 = 10;
const BORDER_THICKNESS: u32 = 1;

const IMG_SIZE: usize = (MAZE_SIZE * OPEN_PATH_SIZE as usize) + ((MAZE_SIZE+1) * BORDER_THICKNESS as usize);

fn fill_rect(pixels: &mut [[u32; IMG_SIZE]; IMG_SIZE], rx: u32, ry: u32, rw: u32, rh: u32, color: u32) {
    assert!(rx + rw <= IMG_SIZE as u32);
    assert!(ry + rh <= IMG_SIZE as u32);

    for y in ry..(ry + rh) {
        for x in rx..(rx + rw) {
            pixels[y as usize][x as usize] = color;
        }
    }
}

fn draw_maze(env: &Env, pixels: &mut [[u32; IMG_SIZE]; IMG_SIZE]) {
    let mut y;
    let mut x;
    
    for r in 0..(MAZE_SIZE as u32) {
        for c in 0..=(MAZE_SIZE as u32) {
            x = (c * OPEN_PATH_SIZE) + (c * BORDER_THICKNESS);
            y = (r * OPEN_PATH_SIZE) + (r * BORDER_THICKNESS);
            fill_rect(pixels, x, y, BORDER_THICKNESS, OPEN_PATH_SIZE + (2*BORDER_THICKNESS), SOLID_COLOR);
        }
    }

    for r in 0..=(MAZE_SIZE as u32) {
        for c in 0..(MAZE_SIZE as u32) {
            x = (c * OPEN_PATH_SIZE) + (c * BORDER_THICKNESS);
            y = (r * OPEN_PATH_SIZE) + (r * BORDER_THICKNESS);
            fill_rect(pixels, x, y, OPEN_PATH_SIZE + (2*BORDER_THICKNESS), BORDER_THICKNESS, SOLID_COLOR);
        }
    }

    
    for wall in env.removed_walls.iter() {
        match wall.kind {
            WallKind::Vertical => {
                fill_rect(pixels,
                    ((wall.target.col as u32) * OPEN_PATH_SIZE) + ((wall.target.col as u32) * BORDER_THICKNESS),
                    ((wall.target.row as u32) * OPEN_PATH_SIZE) + ((wall.target.row as u32) * BORDER_THICKNESS) + BORDER_THICKNESS,
                    BORDER_THICKNESS, OPEN_PATH_SIZE, OPEN_COLOR
                );
            },
            WallKind::Horizontal => {
                fill_rect(pixels,
                    ((wall.target.col as u32) * OPEN_PATH_SIZE) + ((wall.target.col as u32) * BORDER_THICKNESS) + BORDER_THICKNESS,
                    ((wall.target.row as u32) * OPEN_PATH_SIZE) + ((wall.target.row as u32) * BORDER_THICKNESS),
                    OPEN_PATH_SIZE, BORDER_THICKNESS, OPEN_COLOR
                );
            },
        }
    }
   
}

fn save_as_ppm(pixels: &[[u32; IMG_SIZE]; IMG_SIZE], filename: &str) -> Result<(), io::Error> {
    if Path::exists(Path::new(filename)) {
        fs::remove_file(filename)?;
    }
    let mut file = File::create(filename)?;

    write!(&mut file, "P6\n{} {} 255\n", IMG_SIZE, IMG_SIZE)?;
    for y in 0..IMG_SIZE {
        for x in 0..IMG_SIZE {
            let pixel = pixels[y][x];
            // Color HEX code format: 0xRRGGBB
            let color_components = [
                ((pixel >> 8*2) & 0xFF) as u8, //     0xRR & 0xFF
                ((pixel >> 8*1) & 0xFF) as u8, //   0x__GG & 0xFF
                ((pixel >> 8*0) & 0xFF) as u8, // 0x____BB & 0xFF
            ];
            file.write(&color_components)?;
        }
    }
    Ok(())
}

fn main() {
    let mut env = Env::init();
    gen_maze(&mut env);
    let mut pixels = [[0u32; IMG_SIZE]; IMG_SIZE];
    draw_maze(&env, &mut pixels);
    if let Err(_) = save_as_ppm(&pixels, "out.ppm") {
        panic!("ERROR: Failed to save maze as ppm");
    }
}
