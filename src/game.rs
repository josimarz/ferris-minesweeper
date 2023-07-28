use std::{iter::repeat_with, vec};

use rand::{thread_rng, Rng};

pub enum Level {
    Easy,
    Medium,
    Hard,
}

enum Status {
    Stopped,
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug)]
pub enum Tag {
    None,
    Flag,
    Question,
}

enum Direction {
    NW,
    N,
    NE,
    W,
    E,
    SW,
    S,
    SE,
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    pub fn equals(&self, position: &Position) -> bool {
        self.x == position.x && self.y == position.y
    }

    fn adjacent_position(&self, direction: &Direction) -> Position {
        match direction {
            Direction::NW => Position { x: self.x - 1, y: self.y - 1 },
            Direction::N => Position { x: self.x - 1, y: self.y },
            Direction::NE => Position { x: self.x - 1, y: self.y + 1 },
            Direction::W => Position { x: self.x, y: self.y - 1 },
            Direction::E => Position { x: self.x, y: self.y + 1 },
            Direction::SW => Position { x: self.x + 1, y: self.y - 1 },
            Direction::S => Position { x: self.x + 1, y: self.y },
            Direction::SE => Position { x: self.x + 1, y: self.y + 1},
        }
    }
}

struct Square {
    position: Position,
    mined: bool,
    openned: bool,
    tag: Tag,
}

impl Square {
    fn new(position: Position, mined: bool) -> Self {
        Square {
            position,
            mined,
            openned: false,
            tag: Tag::None,
        }
    }

    fn open(&mut self) -> bool {
        if !self.openned && matches!(self.tag, Tag::None) {
            self.openned = true;
        }
        self.openned
    }

    fn tag_it(&mut self) -> Tag {
        self.tag = match self.tag {
            Tag::None => Tag::Flag,
            Tag::Flag => Tag::Question,
            Tag::Question => Tag::None,
        };
        self.tag
    }

    fn matches_position(&self, position: &Position) -> bool {
        self.position.equals(position)
    }

    fn is_adjacent(&self, position: &Position) -> bool {
        let directions = vec![
            Direction::NW,
            Direction::N,
            Direction::NE,
            Direction::W,
            Direction::E,
            Direction::SW,
            Direction::S,
            Direction::SE,
        ];
        directions.iter().find(|direction| {
            let adjacent_pos = position.adjacent_position(direction);
            self.matches_position(&adjacent_pos)
        }).is_some()
    }
}

pub struct Board {
    rows: u8,
    cols: u8,
    mines: u8,
    squares: Vec<Square>,
}

impl Board {
    fn new(rows: u8, cols: u8, mines: u8) -> Self {
        Board {
            rows,
            cols,
            mines,
            squares: Vec::with_capacity(usize::from(rows) * usize::from(cols)),
        }
    }

    fn build_mine_map(&self) -> Vec<Vec<bool>> {
        let mut mine_map = repeat_with(||
                repeat_with(|| false)
                    .take(usize::from(self.cols))
                    .collect::<Vec<_>>()
            )
            .take(usize::from(self.rows))
            .collect::<Vec<_>>();

        let mut count = 0;

        while count < self.mines {
            let x = thread_rng().gen_range(0..usize::from(self.rows));
            let y = thread_rng().gen_range(0..usize::from(self.cols));
            if !mine_map[x][y] {
                mine_map[x][y] = true;
                count += 1;
            }
        }

        mine_map
    }

    fn build(&mut self) {
        let mine_map = self.build_mine_map();

        for x in 0..self.rows {
            for y in 0..self.cols {
                let position = Position::new(i32::from(x), i32::from(y));
                let mined = mine_map[usize::from(x)][usize::from(y)];
                let square = Square::new(position, mined);
                self.squares.push(square);
            }
        }
    }

    fn open_adjacencies(&mut self, position: &Position) -> Vec<Position> {
        self.squares
            .iter_mut()
            .for_each(|square| {
                if !square.openned && square.is_adjacent(position) {
                    square.open();
                }
            });
        self.squares
            .iter()
            .filter(|square| square.openned && square.is_adjacent(position))
            .map(|square| square.position)
            .collect::<Vec<_>>()
    }

    fn count_adjacent_mines(&self, position: &Position) -> usize {
        self.squares.iter().filter(|square| square.is_adjacent(position) && square.mined).count()
    }

    fn open_square(&mut self, position: &Position) -> Vec<Position> {
        let found = self.squares.iter_mut().find(|square| square.matches_position(position) && !square.openned);
        if let Some(square) = found {
            if square.open() {
                if !square.mined && self.count_adjacent_mines(position) == 0 {
                    let mut openned_squares = self.open_adjacencies(position);
                    openned_squares.push(*position);
                    return openned_squares;
                }
            }
        }
        vec![*position]
    }

    fn tag_square(&mut self, position: &Position) -> Tag {
        let found = self.squares.iter_mut().find(|square| square.matches_position(position));
        if let Some(square) = found {
            return square.tag_it();
        }
        Tag::None
    }

    fn won(&self) -> bool {
        self.squares.iter().filter(|square| !square.mined && !square.openned).count() == 0
    }

    fn lost(&self) -> bool {
        self.squares.iter().filter(|square| square.mined && square.openned).count() > 0
    }

    fn is_mined(&self, position: &Position) -> bool {
        let found = self.squares.iter().find(|square| square.matches_position(position) && square.mined);
        found.is_some()
    }
}

pub struct Game {
    level: Level,
    board: Option<Board>,
    status: Status,
}

impl Game {
    pub fn new(level: Level) -> Self {
        Game { level, board: None, status: Status::Stopped }
    }

    pub fn count_rows(&self) -> u8 {
        match self.level {
            Level::Easy => 8,
            Level::Medium => 14,
            Level::Hard => 20,
        }
    }

    pub fn count_cols(&self) -> u8 {
        match self.level {
            Level::Easy => 10,
            Level::Medium => 18,
            Level::Hard => 24,
        }
    }

    fn count_mines(&self) -> u8 {
        match self.level {
            Level::Easy => 10,
            Level::Medium => 40,
            Level::Hard => 99,
        }
    }

    pub fn start(&mut self) {
        let rows = self.count_rows();
        let cols = self.count_cols();
        let mines = self.count_mines();
        let mut board = Board::new(rows, cols, mines);
        board.build();
        self.board = Some(board);
        self.status = Status::Playing;
    }

    pub fn open_square(&mut self, position: &Position) -> Vec<Position> {
        if matches!(self.status, Status::Playing) {
            if let Some(board) = self.board.as_mut() {
                let openned_positions = board.open_square(position);
                if board.won() {
                    self.status = Status::Won;
                }
                if board.lost() {
                    self.status = Status::Lost;
                }
                return openned_positions;
            }
        }
        vec![]
    }

    pub fn tag_square(&mut self, position: &Position) -> Tag {
        if let Some(board) = self.board.as_mut() {
            return board.tag_square(position);
        }
        Tag::None
    }

    pub fn count_adjacent_mines(&mut self, position: &Position) -> usize {
        match self.board.as_mut() {
            Some(board) => board.count_adjacent_mines(position),
            None => 0,
        }
    }

    pub fn is_over(&self) -> bool {
        match self.status {
            Status::Stopped => false,
            Status::Playing => false,
            Status::Lost => true,
            Status::Won => true,
        }
    }

    pub fn is_mined(&self, position: &Position) -> bool {
        match self.board.as_ref() {
            Some(board) => board.is_mined(position),
            None => false,
        }
    }
}