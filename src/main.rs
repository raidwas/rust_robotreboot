use chrono::prelude::*;
use curl::easy::{Easy, List};
#[macro_use]
extern crate enum_map;
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Read;
use std::thread;

const CELLS: usize = 16;
//const BORDER_WIDTH: usize = 1;

//0 -> empty, 1 -> ⌜, 2 -> ⌝, 3 -> ⌟, 4 -> ⌞; +4 if it may contain a goal
const BOARDS: [[[[u8; 8]; 8]; 3]; 4] = [
    [
        //yellow
        [
            [0, 0, 2, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 8, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 7],
            [4, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 7, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 5, 0],
            [0, 6, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        [
            [0, 0, 0, 0, 0, 1, 0, 0],
            [0, 0, 7, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 8, 0, 0, 0, 0, 0, 0],
            [4, 0, 0, 0, 0, 0, 5, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 6, 0, 0],
            [0, 0, 0, 7, 0, 0, 0, 0],
        ],
        [
            [0, 0, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 8, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 6, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 5, 0, 0],
            [0, 0, 7, 0, 0, 0, 0, 3],
            [4, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
    ],
    [
        //red
        [
            [0, 2, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 6, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 8, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 5, 0, 0],
            [1, 0, 0, 7, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        [
            [0, 0, 0, 2, 0, 0, 0, 0],
            [0, 8, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 6, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 7, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 5],
            [1, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        [
            [0, 0, 0, 2, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 6, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [4, 0, 6, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 8],
            [0, 5, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
    ],
    [
        //green
        [
            [0, 2, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 5, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 7, 0],
            [0, 8, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 6, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        [
            [0, 0, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0, 0, 7, 0],
            [0, 5, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 6, 0],
            [1, 0, 8, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        [
            [0, 2, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 5, 0, 0, 0],
            [0, 6, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 7, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [4, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 8, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
    ],
    [
        //blue
        [
            [0, 0, 0, 2, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 7, 0, 0],
            [0, 8, 0, 0, 0, 0, 0, 0],
            [4, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 7, 0, 0, 5, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 6, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        [
            [0, 0, 0, 0, 0, 1, 0, 0],
            [0, 0, 5, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 8, 0],
            [4, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 6, 0, 0, 0],
            [0, 7, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        [
            [0, 0, 0, 0, 0, 2, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 5, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 8, 0, 0],
            [1, 0, 6, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 7, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
    ],
];

fn main() {
    /*
    let s = Board::hard_puzzle().solve();
    println!("{:?}", s);
    println!("{}", s.len());
    */
    loop {
        //wait for new config
        JsonNextConfigTime::load().wait();
        //retrieve config
        let config = JsonConfig::load();
        let mut total_moves = 0;
        let mut solution_entrys: Vec<JsonSolutionEntry> = Vec::new();
        for board in config.get_boards() {
            let solution = board.solve();
            println!("{:?}", solution);
            let goal_color = String::from(board.goal_color.as_str());
            let mut sol = Vec::new();
            for (c, d) in solution {
                total_moves += 1;
                sol.push(String::from(c.as_str()));
                sol.push(String::from(d.as_str()));
            }
            solution_entrys.push(JsonSolutionEntry {
                goal: (board.goal.y, board.goal.x, goal_color.clone()),
                goalColor: goal_color,
                solution: sol,
            })
        }
        solution_entrys.reverse();
        let json_solution = JsonSolution {
            challengeId: config.challengeId,
            config: config.config.clone(),
            name: String::from("not_a_bot"),
            remainingPuzzle: 0,
            store: solution_entrys,
            totalMoves: total_moves,
        };
        json_solution.publish();
    }
}

#[derive(Deserialize)]
struct JsonNextConfigTime(String);
impl JsonNextConfigTime {
    fn load() -> Self {
        const CONFIG: &str = "http://www.robotreboot.com/challenge/nextconfigtime";
        let mut data = Vec::new();
        let mut handle = Easy::new();
        handle.url(CONFIG).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer
                .write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                })
                .unwrap();
            transfer.perform().unwrap();
        }
        //println!("{:?}", data);
        let json = String::from_utf8(data).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    fn wait(&self) {
        //"2019-08-20T20:00:00.000Z"
        //println!("{}", self.0);
        let now = Utc::now();
        let t = self.0.parse::<DateTime<Utc>>().unwrap();
        let dt = t - now;
        println!(
            "Waiting for the next puzzle in {} seconds",
            dt.num_seconds()
        );
        thread::sleep(dt.to_std().unwrap());
        //panic!();
    }
}

//json represantation of challenge
//can be found under http://www.robotreboot.com/challenge/config
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct JsonConfig {
    _id: String,
    config: String,
    challengeId: u64,
    __v: u64,
    updated_at: String,
    created_at: String,
    challenge_date: String,
    goals: [(u8, u8, String); 5],
    robots: [(u8, u8, String); 4],
}

impl JsonConfig {
    //gets the current challenge from the web
    fn load() -> Self {
        const CONFIG: &str = "http://www.robotreboot.com/challenge/config";
        let mut data = Vec::new();
        let mut handle = Easy::new();
        handle.url(CONFIG).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer
                .write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                })
                .unwrap();
            transfer.perform().unwrap();
        }
        //println!("{:?}", data);
        let json = String::from_utf8(data).unwrap();
        serde_json::from_str(&json).unwrap()
    }
    //parses the self.config field into something more usable.
    //this is then used to construct the actual board from the BOARDS array (using the two indices)
    fn parse_board_config(&self) -> [(u8, u8); 4] {
        let mut ret = [(0, 0); 4];
        let mut c: u64 = u64::from_str_radix(&self.config, 10).unwrap();
        for i in 0..4 {
            ret[3 - i].1 = (c % 10) as u8;
            c /= 10;
            ret[3 - i].0 = (c % 10) as u8;
            c /= 10;
        }
        ret
    }
    //gets a single field according to config
    fn get_tile(config: &[(u8, u8); 4], p: Position) -> u8 {
        //correctly "rotate" the point
        let (b, p) = match (p.x < (CELLS / 2) as u8, p.y < (CELLS / 2) as u8) {
            (true, true) => (0, p),
            (false, true) => (1, Position::new(p.y, (CELLS as u8) - p.x - 1)),
            (true, false) => (2, Position::new((CELLS as u8) - p.y - 1, p.x)),
            (false, false) => (
                3,
                Position::new((CELLS as u8) - p.x - 1, (CELLS as u8) - p.y - 1),
            ),
        };
        //println!("{:?}", p);
        let val = if p.x == 7 && p.y == 7 {
            1
        } else {
            BOARDS[config[b].0 as usize][config[b].1 as usize][p.y as usize][p.x as usize]
        };
        //rotate the value we found
        let mut new_val = val as i8;
        if new_val != 0 {
            match b {
                0 => {}
                1 => new_val += 1,
                2 => new_val -= 1,
                3 => new_val += 2,
                _ => unreachable!(),
            }
            if val > 0 && val < 5 {
                new_val = ((new_val - 1 + 4) % 4) + 1;
            }
            if val > 4 && val < 9 {
                new_val = ((new_val - 5 + 4) % 4) + 5;
            }
        }
        new_val as u8
    }
    //constructs all boards
    fn get_boards(&self) -> Vec<Board> {
        //parse config
        let board_config = self.parse_board_config();
        //first we construct our border arrays
        let mut borders_v: [bool; CELLS * (CELLS - 1)] = [false; CELLS * (CELLS - 1)];
        let mut borders_h: [bool; CELLS * (CELLS - 1)] = [false; CELLS * (CELLS - 1)];
        //set the middle block
        borders_v[7 * (CELLS - 1) + 7] = true;
        borders_v[8 * (CELLS - 1) + 7] = true;
        borders_h[7 * (CELLS - 1) + 7] = true;
        borders_h[8 * (CELLS - 1) + 7] = true;

        for y in 0..CELLS {
            for x in 0..CELLS {
                let mut val = JsonConfig::get_tile(&board_config, Position::new(x as u8, y as u8));
                if val == 0 {
                    continue;
                }
                if val > 4 {
                    val -= 4;
                }
                let (x, y): (i8, i8) = (x as i8, y as i8);
                let vy: i8 = match val {
                    1 => y - 1,
                    2 => y - 1,
                    3 => y,
                    4 => y,
                    _ => unreachable!(),
                };
                let hx: i8 = match val {
                    1 => x - 1,
                    2 => x,
                    3 => x,
                    4 => x - 1,
                    _ => unreachable!(),
                };
                if vy >= 0 && vy < (CELLS - 1) as i8 {
                    borders_v[x as usize * (CELLS - 1) + vy as usize] = true;
                }
                if hx >= 0 && hx < (CELLS - 1) as i8 {
                    borders_h[y as usize * (CELLS - 1) + hx as usize] = true;
                }
            }
        }
        //bot start positions
        let mut starts = EnumMap::new();
        for (y, x, color) in self.robots.iter() {
            let color = Color::from_str(color);
            starts[color] = Position::new(*x, *y);
        }
        //goals
        let mut boards: Vec<Board> = Vec::new();
        for (y, x, color) in self.goals.iter().rev() {
            let goal_p = Position::new(*x, *y);
            let goal_c = Color::from_str(color);
            boards.push(Board {
                borders_v: borders_v,
                borders_h: borders_h,
                goal: goal_p,
                goal_color: goal_c,
                starts: starts,
            });
        }
        boards
    }
}

//json to send to the server to publish solution
#[derive(Serialize)]
#[allow(non_snake_case)]
struct JsonSolution {
    remainingPuzzle: u64,
    totalMoves: u64,
    store: Vec<JsonSolutionEntry>,
    config: String,
    challengeId: u64,
    name: String,
}

impl JsonSolution {
    fn publish(&self) {
        let json = serde_json::to_string(&self).unwrap();
        let mut easy = Easy::new();
        easy.url("http://www.robotreboot.com/challenge/submission")
            .unwrap();
        easy.post(true).unwrap();
        let mut list = List::new();
        list.append("Host: www.robotreboot.com").unwrap();
        list.append(
            "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:68.0) Gecko/20100101 Firefox/68.0",
        )
        .unwrap();
        list.append("Accept: application/json").unwrap();
        list.append("Accept-Language: en-US,en;q=0.5").unwrap();
        //list.append("Accept-Encoding: gzip, deflate").unwrap();
        list.append("Referer: http://www.robotreboot.com/challenge")
            .unwrap();
        list.append("Content-Type: application/json").unwrap();
        list.append("Origin: http://www.robotreboot.com").unwrap();
        //list.append("Content-Length: 722").unwrap();
        list.append("Connection: keep-alive").unwrap();
        easy.http_headers(list).unwrap();
        easy.post_field_size(json.as_bytes().len() as u64).unwrap();
        let mut transfer = easy.transfer();
        transfer
            .read_function(|buf| Ok(json.as_bytes().read(buf).unwrap_or(0)))
            .unwrap();
        transfer.perform().unwrap();
    }
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct JsonSolutionEntry {
    goal: (u8, u8, String),
    goalColor: String,
    solution: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Direction {
    fn as_str(&self) -> &str {
        match self {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Enum)]
enum Color {
    R,
    G,
    B,
    Y,
}
impl Color {
    fn as_str(&self) -> &str {
        match self {
            Color::R => "red",
            Color::G => "green",
            Color::B => "blue",
            Color::Y => "yellow",
        }
    }
    fn from_str(color: &str) -> Self {
        match color {
            "red" => Color::R,
            "green" => Color::G,
            "blue" => Color::B,
            "yellow" => Color::Y,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
struct Position {
    x: u8,
    y: u8,
}

impl Position {
    fn new(x: u8, y: u8) -> Self {
        Position { x, y }
    }
}

struct Board {
    //since there are 16 cells we need 15 "walls" per row
    borders_v: [bool; CELLS * (CELLS - 1)],
    borders_h: [bool; CELLS * (CELLS - 1)],
    goal: Position,
    goal_color: Color,
    starts: EnumMap<Color, Position>,
}

#[derive(Default)]
struct CompressedState {
    //each position can be saved in one u8 => 4*u8
    //there exist at most (16*16)^4 == 1<<32 different configurations, therefore parent_id can be saved in a u32
    //color and direction have at most 16 combinations, u8
    data: [u8; 9],
}

impl CompressedState {
    fn from(state: &State) -> Self {
        let b0 = (state.positions[Color::R].x & 0xF) | (state.positions[Color::R].y << 4);
        let b1 = (state.positions[Color::G].x & 0xF) | (state.positions[Color::G].y << 4);
        let b2 = (state.positions[Color::B].x & 0xF) | (state.positions[Color::B].y << 4);
        let b3 = (state.positions[Color::Y].x & 0xF) | (state.positions[Color::Y].y << 4);

        let b4 = ((state.parent_id >> 0) & 0xFF) as u8;
        let b5 = ((state.parent_id >> 8) & 0xFF) as u8;
        let b6 = ((state.parent_id >> 16) & 0xFF) as u8;
        let b7 = ((state.parent_id >> 24) & 0xFF) as u8;

        let color = match state.color {
            Color::R => 0,
            Color::G => 1,
            Color::B => 2,
            Color::Y => 3,
        };
        let direction = match state.dir {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Left => 2,
            Direction::Right => 3,
        };
        let b8 = (color << 2) | direction;
        CompressedState {
            data: [b0, b1, b2, b3, b4, b5, b6, b7, b8],
        }
    }
    fn get_comp_position(&self) -> u32 {
        ((self.data[0] as u32) << 0)
            | ((self.data[1] as u32) << 8)
            | ((self.data[2] as u32) << 16)
            | ((self.data[3] as u32) << 24)
    }
    fn get_parent_id(&self) -> usize {
        ((self.data[4] as usize) << 0)
            | ((self.data[5] as usize) << 8)
            | ((self.data[6] as usize) << 16)
            | ((self.data[7] as usize) << 24)
    }
    fn into_state(&self) -> State {
        let rx = self.data[0] & 0xF;
        let ry = self.data[0] >> 4;
        let gx = self.data[1] & 0xF;
        let gy = self.data[1] >> 4;
        let bx = self.data[2] & 0xF;
        let by = self.data[2] >> 4;
        let yx = self.data[3] & 0xF;
        let yy = self.data[3] >> 4;
        let parent_id = self.get_parent_id();
        let color = match (self.data[8] & 0xF) >> 2 {
            0 => Color::R,
            1 => Color::G,
            2 => Color::B,
            3 => Color::Y,
            _ => unreachable!(),
        };
        let direction = match self.data[8] & 0b11 {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => unreachable!(),
        };
        State {
            parent_id: parent_id,
            color: color,
            dir: direction,
            positions: enum_map! {
                Color::R => Position::new(rx, ry),
                Color::G => Position::new(gx, gy),
                Color::B => Position::new(bx, by),
                Color::Y => Position::new(yx, yy),
            },
        }
    }
}

#[derive(Clone)]
struct State {
    //saves current positions of the colors
    parent_id: usize,
    //how to get from parent to self
    color: Color,
    dir: Direction,
    positions: EnumMap<Color, Position>,
}

impl State {
    fn set_position(&mut self, c: Color, p: Position) {
        self.positions[c] = p;
    }
    fn get_position(&self, c: Color) -> Position {
        self.positions[c]
    }
    fn is_free(&self, p: Position) -> bool {
        for pos in self.positions.values() {
            if p == *pos {
                return false;
            }
        }
        return true;
    }
    fn new_position(&self, current: Position, d: Direction, board: &Board) -> Position {
        let mut current = current;
        //println!("a {:?}", current);
        if d == Direction::Up {
            let walls = &board.borders_v;
            while current.y > 0
                && self.is_free(Position::new(current.x, current.y - 1))
                && !walls[(current.x * (CELLS - 1) as u8 + current.y - 1) as usize]
            {
                current.y -= 1;
            }
        } else if d == Direction::Down {
            let walls = &board.borders_v;
            while current.y < (CELLS - 1) as u8
                && self.is_free(Position::new(current.x, current.y + 1))
                && !walls[(current.x * (CELLS - 1) as u8 + current.y) as usize]
            {
                current.y += 1;
            }
        } else if d == Direction::Left {
            let walls = &board.borders_h;
            while current.x > 0
                && self.is_free(Position::new(current.x - 1, current.y))
                && !walls[(current.y * (CELLS - 1) as u8 + current.x - 1) as usize]
            {
                current.x -= 1;
            }
        } else if d == Direction::Right {
            let walls = &board.borders_h;
            //println!("b {:?}", current);
            while current.x < (CELLS - 1) as u8
                && self.is_free(Position::new(current.x + 1, current.y))
                && !walls[(current.y * (CELLS - 1) as u8 + current.x) as usize]
            {
                //println!("c {:?}", current);
                current.x += 1;
            }
        }
        current
    }
    fn turn(&self, self_id: usize, c: Color, d: Direction, board: &Board) -> Option<Self> {
        let pos = self.get_position(c);
        let new_pos = self.new_position(pos, d, board);
        if new_pos == pos {
            None
        } else {
            let mut ret = self.clone();
            ret.set_position(c, new_pos);
            ret.parent_id = self_id;
            ret.color = c;
            ret.dir = d;
            Some(ret)
        }
    }
}

struct CompactBoolArray {
    size: usize,
    vec: Vec<u8>,
}

impl CompactBoolArray {
    fn new(size: usize) -> Self {
        let mut vec: Vec<u8> = Vec::new();
        let need_u8 = (size + 7) / 8;
        //println!("compact bool array needs {} u8", need_u8);
        vec.resize_with(need_u8, || 0);
        CompactBoolArray {
            size: size,
            vec: vec,
        }
    }
    fn is_set(&self, ind: usize) -> bool {
        assert!(ind < self.size);
        let a = ind / 8;
        let b = ind - (a * 8);
        let d = self.vec[a];
        (d >> b) & 0b1 == 1
    }
    fn set(&mut self, ind: usize) {
        assert!(ind < self.size);
        let a = ind / 8;
        let b = ind - (a * 8);
        let d = &mut self.vec[a];
        *d |= 1 << b;
    }
}

impl Board {
    fn solve(&self) -> Vec<(Color, Direction)> {
        //println!("solve");
        let state = State {
            parent_id: 0,
            color: Color::R,
            dir: Direction::Up,
            positions: self.starts,
        };
        //println!("creating compact bool array");
        let mut known = CompactBoolArray::new(1 << 32);
        //println!("done");
        let mut vec = vec![CompressedState::from(&state)];
        known.set(vec[vec.len() - 1].get_comp_position() as usize);
        let mut current_state_ind = 0;
        let mut current_depth = 1;
        let mut current_depth_end = vec.len();
        //println!("entering loop");
        loop {
            if current_depth_end == current_state_ind {
                println!("No solution found with {} moves", current_depth);
                current_depth += 1;
                current_depth_end = vec.len();
                println!("Next depth len: {}", current_depth_end - current_state_ind);
            }
            let state: State = vec[current_state_ind].into_state();
            for c in &[Color::R, Color::G, Color::B, Color::Y] {
                for d in &[
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    if let Some(new_state) = state.turn(current_state_ind, *c, *d, &self) {
                        let cstate = CompressedState::from(&new_state);
                        if !known.is_set(cstate.get_comp_position() as usize) {
                            vec.push(CompressedState::from(&new_state));
                            known.set(cstate.get_comp_position() as usize);
                            //check if we finished and if yes construct path
                            if new_state.get_position(self.goal_color) == self.goal {
                                let mut ret = Vec::new();
                                current_state_ind = vec.len() - 1;
                                while current_state_ind != 0 {
                                    let state = vec[current_state_ind].into_state();
                                    ret.push((state.color, state.dir));
                                    current_state_ind = state.parent_id;
                                }
                                ret.reverse();
                                return ret;
                            }
                        }
                    }
                }
            }
            current_state_ind += 1;
        }
    }
    fn hard_puzzle() -> Self {
        let mut borders_v = [false; CELLS * (CELLS - 1)];
        let mut borders_h = [false; CELLS * (CELLS - 1)];
        for (y, x) in [
            (0, 14),
            (1, 2),
            (2, 11),
            (3, 1),
            (3, 6),
            (3, 15),
            (4, 0),
            (5, 5),
            (6, 7),
            (6, 8),
            (6, 10),
            (6, 13),
            (7, 3),
            (8, 3),
            (8, 7),
            (8, 8),
            (8, 14),
            (10, 11),
            (10, 15),
            (11, 1),
            (11, 9),
            (12, 6),
            (13, 0),
            (13, 2),
            (14, 13),
        ]
        .iter()
        {
            borders_v[(x * (CELLS - 1) as u8 + y) as usize] = true;
        }
        for (y, x) in [
            (0, 4),
            (0, 9),
            (1, 2),
            (1, 13),
            (2, 10),
            (3, 0),
            (4, 5),
            (6, 5),
            (6, 13),
            (7, 3),
            (7, 6),
            (7, 8),
            (7, 10),
            (8, 6),
            (8, 8),
            (9, 3),
            (9, 13),
            (10, 10),
            (11, 0),
            (12, 6),
            (12, 9),
            (14, 1),
            (14, 13),
            (15, 5),
            (15, 10),
        ]
        .iter()
        {
            borders_h[(y * (CELLS - 1) as u8 + x) as usize] = true;
        }
        Board {
            borders_v: borders_v,
            borders_h: borders_h,
            goal: Position::new(9, 12),
            goal_color: Color::B,
            starts: enum_map! {
                Color::R => Position::new(02,14),
                Color::G => Position::new(00,3),
                Color::B => Position::new(11,2),
                Color::Y => Position::new(02,1),
            },
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn hard_puzzle() {
        let puzzle = super::Board::hard_puzzle();
        assert!(puzzle.solve().len() == 25);
    }
}
