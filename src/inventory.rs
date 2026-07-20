use crate::map::{Room, Map, MAX_ITEMS_PER_ROOM, is_blocked};
use crate::{player::PLAYER, game::Time, components::*};
use crate::object::{Object, ObjectBuilder};
use crate::{Tcod, Game, renderer, game::Time::*};
use crate::game::{Transition, from_dungeon_level};

use itertools::Itertools;
use rand::prelude::*;
use rand::distr::weighted::{WeightedIndex};

use tcod::console::Root;
use tcod::colors::*;

pub const INVENTORY_SIZE: usize = 25;
pub const INVENTORY_WIDTH: i32 = 50;

const HEAL_AMOUNT: i32 = 10;
const TIME_STOP_DURATION: i32 = 10;
const LIGHTNING_RANGE: i32 = 5;
const LIGHTNING_DAMAGE: i32 = 6;
const CONFUSION_RANGE: i32 = 10;
const CONFUSION_LENGTH: i32 = 5;

const ITEMS: [(&str, &[Transition]);7] = [
    ("Healing Potion", &[
        Transition {level: 3, value: 1.5},
        Transition {level: 5, value: 3.0},
        Transition {level: 7, value: 6.0}
    ]),
    ("Lightning Spell", &[
        Transition {level: 3, value: 1.5},
        Transition {level: 5, value: 3.0},
        Transition {level: 7, value: 6.0}
    ]), ("Time Amulet", &[
        Transition {level: 3, value: 1.5},
        Transition {level: 5, value: 3.0},
        Transition {level: 7, value: 6.0}
    ]), ("Confusion Spell", &[
        Transition {level: 3, value: 1.5},
        Transition {level: 5, value: 3.0},
        Transition {level: 7, value: 6.0}
    ]), ("Sword", &[
        Transition {level: 3, value: 3.0},
        Transition {level: 5, value: 4.0},
        Transition {level: 7, value: 7.0}
    ]), ("Shield", &[
        Transition {level: 3, value: 3.0},
        Transition {level: 5, value: 5.0},
        Transition {level: 7, value: 7.0}
    ]), ("Cloak", &[
        Transition {level: 3, value: 3.0},
        Transition {level: 5, value: 5.0},
        Transition {level: 7, value: 7.0}
    ])
];

enum UseResult {
    UsedUp,
    UsedAndKept,
    Cancelled,
}

pub fn get_equipped_in_slot(slot: Slot, inventory: &[Object]) -> Option<usize> {
    inventory
        .iter()
        .enumerate()
        .find(|(_, item)| {
            item.equipment
                .as_ref()
                .is_some_and(|e| e.equipped && e.slot == slot)
        })
        .map(|(id, _)| id)
}

pub fn inventory_menu(inventory: &[Object], header: &str, root: &mut Root) -> Option<usize> {
    let options: Vec<String> = if inventory.len() > 0 {
        inventory
            .iter()
            .map(|item| {
                match item.equipment {
                    Some(equipment) if equipment.equipped => format!("{} [{}]", item.name, equipment.slot),
                    _ => item.name.clone()
                }
            })
            .collect()
    } else {
        vec!["Inventory is empty".into()]
    };

    if inventory.len() > 0 {
        renderer::menu(header, &options, INVENTORY_WIDTH, root)
    } else {
        None
    }
}

pub fn generate_items(room: Room, map: &Map, objects: &mut Vec<Object>, level: u32) {
    let mut rng = rand::rng();
    let num_items = rng.random_range(0..=MAX_ITEMS_PER_ROOM);

    for _ in 0..num_items {
        let x = rng.random_range(room.x1+1..room.x2);
        let y = rng.random_range(room.y1+1..room.y2);

        if !is_blocked(x, y, map, objects) {
            let dist = WeightedIndex::new(
                ITEMS
                .iter()
                .map(|f| from_dungeon_level(f.1, level))
            ).unwrap();
            
            let item = match ITEMS[dist.sample(&mut rng)].0 {
                "Healing Potion" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('!')
                        .color(VIOLET)
                        .name("Healing Potion")
                        .item(Item::Heal)
                        .build()
                },
                "Time Amulet" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('x')
                        .color(DARKER_RED)
                        .name("Time stop amulet")
                        .item(Item::StopTime)
                        .build()
                }
                "Lightning Spell" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('#')
                        .color(LIGHT_YELLOW)
                        .name("Scroll: Lightning bolt")
                        .item(Item::Lightning)
                        .build()
                }
                "Confusion Spell" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('#')
                        .color(LIGHTER_TURQUOISE)
                        .name("Scroll: Confusion")
                        .item(Item::Confuse)
                        .build()
                }
                "Sword" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('/')
                        .color(SKY)
                        .name("Sword")
                        .item(Item::Equipment)
                        .equipment(Equipment { 
                            slot: Slot::RightHand,
                            equipped: false,
                            power_bonus: 4,
                            defense_bonus: 0,
                            max_hp_bonus: 0
                        })
                        .build()
                }
                "Shield" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('[')
                        .color(DARKER_ORANGE)
                        .name("Shield")
                        .item(Item::Equipment)
                        .equipment(Equipment { 
                            slot: Slot::LeftHand,
                            equipped: false,
                            power_bonus: 0,
                            defense_bonus: 2,
                            max_hp_bonus: 0
                        })
                        .build()
                }
                "Cloak" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('U')
                        .color(DARKER_TURQUOISE)
                        .name("Cloak")
                        .item(Item::Equipment)
                        .equipment(Equipment { 
                            slot: Slot::Body,
                            equipped: false,
                            power_bonus: 0,
                            defense_bonus: 1,
                            max_hp_bonus: 10
                        })
                        .build()
                }
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
            Heal => cast_heal,
            StopTime => stop_time,
            Lightning => cast_lightning,
            Confuse => cast_confusion,
            Equipment => toggle_equipment,
        };

        match on_use(inventory_id, tcod, game, objects) {
            UseResult::UsedUp => {
                game.messages.add(
                    format!("Used up {}", game.inventory[inventory_id].name),
                    WHITE
                );
                game.inventory.remove(inventory_id);
            },
            UseResult::UsedAndKept => {},
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
        //let slot = item.equipment.map(|e| e.slot);
        //let index = game.inventory.len();
        game.inventory.push(item);

        /* auto equipping upon pick up, the 2 commented out lines are included with this segment
        if let Some(slot) = slot {
            if get_equipped_in_slot(slot, &game.inventory).is_none() {
                game.inventory[index].equip(&mut game.messages);
            }
        }
        */
    } else {
        game.messages
            .add(format!("Cannot pick up {}: Inventory is full.", objects[item_id].name), 
            RED  
        );
    }
}

pub fn drop_item(inventory_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    if inventory_id < objects.len() {
        let mut item: Object = game.inventory.remove(inventory_id);
        item.set_pos(objects[PLAYER].get_x(), objects[PLAYER].get_y());
        
        game.messages.add(
            format!("You dropped {}", item.name), 
            YELLOW
        );
        
        if item.equipment.is_some_and(|i| i.equipped) {
            item.unequip(&mut game.messages);
        }
        
        objects.push(item);
    }
}

// Healing
fn cast_heal(_inventory_id: usize, _tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    if let Some(fighter) = objects[PLAYER].fighter {
        if fighter.hp == objects[PLAYER].max_hp(game) {
            game.messages.add("Health already full", WHITE);
            return UseResult::Cancelled;
        } else {
            game.messages.add("Your wounds start to heal", LIGHT_GREEN);
            objects[PLAYER].heal(HEAL_AMOUNT, game);
            return UseResult::UsedUp;
        }
    }

    UseResult::Cancelled
}

// Time stop
fn stop_time(_inventory_id: usize, _tcod: &mut Tcod, game: &mut Game, _objects: &mut [Object]) -> UseResult {
    match game.time {
        Resume(_) => {
            game.time = Time::Stasis(TIME_STOP_DURATION);
            game.messages.add(
                format!("Time has been stopped for {} turns", TIME_STOP_DURATION),
                LIGHTER_YELLOW
            );

            return UseResult::UsedUp;
        }
        Stasis(_) => {
            game.messages.add("Time already stopped", WHITE);
            return UseResult::Cancelled;
        }
    }
}

fn closest_monster(tcod: &mut Tcod, objects: &mut [Object], range: i32) -> Option<usize> {
    objects
        .iter()
        .enumerate()
        .filter(|(id, object)|
            *id != PLAYER
            && object.fighter.is_some()
            && object.ai.is_some()
            && tcod.fov.is_in_fov(object.get_x(), object.get_y())
            && (objects[PLAYER].distance_to(object) as i32) < range
        )
        .sorted_by(|(_,a), (_,b)| 
            b.distance_to(&objects[PLAYER])
            .partial_cmp(&a.distance_to(&objects[PLAYER]))
            .unwrap_or(std::cmp::Ordering::Equal)
        )
        .map(|(id, _)| id)
        .last()
}

// lightning bolt
fn cast_lightning(_inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    if let Some(target_id) = closest_monster(tcod, objects, LIGHTNING_RANGE) {
        game.messages.add(
            format!("Lightning bolt strikes {} dealing {} damage", objects[target_id].name, LIGHTNING_DAMAGE),
            LIGHT_BLUE
        );

        if let Some(xp) = objects[target_id].take_damage(LIGHTNING_DAMAGE) {
            objects[PLAYER].fighter.as_mut().unwrap().xp += xp;
            game.messages.add(
                format!("+{} XP", xp),
                ORANGE
            );
        }
        UseResult::UsedUp
    } else {
        UseResult::Cancelled
    }
}

fn cast_confusion(_inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> UseResult {
    if let Some(target_id) = closest_monster(tcod, objects, CONFUSION_RANGE) {
        let old_ai: Ai = objects[target_id].ai.take().unwrap_or(Ai::Basic);

        objects[target_id].ai = Some(
            Ai::Confused { 
                previous_ai: Box::new(old_ai), 
                num_turns: CONFUSION_LENGTH
            }
        );
        
        game.messages.add(
            format!("Confusion takes effect on {} for {} turns", objects[target_id].name, CONFUSION_LENGTH),
            LIGHT_BLUE
        );

        UseResult::UsedUp
    } else {

        UseResult::Cancelled
    }

}

fn toggle_equipment(inventory_id: usize, _tcod: &mut Tcod, game: &mut Game, _objects: &mut [Object]) -> UseResult {
    let equipment: Equipment = match game.inventory[inventory_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };

    if equipment.equipped {
        game.inventory[inventory_id].unequip(&mut game.messages);
    } else {
        if let Some(current) = get_equipped_in_slot(equipment.slot, &game.inventory) {
            game.inventory[current].unequip(&mut game.messages);
        }
        
        game.inventory[inventory_id].equip(&mut game.messages);
    }

    UseResult::UsedAndKept
}
