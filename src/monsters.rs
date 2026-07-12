use crate::game::{from_dungeon_level, Transition};
use crate::map::{Room, Map, MAX_MONSTER_PER_ROOM, is_blocked};
use crate::components::*;
use crate::object::{Object, ObjectBuilder};
use crate::util::*;
use crate::{Tcod, Game, player::PLAYER};

use rand::prelude::*;
use rand::distr::weighted::{WeightedIndex};
use tcod::colors;

// (MONSTER,PROBABILITY OF SPAWNING) key: 3.0 -> 30%
const MONSTERS: [(&str, &[Transition]);4] = [
    ("orc", &[
        Transition {level: 3, value: 1.5},
        Transition {level: 5, value: 3.0},
        Transition {level: 7, value: 6.0}
    ]), 
    ("troll", &[
        Transition {level: 3, value: 6.0},
        Transition {level: 5, value: 2.0},
        Transition {level: 7, value: 1.0}
    ]), 
    ("skaven", &[
        Transition {level: 3, value: 7.0},
        Transition {level: 5, value: 3.0},
        Transition {level: 7, value: 5.0}
    ]), 
    ("demon", &[
        Transition {level: 3, value: 0.0},
        Transition {level: 5, value: 2.0},
        Transition {level: 7, value: 7.0}
    ])
];

pub fn ai_take_turn(ai_id: usize, tcod: &Tcod, game: &mut Game, objects: &mut [Object]) {
    if let Some(ai) = objects[ai_id].ai.take() {
        objects[ai_id].ai = Some(
            match ai {
                Ai::Basic => ai_basic(ai_id, tcod, game, objects),
                Ai::Confused { 
                    previous_ai, 
                    num_turns 
                } => ai_confused(ai_id, game, objects, previous_ai, num_turns)
            }
        )
    }
}

fn ai_basic(ai_id: usize, tcod: &Tcod, game: &mut Game, objects: &mut [Object]) -> Ai {
    let (ai_x, ai_y) = objects[ai_id].pos();
    if !tcod.fov.is_in_fov(ai_x, ai_y) {return Ai::Basic;}

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

    Ai::Basic
}

fn ai_confused(
    ai_id: usize,
    game: &mut Game, 
    objects: &mut [Object], 
    previous_ai: Box<Ai>, 
    num_turns: i32
) -> Ai
{
    if num_turns >= 0 {
        Object::move_by(
            ai_id, 
            rand::random_range(-1..=2), 
            rand::random_range(-1..=2), 
            &game.map, 
            objects
        );

        Ai::Confused { 
            previous_ai: previous_ai,
            num_turns: num_turns - 1 
        }
    } else {
        *previous_ai
    }
}

pub fn generate_monsters(room: Room, map: &Map, objects: &mut Vec<Object>, level: u32) {
    let mut rng: ThreadRng = rand::rng();
    let monster_amount = rng.random_range(0..=MAX_MONSTER_PER_ROOM);

    for _ in 0..monster_amount {
        let x: i32 = rng.random_range(room.x1+1..room.x2);
        let y: i32 = rng.random_range(room.y1+1..room.y2);

        if !is_blocked(x, y, map, objects) {
            let dist = WeightedIndex::new(
                MONSTERS
                .iter()
                .map(|s| from_dungeon_level(s.1, level))
            ).unwrap();
            
            let monster: Object = match MONSTERS[dist.sample(&mut rng)].0 {
                "orc" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('O')
                        .color(colors::DESATURATED_GREEN)
                        .name("ORC")
                        .alive(true)
                        .blocks(true)
                        .fighter(Fighter {
                            max_hp: 10,
                            hp: 10,
                            defense: 2,
                            power: 4,
                            xp: 100,
                            on_death: DeathCallback::Monster
                        })
                        .ai(Ai::Basic)
                        .build()
                },
                "troll" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('T')
                        .color(colors::DARKER_GREEN)
                        .name("TROLL")
                        .alive(true)
                        .blocks(true)
                        .fighter(Fighter {
                            max_hp: 5,
                            hp: 5,
                            defense: 1,
                            power: 2,
                            xp: 75,
                            on_death: DeathCallback::Monster
                        })
                        .ai(Ai::Basic)
                        .build()
                },
                "skaven" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('S')
                        .color(colors::COPPER)
                        .name("SKAVEN")
                        .alive(true)
                        .blocks(true)
                        .fighter(Fighter {
                            max_hp: 7,
                            hp: 7,
                            defense: 2,
                            power: 3,
                            xp: 50,
                            on_death: DeathCallback::Monster
                        })
                        .ai(Ai::Basic)
                        .build()
                },
                "demon" => {
                    ObjectBuilder::new()
                        .pos(x, y)
                        .skin('D')
                        .color(colors::DARK_CRIMSON)
                        .name("DEMON")
                        .alive(true)
                        .blocks(true)
                        .fighter(Fighter {
                            max_hp: 15,
                            hp: 15,
                            defense: 2,
                            power: 5,
                            xp: 250,
                            on_death: DeathCallback::Monster
                        })
                        .ai(Ai::Basic)
                        .build()
                },
                _ => unreachable!()
            };

            objects.push(monster);
        }
    }
}
