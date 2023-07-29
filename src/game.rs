use std::iter::repeat_with;

use rand::{thread_rng, Rng};

pub enum Level {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone, Copy)]
pub enum Tag {
    None,
    Flag,
    Question,
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    fn adjacencies(&self) -> Vec<Position> {
        vec![
            Position::new(self.x - 1, self.y - 1),
            Position::new(self.x - 1, self.y),
            Position::new(self.x - 1, self.y + 1),
            Position::new(self.x, self.y - 1),
            Position::new(self.x, self.y + 1),
            Position::new(self.x + 1, self.y - 1),
            Position::new(self.x + 1, self.y),
            Position::new(self.x + 1, self.y + 1),
        ]
    }

    fn adjacent(&self, position: Position) -> bool {
        self.adjacencies().iter().find(|adjacency| **adjacency == position).is_some()
    }
}

struct Square {
    position: Position,
    mined: bool,
    openned: bool,
    tag: Tag,
}

impl PartialEq for Square {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Square {
    fn new(position: Position, mined: bool) -> Self {
        Square { position, mined, openned: false, tag: Tag::None }
    }

    fn openable(&self) -> bool {
        matches!(self.tag, Tag::None) && !self.openned
    }

    fn open(&mut self) -> bool {
        self.openned = self.openable();
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

    fn adjacent(&self, position: Position) -> bool {
        self.position.adjacent(position)
    }
}

struct Board {
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

    fn create_mine_map(&self) -> Vec<Vec<bool>> {
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
        let mine_map = self.create_mine_map();

        for x in 0..self.rows {
            for y in 0..self.cols {
                let position = Position::new(i32::from(x), i32::from(y));
                let mined = mine_map[usize::from(x)][usize::from(y)];
                let square = Square::new(position, mined);
                self.squares.push(square);
            }
        }
    }

    fn won(&self) -> bool {
        self.squares.iter().find(|square| !square.mined && !square.openned).is_none()
    }

    fn lost(&self) -> bool {
        self.squares.iter().find(|square| square.mined && square.openned).is_some()
    }

    fn adjacent_mines(&self, position: Position) -> usize {
        self.squares.iter().filter(|square| square.adjacent(position) && square.mined).count()
    }

    fn openable_positions(&self, position: Position, openned: &mut Vec<Position>) {
        let added = openned
            .iter()
            .find(|openned| **openned == position);
        if added.is_some() {
            return;
        }
        let square = self.squares
            .iter()
            .find(|square| square.position == position);
        if square.is_none() {
            return;
        }
        let square = square.unwrap();
        if !square.openable() {
            return;
        }
        openned.push(position);
        if square.mined {
            return;
        }
        let adjacent_mines = self.adjacent_mines(position);
        if adjacent_mines > 0 {
            return;
        }
        self.squares
            .iter()
            .filter(|square| square.adjacent(position))
            .for_each(|square| {
                self.openable_positions(square.position, openned);
            });
    }

    fn open_position(&mut self, position: Position) -> Vec<Position> {
        let mut positions = Vec::new();
        self.openable_positions(position, &mut positions);
        positions
            .iter()
            .for_each(|position| {
                let square = self.squares
                    .iter_mut()
                    .find(|square| square.position == *position);
                if let Some(square) = square {
                    square.open();
                }
            });
        positions
    }

    fn tag_position(&mut self, position: Position) -> Tag {
        let square = self.squares.iter_mut().find(|square| square.position == position);
        if let Some(square) = square {
            return square.tag_it();
        }
        Tag::None
    }

    fn position_mined(&self, position: Position) -> bool {
        let square = self.squares.iter().find(|square| square.position == position && square.mined);
        square.is_some()
    }
}

pub struct OpennedPosition {
    pub position: Position,
    pub mined: bool,
    pub adjacent_mines: usize,
}

impl OpennedPosition {
    fn new(position: Position, mined: bool, adjacent_mines: usize) -> Self {
        OpennedPosition { position, mined, adjacent_mines }
    }
}

pub struct Game {
    level: Level,
    board: Option<Board>,
}

impl Game {
    pub fn new(level: Level) -> Self {
        Game { level, board: None }
    }

    pub fn rows(&self) -> u8 {
        match self.level {
            Level::Easy => 8,
            Level::Medium => 14,
            Level::Hard => 20,
        }
    }

    pub fn cols(&self) -> u8 {
        match self.level {
            Level::Easy => 10,
            Level::Medium => 18,
            Level::Hard => 24,
        }
    }

    fn mines(&self) -> u8 {
        match self.level {
            Level::Easy => 10,
            Level::Medium => 40,
            Level::Hard => 99,
        }
    }

    pub fn start(&mut self) {
        let mut board = Board::new(self.rows(), self.cols(), self.mines());
        board.build();
        self.board = Some(board);
    }

    pub fn open_position(&mut self, position: Position) -> Vec<OpennedPosition> {
        if let Some(board) = self.board.as_mut() {
            let positions = board.open_position(position);
            return positions
                .iter()
                .map(|position| {
                    let position = Position::new(position.x, position.y);
                    let mined = board.position_mined(position);
                    let adjacent_mines = board.adjacent_mines(position);
                    OpennedPosition::new(position, mined, adjacent_mines)
                })
                .collect::<Vec<_>>()
        }
        vec![]
    }

    pub fn tag_position(&mut self, position: Position) -> Tag {
        if let Some(board) = self.board.as_mut() {
            return board.tag_position(position);
        }
        Tag::None
    }

    pub fn over(&self) -> bool {
        if let Some(board) = self.board.as_ref() {
            return board.won() || board.lost();
        }
        true
    }
}