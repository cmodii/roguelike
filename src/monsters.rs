use crate::map::{Room, Map, MAX_MONSTER_PER_ROOM, is_blocked};
use crate::components::*;
use crate::object::Object;

use crate::{Tcod, Game, PLAYER, get_mut_two};
use rand::prelude::*;
use tcod::colors;

pub fn ai_take_turn(ai_id: usize, tcod: &Tcod, game: &mut Game, objects: &mut [Object]) {
    let (ai_x, ai_y) = objects[ai_id].pos();
    if !tcod.fov.is_in_fov(ai_x, ai_y) {return;}

    match objects[ai_id].distance_to(&objects[PLAYER]) {
        0.0..2.0 => {
            if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
                if let Some((ai, player)) = get_mut_two(objects, ai_id, PLAYER) {
                    ai.attack(player, &mut game.messages);
                } else {
                    eprintln!("attack missed due to split_at_mut() failure");
                }
            }
        },
        2.0.. => {
            let (player_x, player_y): (i32, i32) = objects[PLAYER].pos();
            Object::move_towards(ai_id, player_x, player_y, &game.map, objects);
        }
        _ => {}
    }
}

pub fn generate_monsters(room: Room, map: &Map, objects: &mut Vec<Object>) {
    let mut rng: ThreadRng = rand::rng();
    let monster_amount = rng.random_range(0..=MAX_MONSTER_PER_ROOM);

    for _ in 0..monster_amount {
        let x: i32 = rng.random_range(room.x1+1..room.x2);
        let y: i32 = rng.random_range(room.y1+1..room.y2);

        if !is_blocked(x, y, map, objects) {
            let mut monster: Object = match rng.random::<f32>() {
                0.0..0.3 => {
                    let mut orc: Object = Object::new(x, y, 'O', colors::DESATURATED_GREEN, "ORC", true);
                    orc.alive = true;
                    orc.fighter = Some(Fighter {
                        max_hp: 10,
                        hp: 10,
                        defense: 3,
                        power: 5,
                        on_death: DeathCallback::Monster
                    });
                    orc.ai = Some(Ai::Basic);

                    orc
                },
                0.3..0.6 => {
                    let mut troll: Object = Object::new(x, y, 'T', colors::DARKER_GREEN, "TROLL", true);
                    troll.alive = true;
                    troll.fighter = Some(Fighter {
                        max_hp: 5,
                        hp: 5,
                        defense: 1,
                        power: 2,
                        on_death: DeathCallback::Monster
                    });
                    troll.ai = Some(Ai::Basic);

                    troll
                },
                0.6..0.9 => {
                    let mut skaven: Object = Object::new(x, y, 'S', colors::COPPER, "SKAVEN", true);
                    skaven.alive = true;
                    skaven.fighter = Some(Fighter {
                        max_hp: 7,
                        hp: 7,
                        defense: 2,
                        power: 3,
                        on_death: DeathCallback::Monster
                    });
                    skaven.ai = Some(Ai::Basic);

                    skaven
                },
                0.9..1.0 => {
                    let mut demon: Object = Object::new(x, y, 'D', colors::DARK_CRIMSON, "DEMON", true);
                    demon.alive = true;
                    demon.fighter = Some(Fighter {
                        max_hp: 15,
                        hp: 15,
                        defense: 5,
                        power: 7,
                        on_death: DeathCallback::Monster
                    });
                    demon.ai = Some(Ai::Basic);

                    demon
                },
                _ => unreachable!()
            };

            monster.alive = true;
            objects.push(monster);
        }
    }
}
