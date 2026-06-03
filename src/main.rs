mod object;
mod map;

use tcod::console::*;
use tcod::colors::*;
use tcod::input::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

use crate::object::Object;
use crate::map::*;
use crate::PlayerAction::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FPS: i32 = 20;

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
    fov: FovMap
}

struct Game {
    map: Map,
}

fn player_turn(dx: i32, dy: i32, game: &Game, objects: &mut [Object]) {
    let (target_x, target_y): (i32, i32) = (
        objects[PLAYER].get_x() + dx,
        objects[PLAYER].get_y() + dy
    );

    if let Some(i) = objects.iter().position(|o| o.pos() == (target_x, target_y)) {
        println!("Interacting with [{:?}] {}", objects[i], objects[i].name);
    } else {
        Object::move_by(PLAYER, dx, dy, game, objects);
    }
}

fn render(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        let player = &objects[PLAYER];
        tcod.fov.compute_fov(player.get_x(), player.get_y(), TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    for (i, tile) in game.map.iter_mut().enumerate() {
        let (x, y): (i32, i32) = Tile::id_to_pos(i as i32);
        let visible: bool = tcod.fov.is_in_fov(x, y);


        let color = match (visible, tile.is_sight_blocked()) {
            // not within fov
            (false, true) => COLOR_DARK_WALL,
            (false, false) => COLOR_DARK_GROUND,
            // within fov
            (true, true) => COLOR_LIGHT_WALL,
            (true, false) => COLOR_LIGHT_GROUND
        };

        if visible {
            tile.explore();
        }

        if tile.is_explored() {
            tcod.con.set_char_background(x, y, color, BackgroundFlag::Set); // render map tiles
        }
    }

    for object in objects { // render game objects within player's fov
        if tcod.fov.is_in_fov(object.get_x(), object.get_y()) {
            object.draw(&mut tcod.con);
        }
    }

}

fn handle_keys(tcod: &mut Tcod, game: &Game, objects: &mut [Object]) -> PlayerAction {
    let key: Key = tcod.root.wait_for_keypress(true);
    let player_alive: bool = objects[PLAYER].is_alive();

    match (key, key.text(), player_alive) {
        (Key {code: KeyCode::Up, ..}, _, true) => {
            player_turn(0, -1, &game, objects);
            TookTurn
        } // move up
        (Key {code: KeyCode::Down, ..}, _, true) => {
            player_turn(0, 1, &game, objects);
            TookTurn
        } // move down
        (Key {code: KeyCode::Right, ..}, _, true) => {
            player_turn(1, 0, &game, objects);
            TookTurn
        } // move right
        (Key {code: KeyCode::Left, ..}, _, true) => {
            player_turn(-1, 0, &game, objects);
            TookTurn
        } // move left
        (
            Key { // fullscreen toggle
                code: KeyCode::Enter,
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
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT)
    };

    let player: Object = Object::new(-1, -1, '@', WHITE, "PLAYER", true, true);
    let mut game_objects = vec![player];

    let mut game = Game {
        map: map::make_map(&mut game_objects)
    };


    for (i, tile) in game.map.iter().enumerate() {
        let (x, y): (i32, i32) = Tile::id_to_pos(i as i32);
        tcod.fov.set(x, y, !tile.is_sight_blocked(), !tile.is_blocked());
    }

    let mut previous_player_position: (i32, i32) = (-1, -1);

    // main game loop
    while !tcod.root.window_closed() {
        tcod.con.clear();
        let fov_recompute: bool = previous_player_position != game_objects[PLAYER].pos();

        render(&mut tcod, &mut game, &mut game_objects, fov_recompute);

        blit(
            &tcod.con,
            (0, 0),
            (MAP_WIDTH, MAP_HEIGHT),
            &mut tcod.root,
            (0, 0),
            1.0, 1.0
        );
        tcod.root.flush();

        previous_player_position = game_objects[PLAYER].pos();

        if handle_keys(&mut tcod, &game, &mut game_objects) == PlayerAction::Exit {
            break;
        }

        // AI turn
        //game_objects.iter().skip(1).filter(|o| o.is_alive()).for_each(|obj| println!("{}'s turn!", obj.name));
    }
}
