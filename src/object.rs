use serde::{Serialize, Deserialize};

use tcod::colors::*;
use tcod::console::*;

use crate::map::{self, Map};
use crate::{MAP_HEIGHT, MAP_WIDTH};
use crate::components::*;


#[derive(Debug, Serialize, Deserialize)]
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
    pub item: Option<Item>,
    pub level: i32
}

pub struct ObjectBuilder {
    x: i32,
    y: i32,
    skin: char,
    color: Color,
    name: String,
    blocks: bool,
    alive: bool,
    fighter: Option<Fighter>,
    ai: Option<Ai>,
    item: Option<Item>,
    level: i32
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
            item: None,
            level: 0
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

    pub fn take_damage(&mut self, amount: i32) -> Option<i32> {
        if let Some(fighter) = self.fighter.as_mut() {
           fighter.hp -= amount;

           if fighter.hp <= 0 {
               self.alive = false;
               let xp: i32 = fighter.xp;
               fighter.on_death.callback(self);
               return Some(xp);
           }
        }

        None
    }

    pub fn attack(&mut self, target: &mut Object, msg: &mut crate::renderer::Messages) {
        let damage = (self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense)).abs();
        
        if damage > 0 {
            msg.add(
                format!("{} attacks {} for {} points", self.name, target.name, damage),
                LIGHTER_CRIMSON
            );
            if let Some(xp) = target.take_damage(damage) {
                self.fighter.as_mut().unwrap().xp += xp;
                msg.add(
                    format!("+{} XP", xp),
                    ORANGE
                );
            }
        } else {
            msg.add(
                format!("{} misses {}", self.name, target.name),
                LIGHTER_CRIMSON
            );
        }
    }

    pub fn heal(&mut self, amount: i32) {
        if let Some(fighter) = self.fighter.as_mut() {
            fighter.hp = (fighter.hp + amount).min(fighter.max_hp);
        }
    }
}

impl ObjectBuilder {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            skin: '?',
            color: WHITE,
            name: "N/A".into(),
            blocks: false,
            alive: false,
            fighter: None,
            ai: None,
            item: None,
            level: 0
        }
    }

    pub fn pos(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn skin(mut self, skin: char) -> Self {
        self.skin = skin;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    pub fn blocks(mut self, blocks: bool) -> Self {
        self.blocks = blocks;
        self
    }

    pub fn alive(mut self, alive: bool) -> Self {
        self.alive = alive;
        self
    }

    pub fn fighter(mut self, fighter: Fighter) -> Self {
        self.fighter = Some(fighter);
        self
    }

    pub fn ai(mut self, ai: Ai) -> Self {
        self.ai = Some(ai);
        self
    }
    
    pub fn item(mut self, item: Item) -> Self {
        self.item = Some(item);
        self
    }

    pub fn level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    pub fn build(self) -> Object {
        Object { 
            x: self.x, 
            y: self.y, 
            skin: self.skin, 
            color: self.color, 
            name: self.name, 
            blocks: self.blocks, 
            alive: self.alive, 
            fighter: self.fighter, 
            ai: self.ai, 
            item: self.item, 
            level: self.level 
        }
    }
}