mod object;
mod map;

use tcod::console::*;
use tcod::colors::*;
use tcod::input::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

use crate::object::Object;
use crate::map::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FPS: i32 = 20;

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


struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap
}

struct Game {
    map: Map,
}

fn render(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        let player = &objects[0];
        tcod.fov.compute_fov(player.get_x(), player.get_y(), TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    for (i, tile) in game.map.iter_mut().enumerate() {
        let (x, y): (i32, i32) = get_coord(i as i32);
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

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    let key: Key = tcod.root.wait_for_keypress(true);

    match key {
        Key {code: KeyCode::Up, ..} => player.move_by(0, -1, &game), // move up
        Key {code: KeyCode::Down, ..} => player.move_by(0, 1, &game), // move down
        Key {code: KeyCode::Right, ..} => player.move_by(1, 0, &game), // move right
        Key {code: KeyCode::Left, ..} => player.move_by(-1, 0, &game), // move left
        Key { // fullscreen toggle
                code: KeyCode::Enter,
                alt: true,
                ..
            } => {
                let full_state = tcod.root.is_fullscreen();
                tcod.root.set_fullscreen(!full_state);
            },
        Key {code: KeyCode::Escape, ..} => {return true;} // exit game

        _ => {}
    }

    false
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


    let mut player: Object = Object::new(25, 23, '@', WHITE);
    let mut game = Game {
        map: map::make_map(&mut player)
    };

    let mut game_objects = [
        player,
    ];

    for (i, tile) in game.map.iter().enumerate() {
        let (x, y): (i32, i32) = get_coord(i as i32);
        tcod.fov.set(x, y, !tile.is_sight_blocked(), !tile.is_blocked());
    }

    let mut previous_player_position: (i32, i32) = (-1, -1);

    // main game loop
    while !tcod.root.window_closed() {
        tcod.con.clear();
        let fov_recompute: bool = previous_player_position != game_objects[0].get_coords();

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

        previous_player_position = game_objects[0].get_coords();
        if handle_keys(&mut tcod, &game,&mut game_objects[0]) {
            break;
        }
    }
}
