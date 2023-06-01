use int_enum::IntEnum;
use crate::Direction::{East, North, West, South};

#[repr(u8)]
#[derive(PartialEq, Eq, Debug,Copy,Clone, IntEnum)]
pub enum Direction {
    North   = 0,
    East    = 1,
    South   = 2,
    West    = 3,
}

pub struct Robot {
    x: i32,
    y: i32,
    d: Direction
}

impl Robot {
    pub fn new(x: i32, y: i32, d: Direction) -> Self {
        Self {
            x,
            y,
            d
        }
    }

    #[must_use]
    pub fn turn_right(self) -> Self {
        Self {
            //x : self.x + 1,
            d : match Direction::from_int(((self.d.int_value() as i32 + 1) % 4) as u8 ){
                Ok(d) => d,
                Err(e) => panic!("{}",e)
            },
            ..self
        }
    }

    #[must_use]
    pub fn turn_left(self) -> Self {
        Self {
            d : match Direction::from_int(((self.d.int_value() as i32 - 1) % 4) as u8 ){
                Ok(d) => d,
                Err(e) => panic!("{}",e)
            },
            ..self
        }
    }

    #[must_use]
    pub fn advance(self) -> Self {
        Self {
            x : match self.d {
                East => self.x + 1,
                West => self.x - 1,
                _ => self.x
            },
            y : match self.d {
                North => self.y + 1,
                Direction::South => self.y - 1,
                _ => self.y
            },
            ..self
        }
    }

    #[must_use]
    pub fn instructions(&mut self, instructions: &str) {
        //unimplemented!("Follow the given sequence of instructions: {instructions}")
        let instructions_vec = instructions.chars();

        //head-extraction
        for c in instructions_vec.rev() {
            match c {
                'L' => {
                    self.d = match  Direction::from_int((( self.d.int_value() as i32 - 1) % 4) as u8 ) {
                        Ok(d) => d,
                        Err(e) => panic!("{}",e)
                    };
                },
                'R' => {
                    self.d = match Direction::from_int((( self.d.int_value() as i32 + 1) % 4) as u8 ){
                        Ok(d) => d,
                        Err(e) => panic!("{}",e)
                    };
                },
                'A' => {
                    match self.d {
                        North => {
                            self.y += 1;
                        }
                        East => {
                            self.x += 1;
                        }
                        Direction::South => {
                            self.y -= 1;
                        }
                        West => {
                            self.x -= 1;
                        }
                    }
                },
                _ => {  }
            }
        }

    }

    pub fn position(&self) -> (i32, i32) {
        return  (self.x, self.y);
    }

    pub fn direction(&self) -> &Direction {
        return &self.d;
    }
}
