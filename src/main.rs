mod game;
mod object;
mod map;
mod components;
mod monsters;
mod renderer;
mod inventory;
mod player;
mod util;

use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};
use tcod::input::{Key, Mouse};

use crate::game::{Game, main_menu};
use crate::object::Object;
use crate::map::*;

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

struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
    key: Key,
    mouse: Mouse
}

fn main() {
    tcod::system::set_fps(FPS);

    let root: Root = Root::initializer()
        .font("assets/arial10x10.png", FontLayout::Tcod)
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

    main_menu(&mut tcod);
}
