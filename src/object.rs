use serde::{Serialize, Deserialize};

use tcod::colors::*;
use tcod::console::*;

use crate::game::Game;
use crate::renderer::Messages;
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
    pub level: i32,
    pub equipment: Option<Equipment>
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
    equipment: Option<Equipment>,
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
            level: 0,
            equipment: None
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

    pub fn attack(&mut self, target: &mut Object, game: &mut Game) {
        let damage = (self.power(game) - target.defense(game)).max(0);
        
        if damage > 0 {
            game.messages.add(
                format!("{} attacks {} for {} points", self.name, target.name, damage),
                LIGHTER_CRIMSON
            );
            if let Some(xp) = target.take_damage(damage) {
                self.fighter.as_mut().unwrap().xp += xp;
                game.messages.add(
                    format!("+{} XP", xp),
                    ORANGE
                );
            }
        } else {
            game.messages.add(
                format!("{} misses {}", self.name, target.name),
                LIGHTER_CRIMSON
            );
        }
    }

    pub fn heal(&mut self, amount: i32, game: &Game) {
        let max_hp = self.max_hp(game);
        if let Some(fighter) = self.fighter.as_mut() {
            fighter.hp = (fighter.hp + amount).min(max_hp);
        }
    }

    pub fn equip(&mut self, messages: &mut Messages) {
        if self.item.is_some() && let Some(ref mut equipment) = self.equipment {
            if !equipment.equipped {
                messages.add(format!("Equipped {} on {}", self.name, equipment.slot), WHITE);
                equipment.equipped = true;
            }
        }
    }

    pub fn unequip(&mut self, messages: &mut Messages) {
        if self.item.is_some() && let Some(ref mut equipment) = self.equipment {
            if equipment.equipped {
                messages.add(format!("Unequipped {}", self.name), WHITE);
                equipment.equipped = false;
            }
        }
    }
    
    pub fn get_all_equipped(&self, game: &Game) -> Vec<Equipment> {
        if self.name.eq("player") {
            game.inventory
                .iter()
                .filter(|item| item.equipment.is_some_and(|e| e.equipped))
                .map(|item| item.equipment.unwrap())
                .collect()
        } else {
            vec![]
        }
    }

    pub fn power(&self, game: &Game) -> i32 {
        let base_power = self.fighter.map_or(0, |f| f.base_power);
        let bonus_power = self
            .get_all_equipped(game)
            .iter()
            .map(|e| e.power_bonus)
            .sum::<i32>();

        base_power + bonus_power
    }

    pub fn defense(&self, game: &Game) -> i32 {
        let base_defense = self.fighter.map_or(0, |f| f.base_defense);
        let bonus_defense = self
            .get_all_equipped(game)
            .iter()
            .map(|e| e.defense_bonus)
            .sum::<i32>();

        base_defense + bonus_defense
    }

    pub fn max_hp(&self, game: &Game) -> i32 {
        let base_max_hp = self.fighter.map_or(0, |f| f.base_max_hp);
        let bonus_max_hp = self
            .get_all_equipped(game)
            .iter()
            .map(|e| e.max_hp_bonus)
            .sum::<i32>();

        base_max_hp + bonus_max_hp
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
            equipment: None,
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

    pub fn equipment(mut self, equipment: Equipment) -> Self {
        self.equipment = Some(equipment);
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
            equipment: self.equipment,
            level: self.level 
        }
    }
}