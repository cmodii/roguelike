use crate::object::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeathCallback {
    Player,
    Monster,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub on_death: DeathCallback
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
    Confuse
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
