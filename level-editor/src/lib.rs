// Define the game configuration
turbo::cfg! {r#"
    name = "Level Editor"
    version = "1.0.0"
    author = "Turbo"
    description = "A Turbo Demo to make a level editor for a platformer"
    [settings]
    resolution = [384, 216]
"#}

const GRAY: usize = 0x808080ff;
const LIGHT_BLUE: usize = 0xADD8E6ff;
const MAP_BOUNDS: (f32, f32, f32, f32) = (0., 2048., 0., 2048.);

turbo::init! {
    struct GameState {
       selected_tile_id: u32,
       tile_map: Vec<Tile>
    }= {
    GameState {
        selected_tile_id: 3,
        tile_map: Vec::new(),
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    //z and x to change selected tile, space to submit map
    let gp = gamepad(0);
    if gp.a.just_pressed() {
        change_selected_tile(&mut state, 1);
    } else if gp.b.just_pressed() {
        change_selected_tile(&mut state, -1);
    } else if gp.start.just_pressed() {
        submit_tile_map(&mut state);
    } else if gp.select.just_pressed() {
        load_tile_map(&mut state);
    }
    //handle camera panning
    if gp.up.pressed() {
        pan_camera(&mut state, (0, -1));
    }
    if gp.down.pressed() {
        pan_camera(&mut state, (0, 1));
    }
    if gp.right.pressed() {
        pan_camera(&mut state, (1, 0));
    }
    if gp.left.pressed() {
        pan_camera(&mut state, (-1, 0));
    }
    clear!(LIGHT_BLUE);
    draw_grid();
    let m = mouse(0);
    let [mx, my] = m.position;
    if m.left.just_pressed() {
        place_tile(&mut state, grid_pos_from_mouse_pos((mx as f32, my as f32)));
    }
    draw_tile_map(&mut state);

    draw_selected_tile(
        state.selected_tile_id,
        grid_pos_from_mouse_pos((mx as f32, my as f32)),
    );
    state.save();
});

fn submit_tile_map(gs: &mut GameState) {
    //do something with turbo OS from gs.tile_map
}

fn load_tile_map(gs: &mut GameState) {
    //do something from turbo OS to get the tile map and set it to the gs tile map
}

fn place_tile(gs: &mut GameState, grid_pos: (u32, u32)) {
    if gs.selected_tile_id == 11 {
        // Eraser mode - remove tile if it exists
        if let Some(index) = gs.tile_map.iter().position(|tile| tile.pos == grid_pos) {
            gs.tile_map.remove(index);
        }
    } else if gs.selected_tile_id == 0 || gs.selected_tile_id == 1 {
        // First remove any tile at the target position
        if let Some(index) = gs.tile_map.iter().position(|tile| tile.pos == grid_pos) {
            gs.tile_map.remove(index);
        }
        // Then remove any existing start/end point
        if let Some(index) = gs
            .tile_map
            .iter()
            .position(|tile| tile.id == gs.selected_tile_id)
        {
            gs.tile_map.remove(index);
        }
        // Add new start/end point
        gs.tile_map.push(Tile {
            id: gs.selected_tile_id,
            pos: grid_pos,
        });
    } else {
        // Normal tile placement
        if let Some(index) = gs.tile_map.iter().position(|tile| tile.pos == grid_pos) {
            // Replace existing tile
            gs.tile_map[index] = Tile {
                id: gs.selected_tile_id,
                pos: grid_pos,
            };
        } else {
            // No tile found, add new one
            gs.tile_map.push(Tile {
                id: gs.selected_tile_id,
                pos: grid_pos,
            });
        }
    }
}

fn change_selected_tile(gs: &mut GameState, dir: i32) {
    let num_tiles = 11;
    let mut id = gs.selected_tile_id as i32;
    id += dir;
    if id < 0 {
        id = num_tiles;
    } else if id > num_tiles {
        id = 0;
    }
    gs.selected_tile_id = id as u32;
}

fn pan_camera(gs: &mut GameState, dir: (i32, i32)) {
    //TODO: lock to camera bounds first
    let move_speed = 3;
    let dist = (dir.0 * move_speed, dir.1 * move_speed);
    set_cam!(x = cam!()[0] + dist.0, y = cam!()[1] + dist.1);
}

fn draw_tile_map(gs: &mut GameState) {
    for t in gs.tile_map.iter() {
        draw_selected_tile(t.id, t.pos);
    }
}

fn draw_grid() {
    //for 2048 by 2048 draw a vertical and horizontal line going up and down
    // Draw vertical lines
    for x in 0..=128 {
        rect!(w = 1, h = 2048, x = x * 16, y = 0, color = GRAY);
    }

    // Draw horizontal lines
    for y in 0..=128 {
        rect!(w = 2048, h = 1, x = 0, y = y * 16, color = GRAY);
    }
}

fn draw_selected_tile(tile_id: u32, grid_pos: (u32, u32)) {
    //get the sprite name from some lookup
    let spr_name = sprite_name_from_id(tile_id);
    //convert grid position to draw position
    let draw_pos = (grid_pos.0 * 16, grid_pos.1 * 16);
    sprite!(&spr_name, x = draw_pos.0, y = draw_pos.1);
}

fn grid_pos_from_mouse_pos(mouse_pos: (f32, f32)) -> (u32, u32) {
    let grid_pos = (
        (mouse_pos.0 / 16.0).floor() as u32,
        (mouse_pos.1 / 16.0).floor() as u32,
    );
    grid_pos
}

fn sprite_name_from_id(id: u32) -> String {
    match id {
        0 => "kiwi".to_string(),
        1 => "flag".to_string(),
        2 => "dirt_1".to_string(),
        3 => "dirt_2".to_string(),
        4 => "dirt_3".to_string(),
        5 => "grass_1".to_string(),
        6 => "grass_2".to_string(),
        7 => "grass_3".to_string(),
        8 => "stone_1".to_string(),
        9 => "stone_2".to_string(),
        10 => "stone_3".to_string(),
        11 => "erase".to_string(),
        _ => "kiwi".to_string(), // Default case if id doesn't match any above
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
enum Screen {
    Title,
    Builder,
    Browser,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
struct Tile {
    id: u32,
    pos: (u32, u32),
}
