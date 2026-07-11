use crate::game::next_level;
use crate::object::Object;
use crate::renderer::menu;
use crate::{Tcod, Game};
use crate::util::*;
use crate::inventory::{pick_item_up, drop_item, use_item, inventory_menu};

use tcod::input::{Key, KeyCode};

pub const PLAYER: usize = 0;
pub const LEVEL_UP_INITIAL: i32 = 200;
pub const LEVEL_UP_FACTOR: i32 = 150;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit
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