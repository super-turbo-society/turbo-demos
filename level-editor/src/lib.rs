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
    //up and down to change selected tile
    let gp = gamepad(0);
    if gp.a.just_pressed() {
        change_selected_tile(&mut state, 1);
    } else if gp.b.just_pressed() {
        change_selected_tile(&mut state, -1);
    } else if gp.start.just_pressed() {
        submit_tile_map(&mut state);
    }
    //handle camera panning

    clear!(LIGHT_BLUE);
    //draw grid
    draw_grid();
    // Get the mouse state for player 1
    let m = mouse(0);
    // Get the mouse's x and y positions
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

fn place_tile(gs: &mut GameState, grid_pos: (u32, u32)) {
    // Try to find if a tile already exists at this position
    if let Some(index) = gs.tile_map.iter().position(|tile| tile.pos == grid_pos) {
        // Replace existing tile
        gs.tile_map[index] = Tile {
            id: gs.selected_tile_id, // Assuming you have this in GameState
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

fn change_selected_tile(gs: &mut GameState, dir: i32) {
    let num_tiles = 9;
    let mut id = gs.selected_tile_id as i32;
    id += dir;
    if id < 0 {
        id = num_tiles;
    } else if id > num_tiles {
        id = 0;
    }
    gs.selected_tile_id = id as u32;
}

fn pan_camera(gs: &mut GameState) {}

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
