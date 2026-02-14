use tcod::colors::*;
use tcod::console::*;

use crate::Game;
use crate::map::get_index;
use crate::{MAP_HEIGHT, MAP_WIDTH};

#[derive(Debug)]
pub struct Object {
    x: i32,
    y: i32,
    skin: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, skin: char, color: Color) -> Self {
        Self { x, y, skin, color }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if (self.y + dy) < MAP_HEIGHT && (self.y + dy) >= 0 // within bounds checks
            && (self.x + dx) < MAP_WIDTH && (self.x + dx) >= 0
            && !game.map[get_index(self.x + dx, self.y + dy)].is_blocked() {
            self.x += dx;
            self.y += dy;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.skin, BackgroundFlag::None);
    }
}
