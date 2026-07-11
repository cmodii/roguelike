use serde::{Serialize, Deserialize};
use tcod::colors::*;
use crate::{*, player::PLAYER};

const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
const MAX_MENU_SIZE: usize = 26;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {r: 130, g: 110,b: 50,};
const COLOR_DARK_GROUND: Color = Color {r: 50,g: 50,b: 150,};
const COLOR_LIGHT_GROUND: Color = Color {r: 200, g: 180, b: 50,};

#[derive(Serialize, Deserialize)]
pub struct Messages {
    messages: Vec<(String, Color)>
}

impl Messages {
    pub fn new() -> Self {
        Self {messages: vec![]}
    }

    pub fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.messages.push((message.into(), color));
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &(String, Color)> {
        self.messages.iter()
    }

}

fn render_bar(
    panel: &mut Offscreen,
    x: i32,
    y: i32,
    total_width: i32,
    name: &str,
    value: i32,
    maximum: i32,
    bar_color: Color,
    back_color: Color
) {
    let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;
    panel.set_default_background(back_color);
    panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

    panel.set_default_background(bar_color);
    if bar_width > 0 {
        panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
    }

    panel.set_default_foreground(WHITE);
    panel.print_ex(
        x + total_width / 2,
        y,
        BackgroundFlag::None,
        TextAlignment::Center,
        &format!("{}: {}/{}", name, value, maximum),
    );
}

pub fn render(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        let player = &objects[PLAYER];
        tcod.fov.compute_fov(player.get_x(), player.get_y(), TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // render tiles
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

    // render objects
    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| tcod.fov.is_in_fov(o.get_x(), o.get_y()))
        .collect::<Vec<_>>();

    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));

    to_draw.into_iter()
           .for_each(|o| o.draw(&mut tcod.con));

    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0, 1.0
    );

    // render health bar
    tcod.panel.set_default_background(BLACK);
    tcod.panel.clear();
    let (hp, max_hp) = objects[PLAYER].fighter.map_or((0,0), |f| (f.hp, f.max_hp));
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        LIGHT_RED,
        DARK_RED
    );

    tcod.panel.print_ex(
        1, 
        3, 
        BackgroundFlag::None, 
        TextAlignment::Left, 
        format!("Level: {}", game.level)
    );

    // render messages
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.messages.iter().rev() {
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }


    let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

    let objects_under_mouse: String = objects
        .iter()
        .filter(|obj| obj.pos() == (x,y) && tcod.fov.is_in_fov(x, y))
        .map(|obj| obj.name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    tcod.panel.set_default_foreground(LIGHT_GREY);
    tcod.panel.print_ex(
        1,
        0,
        BackgroundFlag::None,
        TextAlignment::Left,
        objects_under_mouse
    );

    blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, PANEL_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );
}

pub fn menu<T: AsRef<str>>(header: &str, options: &[T], width: i32, root: &mut tcod::console::Root) -> Option<usize> {
    assert!(
        options.len() <= MAX_MENU_SIZE,
        "Menu cannot have more than {} options", MAX_MENU_SIZE
    );

    let header_height = if header.is_empty() {0} else {root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header)};
    let height = options.len() as i32 + header_height;

    let mut window = Offscreen::new(width, height);
    
    window.set_default_foreground(WHITE);
    window.print_rect_ex(
        0,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Left,
        header,
    );

    options.iter().enumerate().for_each(|(index, option_txt)| {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!("[{}] {}", menu_letter, option_txt.as_ref());

        window.print_ex(
            0,
            header_height + index as i32,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    });

    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    blit(&window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

    root.flush();
    let key = root.wait_for_keypress(true);

    if key.printable.is_ascii_alphabetic() {
        let index: usize = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < options.len() {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn msgbox(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}