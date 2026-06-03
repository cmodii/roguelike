use tcod::colors::*;
use tcod::console::*;

use crate::Game;
use crate::map;
use crate::{MAP_HEIGHT, MAP_WIDTH};


#[derive(Debug)]
pub struct Object {
    x: i32,
    y: i32,
    skin: char,
    color: Color,
    pub name: String,
    blocks: bool,
    alive: bool
}

impl Object {
    pub fn new(x: i32, y: i32, skin: char, color: Color, name: &str, blocks: bool, alive: bool) -> Self {
        Self {
            x: x,
            y: y,
            skin: skin,
            color: color,
            name: name.into(),
            blocks: blocks,
            alive: alive
        }
    }

    pub fn move_by(id: usize, dx: i32, dy: i32, game: &Game, objects: &mut [Object]) {
        let (x, y): (i32, i32) = objects[id].pos();
        if (y + dy) < MAP_HEIGHT && (y + dy) >= 0 // within bounds checks
            && (x + dx) < MAP_WIDTH && (x + dx) >= 0
            && !map::is_blocked(x + dx, y + dy, &game.map, objects) {
                objects[id].set_pos(x+dx, y+dy);
            }
    }

    pub fn is_alive(&self) -> bool {
        self.alive
    }

    pub fn can_block(&self) -> bool {
        self.blocks
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.skin, BackgroundFlag::None);
    }
}
