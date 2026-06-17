use crate::map::{Room, Map, MAX_ITEMS_PER_ROOM, is_blocked};
use crate::{components::*, PLAYER};
use crate::object::Object;

use crate::{Tcod, Game, renderer};
use rand::prelude::*;
use tcod::console::Root;
use tcod::colors::*;

pub const INVENTORY_SIZE: usize = 25;
pub const INVENTORY_WIDTH: i32 = 50;

const HEAL_AMOUNT: i32 = 10;

enum UseResult {
    UsedUp,
    Cancelled,
}

pub fn inventory_menu(inventory: &[Object], header: &str, root: &mut Root) -> Option<usize> {
    let options: Vec<String> = if inventory.len() > 0 {
        inventory.iter().map(|item| item.name.clone()).collect()
    } else {
        vec!["Inventory is empty".into()]
    };

    if inventory.len() > 0 {
        renderer::menu(header, &options, INVENTORY_WIDTH, root)
    } else {
        None
    }
}

pub fn generate_items(room: Room, map: &Map, objects: &mut Vec<Object>) {
    let mut rng = rand::rng();
    let num_items = rng.random_range(0..=MAX_ITEMS_PER_ROOM);

    for _ in 0..num_items {
        let x = rng.random_range(room.x1+1..room.x2);
        let y = rng.random_range(room.y1+1..room.y2);

        if !is_blocked(x, y, map, objects) {
            let item = match rng.random::<f64>() {
                0.0..1.0 => {
                    let mut item: Object = Object::new(x, y, '!', VIOLET, "Healing Potion", false);
                    item.item = Some(Item::Heal);
                    
                    item
                },
                _ => unreachable!()
            };

            objects.push(item);
        }
    }
}

pub fn use_item(inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    use Item::*;

    if let Some(item) = game.inventory[inventory_id].item {
        let on_use: fn(usize, &mut Tcod, &mut Game, &mut [Object]) -> UseResult = match item {
            Heal => cast_heal
        };

        match on_use(inventory_id, tcod, game, objects) {
            UseResult::UsedUp => {
                game.messages.add(
                    format!("Used up {}", game.inventory[inventory_id].name),
                    WHITE
                );
                game.inventory.remove(inventory_id);
            },
            UseResult::Cancelled => {
                game.messages.add("Cancelled", WHITE);
            }
        }
    } else {
        game.messages.add(
            format!("{} cannot be used", game.inventory[inventory_id].name),
            WHITE
        )
    }
}

pub fn pick_item_up(item_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    if game.inventory.len() < INVENTORY_SIZE {
        let item: Object = objects.swap_remove(item_id);
        game.messages
            .add(format!("Picked up {}", item.name), 
            GREEN  
        );
        game.inventory.push(item);
    } else {
        game.messages
            .add(format!("Cannot pick up {}: Inventory is full.", objects[item_id].name), 
            RED  
        );
    }
}


// Healing
fn cast_heal(_inventory_id: usize, _tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    if let Some(fighter) = objects[PLAYER].fighter {
        if fighter.hp == fighter.max_hp {
            game.messages.add("Health already full", WHITE);
            return UseResult::Cancelled;
        } else {
            game.messages.add("Your wounds start to heal", LIGHT_GREEN);
            objects[PLAYER].heal(HEAL_AMOUNT);
            return UseResult::UsedUp;
        }
    }

    UseResult::Cancelled
}