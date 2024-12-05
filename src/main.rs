use crossterm::{
    cursor::Hide,
    execute,
    terminal::{size, Clear, DisableLineWrap},
};
use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{stdin, stdout, Write},
};

const UP_OR_DOWN: &str = "│";
const LEFT_OR_RIGHT: &str = "─";
const BOTTOM_RIGHT: &str = "┌";
const BOTTOM_LEFT: &str = "┐";
const TOP_RIGHT: &str = "└";
const TOP_LEFT: &str = "┘";
const TOP_RIGHT_BOTTOM: &str = "├";
const TOP_BOTTOM_LEFT: &str = "┤";
const RIGHT_BOTTOM_LEFT: &str = "┬";
const TOP_RIGHT_LEFT: &str = "┴";
const ALL: &str = "┼";

fn main() {
    execute!(
        stdout(),
        Clear(crossterm::terminal::ClearType::Purge),
        Clear(crossterm::terminal::ClearType::All),
        Hide,
        DisableLineWrap
    )
    .expect("Error initializing terminal");
    let (up, left, down, right) = (Command::UP, Command::LEFT, Command::DOWN, Command::RIGHT);
    let mut cursor = Point { x: 0, y: 0 };
    let mut canvas: HashMap<Point, Cell> = HashMap::new();
    let mut commands: Vec<Command> = vec![];

    'main: loop {
        execute!(
            stdout(),
            Clear(crossterm::terminal::ClearType::Purge),
            Clear(crossterm::terminal::ClearType::All)
        )
        .expect("Error resetting terminal");

        println!("{}", get_canvas_string(&canvas, Some(cursor)));
        println!("WASD keys to move, H for help, C to clear, \nF to save, or Q to quit.");
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("Error reading command input");
        input = input.trim().to_uppercase();

        if input == "Q" {
            println!("Thanks for playing!");
            break 'main;
        } else if input == "H" {
            println!("Enter W, A, S, and D characters to move the cursor and");
            println!("draw a line behind it as it moves. For example, ddd");
            println!("draws a line going right and sssdddwwwaaa draws a box.");
            println!("You can save your drawing to a text file by entering F.\n");
            println!("Press Enter to return to the program...");
            let mut buf = String::new();
            stdin()
                .read_line(&mut buf)
                .expect("Error returning to menu");
            continue;
        } else if input == "C" {
            canvas.clear();
            commands.push(Command::CLEAR);
        } else if input == "F" {
            println!("Enter valid name for file without extension.");
            let mut filename = String::new();
            stdin()
                .read_line(&mut filename)
                .expect("Error reading filename input");

            filename = filename.trim().to_lowercase();
            if !filename.ends_with(".txt") {
                filename += ".txt";
            }

            let mut file = File::create(filename).expect("Error creating file.");

            let contents = commands
                .iter()
                .map(|command| command.to_char())
                .collect::<String>()
                + "\n";

            file.write_all(contents.as_bytes())
                .expect("Error saving command data");
            file.write_all(get_canvas_string(&canvas, None).as_bytes())
                .expect("Error saving canvas data");
        }
        for char in input.chars() {
            if [
                up.to_char(),
                left.to_char(),
                down.to_char(),
                right.to_char(),
            ]
            .contains(&char)
            {
                let command = match char {
                    'W' => Some(up),
                    'A' => Some(left),
                    'S' => Some(down),
                    'D' => Some(right),
                    _ => None,
                }
                .expect("Received invalid command");

                commands.push(command);

                let old_cursor = cursor;
                let moved = cursor.mve(command);

                if canvas.is_empty() && moved {
                    let mut new_cell = Cell {
                        ..Default::default()
                    };
                    new_cell.modify_with_command(command);
                    canvas.insert(old_cursor, new_cell);
                } else if moved {
                    canvas.entry(old_cursor).and_modify(|cell| {
                        cell.modify_with_command(command);
                    });
                }
                if moved {
                    let opposite = command.opposite().expect("Command has no opposite");
                    canvas
                        .entry(cursor)
                        .and_modify(|cell| {
                            cell.modify_with_command(opposite);
                        })
                        .or_insert_with(|| {
                            let mut cell = Cell {
                                ..Default::default()
                            };
                            cell.modify_with_command(opposite)
                        });
                }
            }
        }
    }
}

fn get_canvas_string(canvas: &HashMap<Point, Cell>, cursor: Option<Point>) -> String {
    let mut canvas_str = String::new();
    let (width, height) = size().expect("Error reading terminal size");

    for y in 0..=(height - 5) {
        for x in 0..=(width - 1) {
            if cursor.is_some() {
                if i32::from(x) == cursor.unwrap().x && i32::from(y) == cursor.unwrap().y {
                    canvas_str += "#";
                    continue;
                }
            }

            let result = canvas.get(&Point {
                x: x.into(),
                y: y.into(),
            });

            if result.is_some() {
                let cell = result.unwrap();
                match (cell.top, cell.right, cell.bottom, cell.left) {
                    (true, true, true, true) => canvas_str += ALL,
                    (true, true, true, false) => canvas_str += TOP_RIGHT_BOTTOM,
                    (false, true, true, true) => canvas_str += RIGHT_BOTTOM_LEFT,
                    (true, false, true, true) => canvas_str += TOP_BOTTOM_LEFT,
                    (true, true, false, true) => canvas_str += TOP_RIGHT_LEFT,
                    (true, true, false, false) => canvas_str += TOP_RIGHT,
                    (false, true, true, false) => canvas_str += BOTTOM_RIGHT,
                    (false, false, true, true) => canvas_str += BOTTOM_LEFT,
                    (true, false, false, true) => canvas_str += TOP_LEFT,
                    (true, false, false, false)
                    | (false, false, true, false)
                    | (true, false, true, false) => canvas_str += UP_OR_DOWN,
                    (false, true, false, false)
                    | (false, false, false, true)
                    | (false, true, false, true) => canvas_str += LEFT_OR_RIGHT,
                    (false, false, false, false) => canvas_str += " ",
                }
            } else {
                canvas_str += " ";
            }
        }
        canvas_str += "\n";
    }
    canvas_str
}

#[derive(PartialEq, Copy, Clone)]
enum Command {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    CLEAR,
}
impl Command {
    fn to_char(&self) -> char {
        match self {
            Command::UP => 'W',
            Command::LEFT => 'A',
            Command::DOWN => 'S',
            Command::RIGHT => 'D',
            Command::CLEAR => 'C',
        }
    }
    fn opposite(&self) -> Option<Command> {
        match self {
            Command::UP => Some(Command::DOWN),
            Command::DOWN => Some(Command::UP),
            Command::LEFT => Some(Command::RIGHT),
            Command::RIGHT => Some(Command::LEFT),
            _ => None,
        }
    }
}
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
impl Point {
    fn mve(&mut self, command: Command) -> bool {
        let (width, height) = size().expect("Error reading terminal size");
        let mut moved = false;

        if self.y > 0 && command == Command::UP {
            self.y -= 1;
            moved = true;
        } else if self.y < (height - 6).into() && command == Command::DOWN {
            self.y += 1;
            moved = true;
        } else if self.x > 0 && command == Command::LEFT {
            self.x -= 1;
            moved = true;
        } else if self.x < (width - 2).into() && command == Command::RIGHT {
            self.x += 1;
            moved = true;
        }
        moved
    }
}
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
struct Cell {
    top: bool,
    right: bool,
    bottom: bool,
    left: bool,
}
impl Cell {
    fn modify_with_command(&mut self, command: Command) -> Self {
        match command {
            Command::UP => self.top = true,
            Command::RIGHT => self.right = true,
            Command::DOWN => self.bottom = true,
            Command::LEFT => self.left = true,
            _ => (),
        }
        *self
    }
}
