use crate::game::next_level;
use crate::object::Object;
use crate::components::Fighter;
use crate::renderer::{msgbox, menu};
use crate::{Tcod, Game};
use crate::util::*;
use crate::inventory::{pick_item_up, drop_item, use_item, inventory_menu};

use tcod::input::{Key, KeyCode};

pub const PLAYER: usize = 0;

const LEVEL_UP_SCREEN_WIDTH: i32 = 40;
const CHARACTER_SCREEN_WIDTH: i32 = 40; 

const LEVEL_UP_INITIAL: i32 = 200;
const LEVEL_UP_FACTOR: i32 = 150;
const HP_LEVELUP_INCREASE: i32 = 20;
const ATTACK_LEVELUP_INCREASE: i32 = 1;
const DEFENSE_LEVELUP_INCREASE: i32 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit
}

pub fn level_up(tcod: &mut Tcod, _game: &mut Game, objects: &mut [Object]) {
    let player: &mut Object = &mut objects[PLAYER];
    let player_fighter: &mut Fighter = player.fighter.as_mut().unwrap();
    let level_up_xp: i32 = LEVEL_UP_INITIAL + player.level * LEVEL_UP_FACTOR;
    
    if player_fighter.xp >= level_up_xp {
        player.level += 1;
        player_fighter.xp -= level_up_xp;
        
        let choice: Option<usize> = menu(
            "Choose a stat to raise:", 
            &[
                format!("Vigor ({} HP -> {} HP)", player_fighter.max_hp, player_fighter.max_hp + HP_LEVELUP_INCREASE),
                format!("Strength (+1 attack, {} -> {}", player_fighter.power, player_fighter.power + ATTACK_LEVELUP_INCREASE),
                format!("Agility (+1 defense, {} -> {}", player_fighter.defense, player_fighter.defense + DEFENSE_LEVELUP_INCREASE)
            ], 
            LEVEL_UP_SCREEN_WIDTH, 
            &mut tcod.root
        );

        match choice {
            Some(0) => player_fighter.max_hp += 20,
            Some(1) => player_fighter.power += 1,
            Some(2) => player_fighter.defense += 1,
            _ => player_fighter.max_hp += 20
        }

        player.heal(player.fighter.as_ref().map_or(0, |f| f.max_hp));     
    }
}

pub fn player_take_turn(dx: i32, dy: i32, game: &mut Game, objects: &mut [Object]) {
    let (target_x, target_y): (i32, i32) = (
        objects[PLAYER].get_x() + dx,
        objects[PLAYER].get_y() + dy
    );

    if let Some(i) = objects.iter().position(|o| o.fighter.is_some() && o.pos() == (target_x, target_y)) {
        if let Some((player, target)) = get_mut_two(objects, PLAYER, i) {
            player.attack(target, &mut game.messages);
        } else {
            eprintln!("attack missed due to split_at_mut() failure");
        }
    } else {
        Object::move_by(PLAYER, dx, dy, &game.map, objects);
    }
}

pub fn handle_keys(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) -> PlayerAction {
    use PlayerAction::*;
    //let key: Key = tcod.root.wait_for_keypress(true);
    let player_alive: bool = objects[PLAYER].alive;

    match (tcod.key, tcod.key.printable, player_alive) {
        (Key {code: KeyCode::Up, ..}, _, true) => {
            player_take_turn(0, -1, game, objects);
            TookTurn
        } // move up
        (Key {code: KeyCode::Down, ..}, _, true) => {
            player_take_turn(0, 1, game, objects);
            TookTurn
        } // move down
        (Key {code: KeyCode::Right, ..}, _, true) => {
            player_take_turn(1, 0, game, objects);
            TookTurn
        } // move right
        (Key {code: KeyCode::Left, ..}, _, true) => {
            player_take_turn(-1, 0, game, objects);
            TookTurn
        } // move left
        (_, 'g', true) => {
            if let Some(item_id) = objects.iter().position(|obj| obj.pos() == objects[PLAYER].pos() && obj.item.is_some()) {
                pick_item_up(item_id, game, objects);
            }

            DidntTakeTurn
        },
        (_, 'i', true) => {
            let inventory_index = inventory_menu(
                &game.inventory,
                "Press key next to an item to use, any other key to cancel\n",
                &mut tcod.root
            );

            if let Some(inventory_index) = inventory_index {
                use_item(inventory_index, tcod, game, objects);
            }
            DidntTakeTurn
        },
        (_, 'd', true) => {
            let inventory_index = inventory_menu(
                &game.inventory,
                "Press key next to an item to drop it, any other key to cancel\n",
                &mut tcod.root
            );

            if let Some(inventory_index) = inventory_index {
                drop_item(inventory_index, game, objects);
            }
            DidntTakeTurn
        },
        (_, 'p', true) => {
            objects[PLAYER].heal(999);
            
            DidntTakeTurn
        }
        (_, 'e', true) => {
            if objects.iter().any(|obj| obj.name == "stairs" && obj.pos() == objects[PLAYER].pos()) {
                let choice = menu(
                    "Descend into the next level? (you cannot return)", 
                    &["Yes", "No"], 
                    50, 
                    &mut tcod.root
                );

                match choice {
                    Some(0) => next_level(tcod, game, objects),
                    _ => {}
                }
            }
            
            DidntTakeTurn
        }
        (_, 'c', true) => {
            let player = &objects[PLAYER];
            let levelup_xp = LEVEL_UP_INITIAL + player.level * LEVEL_UP_FACTOR;
            let fighter = &player.fighter.unwrap_or_default();

            let msg = format!(
                "Character information
    
    Level: {}
    Experience: {}
    Experience to level up: {}
    
    Maximum HP: {}
    Attack: {}
    Defense: {}",
            player.level, fighter.xp, levelup_xp, fighter.max_hp, fighter.power, fighter.defense
            );

            msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
            
            DidntTakeTurn
        }
        (
            Key { // fullscreen toggle
                code: KeyCode::Control,
                alt: true,
                ..
            },
            _,
            _
        ) => {
                let full_state = tcod.root.is_fullscreen();
                tcod.root.set_fullscreen(!full_state);
                DidntTakeTurn
            },
        (Key {code: KeyCode::Escape, ..}, _, _) => Exit, // exit game

        _ => DidntTakeTurn
    }
}