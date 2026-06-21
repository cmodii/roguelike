mod object;
mod map;
mod components;
mod monsters;
mod renderer;
mod inventory;
mod player;
mod util;

use tcod::console::*;
use tcod::colors::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

use crate::renderer::*;
use crate::monsters::ai_take_turn;
use crate::object::Object;
use crate::map::*;
use crate::components::*;
use crate::player::{PLAYER, PlayerAction, handle_keys};

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FPS: i32 = 20;

const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

const ROOM_MAX_SIZE: i32 = 12;
const ROOM_MIN_SIZE: i32 = 8;
const MAX_ROOMS: i32 = 15;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Time {
    Stasis(i32),
    Resume(i32)
}

struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
}

struct Game {
    map: Map,
    messages: Messages,
    inventory: Vec<Object>,
    time: Time
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

fn main() {
    tcod::system::set_fps(FPS);

    let root: Root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .fullscreen(false)
        .title("Rogue Test")
        .init();

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        panel: Offscreen::new(MAP_WIDTH, PANEL_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        // mouse: Default::default()
    };

    let mut player_object: Object = Object::new(-1, -1, '@', WHITE, "PLAYER", true);
    player_object.alive = true;
    player_object.fighter = Some(Fighter {
       max_hp: 30,
       hp: 30,
       defense: 0,
       power: 5,
       on_death: DeathCallback::Player
    });

    let mut game_objects = vec![player_object];

    let mut game = Game {
        map: map::make_map(&mut game_objects),
        messages: Messages::new(),
        inventory: vec![],
        time: Time::Resume(-1)
    };

    game.messages.add("Welcome player to the dungeon!", YELLOW);
    let mut previous_player_position: (i32, i32) = (-1, -1);
    
    for (i, tile) in game.map.iter().enumerate() {
        let (x, y): (i32, i32) = Tile::id_to_pos(i as i32);
        tcod.fov.set(x, y, !tile.is_sight_blocked(), !tile.is_blocked());
    }

    // main game loop
    while !tcod.root.window_closed() {
        tcod.con.clear();

        let fov_recompute: bool = previous_player_position != game_objects[PLAYER].pos();
        render(&mut tcod, &mut game, &mut game_objects, fov_recompute);

        tcod.root.flush();

        previous_player_position = game_objects[PLAYER].pos();
        process_time(&mut game);

        match handle_keys(&mut tcod, &mut game, &mut game_objects) {
            PlayerAction::TookTurn => {
                if matches!(game.time, Time::Resume(_)) {
                    for ai_id in 0..game_objects.len() {
                        if game_objects[ai_id].ai.is_some() {
                            ai_take_turn(ai_id, &tcod, &mut game, &mut game_objects);
                        }
                    }   
                }
                
            }
            PlayerAction::DidntTakeTurn => {}
            PlayerAction::Exit => break
        }
    }
}
