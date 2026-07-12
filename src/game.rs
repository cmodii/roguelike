use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};

use tcod::colors::*;
use tcod::console::Console;
use tcod::input::{self, Event};
 
use crate::player::{self, PLAYER, PlayerAction, handle_keys};
use crate::Tcod;
use crate::renderer::*;
use crate::object::{Object, ObjectBuilder};
use crate::map::{make_map, Map, Tile};
use crate::components::*;
use crate::monsters::ai_take_turn;

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub map: Map,
    pub messages: Messages,
    pub inventory: Vec<Object>,
    pub time: Time,
    pub level: u32
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Time {
    Stasis(i32),
    Resume(i32)
}

pub struct Transition {
    pub level: u32,
    pub value: f32
}

pub fn from_dungeon_level(table: &[Transition], level: u32) -> f32 {
    table
        .iter()
        .find(|transition| transition.level >= level)
        .map_or(0.0, |transition| transition.value)
}

pub fn main_menu(tcod: &mut Tcod) {
    let bg = tcod::image::Image::from_file("assets/menu_background.png")
        .ok()
        .expect("Error providing background image");

    while !tcod.root.window_closed() {
        tcod::image::blit_2x(&bg, (0,0), (-1,-1), &mut tcod.root, (0,0));

        let choices = &["Play a new game", "Continue last game", "Quit"];
        let choice = menu("", choices, 24, &mut tcod.root);

        match choice {  
            Some(0) => {
                let (mut game, mut objects) = new_game(tcod);
                play_game(tcod, &mut game, &mut objects);
            }
            Some(1) => {
                match load_game() {
                    Ok((mut game, mut objects)) => {
                        initialise_fov(tcod, &game.map);
                        play_game(tcod, &mut game, &mut objects);
                    }
                    Err(e) => {
                        println!("{}", e);
                        msgbox("\nNo sdaved game to load.\n", 24, &mut tcod.root);
                        continue;
                    }
                }
            }
            Some(2) => {
                break;
            }
            _ => {}  
        }
    }
}

pub fn new_game(tcod: &mut Tcod) -> (Game, Vec<Object>) {
    let player: Object = ObjectBuilder::new()
        .pos(-1, -1)
        .skin('@')
        .color(WHITE)
        .name("PLAYER")
        .alive(true)
        .blocks(true)
        .fighter(Fighter {
           max_hp: 30,
           hp: 30,
           defense: 0,
           power: 5,
           xp: 0,
           on_death: DeathCallback::Player
        })
        .level(1)
        .build();

    let mut objects = vec![player];

    let mut game = Game {
        map: make_map(&mut objects, 1),
        messages: Messages::new(),
        inventory: vec![],
        time: Time::Resume(-1),
        level: 1
    };

    initialise_fov(tcod, &game.map);

    game.messages.add("Welcome player to the dungeon!", YELLOW);
    (game, objects)
}

pub fn initialise_fov(tcod: &mut Tcod, map: &Map) {
    for (i, tile) in map.iter().enumerate() {
        let (x, y): (i32, i32) = Tile::id_to_pos(i as i32);
        tcod.fov.set(x, y, !tile.is_sight_blocked(), !tile.is_blocked());
    }
}

pub fn next_level(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) {
    objects.truncate(1);
    game.level += 1;
    game.map = make_map(objects, game.level);
    game.messages.add("You descend deeper into the dungeon..", DARK_YELLOW);
    
    initialise_fov(tcod, &game.map);
}

fn save_game(game: &Game, objects: &[Object]) -> Result<(), Box<dyn Error>> {
    let save_data = serde_json::to_string(&(game, objects))?;
    let mut file = File::create("gamesave")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}

fn load_game() -> Result<(Game, Vec<Object>), Box<dyn Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("gamesave")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Game, Vec<Object>)>(&json_save_state)?;
    Ok(result)
}

fn process_time(game: &mut Game) {
    use Time::*;
    match game.time {
        Stasis(mut time_left) => {
            if time_left != -1 {
                time_left = (time_left - 1).max(-1);

                game.time = if time_left <= 0 {
                    Resume(-1)
                } else {
                    Stasis(time_left)
                }  
            }
        }
        Resume(mut time_left) => {
            if time_left != -1 {
                time_left = (time_left - 1).max(-1);

                game.time = if time_left <= 0 {
                    Stasis(-1)
                } else {
                    Resume(time_left)
                }  
            }
        }
    }
}

pub fn play_game(tcod: &mut Tcod, game: &mut Game, objects: &mut Vec<Object>) {
    let mut previous_player_position: (i32, i32) = (-1, -1);

    // main game loop
    while !tcod.root.window_closed() {
        tcod.con.clear();
        
        match input::check_for_event(input::MOUSE | input::KEY) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => tcod.key = k,
            _ => tcod.key = Default::default()
        }
        
        let fov_recompute: bool = previous_player_position != objects[PLAYER].pos();
        render(tcod, game, objects, fov_recompute);

        tcod.root.flush();
        player::level_up(tcod, game, objects);

        previous_player_position = objects[PLAYER].pos();
        process_time(game);

        match handle_keys(tcod, game, objects) {
            PlayerAction::TookTurn => {
                if matches!(game.time, Time::Resume(_)) {
                    for ai_id in 0..objects.len() {
                        if objects[ai_id].ai.is_some() {
                            ai_take_turn(ai_id, &tcod, game, objects);
                        }
                    }   
                }
                
            }
            PlayerAction::DidntTakeTurn => {}
            PlayerAction::Exit => {
                save_game(game, objects).expect("Failed to save game");
                break;
            }
        }
    }
}