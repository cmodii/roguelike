use tcod::colors::*;
use tcod::console::*;

use crate::map::{self, Map};
use crate::{MAP_HEIGHT, MAP_WIDTH};
use crate::components::*;


#[derive(Debug)]
pub struct Object {
    x: i32,
    y: i32,
    pub skin: char,
    pub color: Color,
    pub name: String,
    pub blocks: bool,
    pub alive: bool,
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
    pub item: Option<Item>
}

impl Object {
    pub fn new(x: i32, y: i32, skin: char, color: Color, name: &str, blocks: bool) -> Self {
        Self {
            x: x,
            y: y,
            skin: skin,
            color: color,
            name: name.into(),
            blocks: blocks,
            alive: false,
            fighter: None,
            ai: None,
            item: None
        }
    }

    pub fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
        let (x, y): (i32, i32) = objects[id].pos();
        if (y + dy) < MAP_HEIGHT && (y + dy) >= 0 // within bounds checks
            && (x + dx) < MAP_WIDTH && (x + dx) >= 0
            && !map::is_blocked(x + dx, y + dy, map, objects) {
                objects[id].set_pos(x+dx, y+dy);
            }
    }

    pub fn move_towards(id: usize, x: i32, y: i32, map: &Map, objects: &mut [Object]) {
        let dx = x - objects[id].get_x();
        let dy = y - objects[id].get_y();
        let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

        let dx = (dx as f32 / distance).round() as i32;
        let dy = (dy as f32 / distance).round() as i32;
        Object::move_by(id, dx, dy, map, objects);
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

    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;

        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.skin, BackgroundFlag::None);
    }

    pub fn take_damage(&mut self, amount: i32) {
        if let Some(fighter) = self.fighter.as_mut() {
           fighter.hp -= amount;

           if fighter.hp <= 0 {
               self.alive = false;
               fighter.on_death.callback(self);
           }
        }
    }

    pub fn attack(&mut self, target: &mut Object, msg: &mut crate::renderer::Messages) {
        let damage = (self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense)).abs();
        msg.add(
            format!("{} attacks {} for {} points", self.name, target.name, damage),
            LIGHTER_CRIMSON
        );
        match damage {
            d if d > 0 => target.take_damage(d),
            0 => {},
            _ => {}
        }
    }

    pub fn heal(&mut self, amount: i32) {
        if let Some(fighter) = self.fighter.as_mut() {
            fighter.hp = (fighter.hp + amount).min(fighter.max_hp);
        }
    }
}
