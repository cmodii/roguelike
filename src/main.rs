mod object;
mod map;

use tcod::console::*;
use tcod::colors::*;
use tcod::input::*;

use crate::object::Object;
use crate::map::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FPS: i32 = 20;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

struct Tcod {
    root: Root,
    con: Offscreen
}

struct Game {
    map: Map,
}

fn render(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    for (i, tile) in game.map.iter().enumerate() {
        let color = if tile.is_sight_blocked() {
            map::DARK_WALL_COLOR
        } else {
            map::DARK_GROUND_COLOR
        };

        tcod.con.set_char_background(i as i32 % MAP_WIDTH, i as i32 / MAP_WIDTH, color, BackgroundFlag::Set); // render map tiles
    }

    for object in objects { // render game objects
        object.draw(&mut tcod.con);
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

    let con: Offscreen = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let mut tcod = Tcod {root, con};

    let player: Object = Object::new(25, 23, '@', WHITE);
    let mut game = Game {
        map: map::make_map(MAP_WIDTH, MAP_HEIGHT)
    };

    let mut game_objects = [
        player,
        Object::new(0, 0, 'T', DARK_RED)
    ];

    while !tcod.root.window_closed() {
        tcod.con.clear();
        render(&mut tcod, &game, &mut game_objects);

        blit(
            &tcod.con,
            (0, 0),
            (MAP_WIDTH, MAP_HEIGHT),
            &mut tcod.root,
            (0, 0),
            1.0, 1.0
        );
        tcod.root.flush();

        if handle_keys(&mut tcod, &game,&mut game_objects[0]) {
            break;
        }
    }
}
