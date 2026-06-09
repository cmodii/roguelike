mod object;
mod map;
mod components;
mod monsters;
mod renderer;

use std::cmp;
use std::hash::Hash;

use tcod::console::*;
use tcod::colors::*;
use tcod::input::MouseState;
use tcod::input::{KeyCode, Key, Mouse, Event, self};
use tcod::map::{FovAlgorithm, Map as FovMap};

use crate::renderer::*;
use crate::monsters::ai_take_turn;
use crate::object::Object;
use crate::map::*;
use crate::PlayerAction::*;
use crate::components::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FPS: i32 = 20;

const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

const PLAYER: usize = 0;

const ROOM_MAX_SIZE: i32 = 12;
const ROOM_MIN_SIZE: i32 = 8;
const MAX_ROOMS: i32 = 15;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 5;
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {r: 130, g: 110,b: 50,};
const COLOR_DARK_GROUND: Color = Color {r: 50,g: 50,b: 150,};
const COLOR_LIGHT_GROUND: Color = Color {r: 200, g: 180, b: 50,};

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit
}

struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
    key: Key,
    mouse: Mouse
}

struct Game {
    map: Map,
    messages: Messages
}

fn get_mut_two<T>(vec: &mut [T], i: usize, j: usize) -> Option<(&mut T, &mut T)> {
    if i == j {return None;}
    let (first, second) = vec.split_at_mut(cmp::max(i, j));
    if i < j {
        Some((&mut first[i], &mut second[0]))
    } else {
        Some((&mut second[0], &mut first[j]))
    }
}

fn player_take_turn(dx: i32, dy: i32, game: &mut Game, objects: &mut [Object]) {
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

fn handle_keys(tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) -> PlayerAction {
    //let key: Key = tcod.root.wait_for_keypress(true);
    let player_alive: bool = objects[PLAYER].alive;

    match (tcod.key, tcod.key.text(), player_alive) {
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
        key: Default::default(),
        mouse: Default::default()
    };

    let mut player: Object = Object::new(-1, -1, '@', WHITE, "PLAYER", true);
    player.alive = true;
    player.fighter = Some(Fighter {
       max_hp: 30,
       hp: 30,
       defense: 0,
       power: 5,
       on_death: DeathCallback::Player
    });

    let mut game_objects = vec![player];

    let mut game = Game {
        map: map::make_map(&mut game_objects),
        messages: Messages::new()
    };

    for (i, tile) in game.map.iter().enumerate() {
        let (x, y): (i32, i32) = Tile::id_to_pos(i as i32);
        tcod.fov.set(x, y, !tile.is_sight_blocked(), !tile.is_blocked());
    }

    game.messages.add("Welcome player to the dungeon!", YELLOW);

    let mut previous_player_position: (i32, i32) = (-1, -1);

    // main game loop
    while !tcod.root.window_closed() {
        match input::check_for_event(input::MOUSE | input::KEY) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => tcod.key = k,
            _ => tcod.key = Default::default()
        }


        tcod.con.clear();

        let fov_recompute: bool = previous_player_position != game_objects[PLAYER].pos();
        render(&mut tcod, &mut game, &mut game_objects, fov_recompute);

        tcod.root.flush();

        previous_player_position = game_objects[PLAYER].pos();

        match handle_keys(&mut tcod, &mut game, &mut game_objects) {
            PlayerAction::TookTurn => {
                for ai_id in 0..game_objects.len() {
                    if game_objects[ai_id].ai.is_some() {
                        ai_take_turn(ai_id, &tcod, &mut game, &mut game_objects);
                    }
                }
            }
            PlayerAction::DidntTakeTurn => {}
            PlayerAction::Exit => break
        }
    }
}
