use crate::{MAX_ROOMS, player::PLAYER, ROOM_MAX_SIZE, ROOM_MIN_SIZE, inventory::generate_items, monsters::generate_monsters, object::Object};
use std::cmp;
use rand::prelude::*;
use serde::{Serialize, Deserialize};

// note that MAP_WIDTH and MAP_HEIGHT cannot bypass crate::SCREEN_WIDTH and crate::SCREEN_HEIGHT
pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;
pub const MAX_MONSTER_PER_ROOM: i32 = 3;
pub const MAX_ITEMS_PER_ROOM: i32 = 2;

pub type Map = Vec<Tile>;

// all map creation logic integrated here, handles only 1 screen (map)
pub fn make_map(objects: &mut Vec<Object>) -> Map {
    // fill map with blocked tiles
    let mut map = vec![Tile::wall(); (MAP_WIDTH*MAP_HEIGHT) as usize];

    // create rooms where player is free to move in
    let mut rooms = Vec::new();

    // create rooms
    for _ in 0..MAX_ROOMS+1 {
        let w = rand::rng().random_range(ROOM_MIN_SIZE..ROOM_MAX_SIZE + 1);
        let h = rand::rng().random_range(ROOM_MIN_SIZE..ROOM_MAX_SIZE + 1);
        let x = rand::rng().random_range(0..MAP_WIDTH - w);
        let y = rand::rng().random_range(0..MAP_HEIGHT - h);

        let new_room = Room::new(x, y, w, h);

        if !rooms.iter().any(|other_room| new_room.intersects(other_room)) {
            rooms.push(new_room);
        }
    }

    // carve tunnels
    for (prev_room, new_room) in rooms.iter().skip(1).zip(rooms.iter()) {
        let (new_x, new_y) = new_room.center();
        let (prev_x, prev_y) = prev_room.center();

        if rand::random() {
            create_h_tunnel(prev_x, new_x, prev_y, &mut map);
            create_v_tunnel(prev_y, new_y, new_x, &mut map);
        } else {
            create_v_tunnel(prev_y, new_y, prev_x, &mut map);
            create_h_tunnel(prev_x, new_x, new_y, &mut map);
        }
    }

    let (player_x, player_y) = match rooms.first() {
        Some(room) => room.center(),
        None => (0, 0) // a room is a guaranteed to exist, this isn't much of a recovery solution
                       // otherwise, maybe panic!() here?
    };

    // spawn player in the first room generated
    objects[PLAYER].set_pos(player_x, player_y);

    rooms.iter().for_each(|room| {
        create_room(*room, &mut map);
        generate_monsters(*room, &map, objects);
        generate_items(*room, &map, objects);
    });

    let (stairs_x, stairs_y): (i32, i32) = rooms.last()
        .expect("No final room exists. make_map() should guarantee at least one room")
        .center();

    objects.push(
        Object::new(stairs_x, stairs_y, '<', tcod::colors::WHITE, "stairs", false)
    );

    map
}

pub fn create_room(room: Room, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[Tile::pos_to_id(x, y)] = Tile::empty();
        }
    }
}

// horizontal tunnel
pub fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[Tile::pos_to_id(x,y)] = Tile::empty();
    }
}

// vertical tunnel
pub fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[Tile::pos_to_id(x, y)] = Tile::empty();
    }
}

pub fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    map[Tile::pos_to_id(x, y)].is_blocked() || objects.iter().any(|object| object.blocks && object.pos() == (x,y))
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Tile {
    blocked: bool,
    block_sight: bool,
    explored: bool
}

#[derive(Clone, Copy, Debug)]
pub struct Room { // more of a Rectangle
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32
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

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    pub fn intersects(&self, target: &Room) -> bool {
        (self.x1 <= target.x2)
            && (self.x2 >= target.x1)
            && (self.y1 <= target.y2)
            && (self.y2 >= target.y1)
    }
}

impl Tile {
    pub fn is_blocked(&self) -> bool {
        self.blocked
    }

    pub fn is_sight_blocked(&self) -> bool {
        self.block_sight
    }

    pub fn is_explored(&self) -> bool {
        self.explored
    }

    pub fn explore(&mut self) {
        self.explored = true;
    }

    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false
        }
    }

    // so accessing (x,y) doesn't get confusing with a one-vector table implementation for tiles (line above)
    pub fn pos_to_id(x: i32, y: i32) -> usize {
        (y * MAP_WIDTH + x) as usize
    }

    pub fn id_to_pos(i: i32) -> (i32, i32) {
        (i as i32 % MAP_WIDTH, i as i32 / MAP_WIDTH)
    }
}
