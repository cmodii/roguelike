use tcod::colors::Color;

use crate::MAP_WIDTH;
use std::cmp;

pub const DARK_WALL_COLOR: Color = Color {r: 0, g: 0, b: 100};
pub const DARK_GROUND_COLOR: Color = Color {r: 50, g: 50, b: 150};

pub type Map = Vec<Tile>;

pub fn get_index(x: i32, y: i32) -> usize {
    (y * MAP_WIDTH + x) as usize
}

pub fn make_map(width: i32, height: i32) -> Map {
    let mut map = vec![Tile::wall(); (width*height) as usize];

    create_room(Room::new(20, 15, 10, 15), &mut map);
    create_room(Room::new(50, 15, 10, 15), &mut map);
    create_h_tunnel(25, 55, 23, &mut map);

    map
}

pub fn create_room(room: Room, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[get_index(x, y)] = Tile::empty();
        }
    }
}

pub fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[get_index(x,y)] = Tile::empty();
    }
}

pub fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[get_index(x, y)] = Tile::empty();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    blocked: bool,
    block_sight: bool
}

#[derive(Clone, Copy, Debug)]
pub struct Room {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32
}

impl Room {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h
        }
    }
}

impl Tile {
    pub fn is_blocked(&self) -> bool {
        self.blocked
    }

    pub fn is_sight_blocked(&self) -> bool {
        self.block_sight
    }

    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true
        }
    }
}
