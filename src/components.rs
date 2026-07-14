use crate::object::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeathCallback {
    Player,
    Monster,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
    pub base_max_hp: i32,
    pub hp: i32,
    pub base_defense: i32,
    pub base_power: i32,
    pub xp: i32,
    pub on_death: DeathCallback
}

impl Default for Fighter {
    fn default() -> Self {
        Fighter {
           hp: 30,
           base_max_hp: 30,
           base_defense: 0,
           base_power: 5,
           xp: 0,
           on_death: DeathCallback::Player
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Ai {
    Basic,
    Confused {
        previous_ai: Box<Ai>,
        num_turns: i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Heal,
    StopTime,
    Lightning,
    Confuse,
    Equipment
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    pub slot: Slot,
    pub equipped: bool,
    pub power_bonus: i32,
    pub defense_bonus: i32,
    pub max_hp_bonus: i32
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Slot {
    LeftHand,
    RightHand,
    Body
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Slot::LeftHand => write!(f, "left hand"),
            Slot::RightHand => write!(f, "right hand"),
            Slot::Body => write!(f, "Body")
        }
    }
}

impl DeathCallback {
    pub fn callback(self, object: &mut Object) {
        use DeathCallback::*;
        let callback: fn(&mut Object) = match self {
            Player => player_death,
            Monster => monster_death
        };
        callback(object);
    }
}

fn player_death(player: &mut Object) {
    player.skin = '%';
    player.color = tcod::colors::DARK_RED;
}

fn monster_death(monster: &mut Object) {
    monster.skin = '%';
    monster.color = tcod::colors::DARK_RED;
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("remains of {}", monster.name);
}