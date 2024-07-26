

// Define the game configuration using the turbo::cfg! macro
turbo::cfg! {r#"
    name = "Kiwi's Fruit Tree"
    version = "1.0.0"
    author = "Turbo"
    description = "Help Kiwi get his fruit back!"
    [settings]
    resolution = [384, 216]
"#}

// Define the constants for player movement
const PLAYER_MOVE_SPEED_MAX: f32 = 5.0;
const PLAYER_ACCELERATION: f32 = 2.0;
const PLAYER_DECELERATION: f32 = 1.0;
const PLAYER_JUMP_FORCE: f32 = 100.0;
const GRAVITY: f32 = 1.0;
const TILE_SIZE: i32 = 16;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
struct Player {
    x: f32,
    y: f32,
    speed_x: f32,
    speed_y: f32,
    max_gravity: f32,

}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            speed_x: 0.0,
            speed_y: 0.0,
            max_gravity: 10.0,
        }
    }


    // Handle player input
    fn handle_input(&mut self) {
        let gp = gamepad(0);
        // Accelerate the player based on input
        if gp.up.just_pressed() {
            self.speed_y -= PLAYER_JUMP_FORCE;
        }
        if gp.left.pressed() {
            self.speed_x -= PLAYER_ACCELERATION;
        }
        else if gp.right.pressed() {
            self.speed_x += PLAYER_ACCELERATION;
        }
        else{
            if self.speed_x> 0.{
                self.speed_x -= PLAYER_DECELERATION
            }
            else if self.speed_x < 0.{
                self.speed_x += PLAYER_DECELERATION
            }
        }

        // Clamp the player's speed to the maximum speed
        self.speed_x = self.speed_x.clamp(-PLAYER_MOVE_SPEED_MAX, PLAYER_MOVE_SPEED_MAX);
        self.speed_y = self.speed_y.clamp(-PLAYER_JUMP_FORCE, PLAYER_MOVE_SPEED_MAX);
        
        self.speed_y += GRAVITY;
        self.speed_y = self.speed_y.clamp(-self.max_gravity, self.max_gravity);

    }
   

    fn check_collision_tilemap(&mut self, tiles: &[Tile]) {
        // Check collision in the downward direction if speed_y is positive
        if self.speed_y > 0.0 {
            if let Some(collision) = check_collision(self.x, self.y, Direction::Down, tiles) {
                self.speed_y = 0.0;
                self.y = collision.y - TILE_SIZE as f32;
                return;
            }
        }
        
        // Check collision in the upward direction if speed_y is negative
        if self.speed_y < 0.0 {
            if let Some(collision) = check_collision(self.x, self.y, Direction::Up, tiles) {
                self.speed_y = 0.0;
                self.y = collision.y + TILE_SIZE as f32;
                return;
            }
        }

        // Check collision in the right direction if speed_x is positive
        if self.speed_x > 0.0 {
            if let Some(collision) = check_collision(self.x, self.y, Direction::Right, tiles) {
                self.speed_x = 0.0;
                self.x = collision.x - TILE_SIZE as f32;
                return;
            }
        }

        // Check collision in the left direction if speed_x is negative
        if self.speed_x < 0.0 {
            if let Some(collision) = check_collision(self.x, self.y, Direction::Left, tiles) {
                self.speed_x = 0.0;
                self.x = collision.x + TILE_SIZE as f32;
                return;
            }
        }
    }

    fn update_position(&mut self){
        // Update the player's position
        self.x += self.speed_x;
        self.y += self.speed_y;
    }

    // Draw the player character
    fn draw(&self) {
        let flipx = self.speed_x<0.;
        //let t = format!("Speed x: {}", self.speed_x);
        //log!("{}", t);
        sprite!("kiwi_idle", x = self.x as i32, y = self.y as i32, sw = 16, flip_x = flipx);
    }
}



#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct Tile {
    grid_x: usize,
    grid_y: usize,
}

impl Tile {
    fn new(grid_x: usize, grid_y: usize) -> Self {
        Self { grid_x, grid_y }
    }

    fn draw(&self) {
        let x = self.grid_x as i32 * TILE_SIZE;
        let y = self.grid_y as i32 * TILE_SIZE;
        rect!(x = x, y = y, w = TILE_SIZE, h = TILE_SIZE, color = 0x0000ffff);
    }

    fn contains(&self, point_x: f32, point_y: f32) -> bool {
        let tile_x = self.grid_x as f32 * TILE_SIZE as f32;
        let tile_y = self.grid_y as f32 * TILE_SIZE as f32;
        point_x >= tile_x && point_x < tile_x + TILE_SIZE as f32 &&
        point_y >= tile_y && point_y < tile_y + TILE_SIZE as f32
    }
}

// Define the collision struct
struct Collision {
    x: f32,
    y: f32,
}

// Define the directions
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

//TODO: We need to check all 4 corners of the player, not just the one point
fn check_collision(player_x: f32, player_y: f32, direction: Direction, tiles: &[Tile]) -> Option<Collision> {
    let (check_x, check_y) = match direction {
        Direction::Up => (player_x, player_y - 1.0),
        Direction::Down => (player_x, player_y + 17.0),
        Direction::Left => (player_x - 1.0, player_y),
        Direction::Right => (player_x + 1.0, player_y),
    };

    for tile in tiles {
        if tile.contains(check_x, check_y) {
            return Some(Collision { x: check_x, y: tile.grid_y as f32 * (TILE_SIZE as f32) });
        }
    }

    None
}
// Define the game state initialization using the turbo::init! macro
turbo::init! {
    struct GameState {
        player: Player,
        tiles: Vec<Tile>,
    } = {
        let mut tiles = Vec::new();
        // Initialize tiles along the ground for 3 units
        for i in 0..(384 / TILE_SIZE) {
            for j in 0..3 {
                tiles.push(Tile::new(i as usize, ((216 / TILE_SIZE) - 1 - j) as usize));
            }
        }
        tiles.push(Tile::new(10,7));
        tiles.push(Tile::new(11,7));
        tiles.push(Tile::new(15,7));
        tiles.push(Tile::new(15,8));
        tiles.push(Tile::new(15,9));
        GameState {
            player: Player::new(0.,0.),
            tiles,
        }
    }
}


// Implement the game loop using the turbo::go! macro
turbo::go! {
    // Load the game state
    let mut state = GameState::load();

    // Handle player input
    state.player.handle_input();

    state.player.check_collision_tilemap(&state.tiles);

    // Update position
    state.player.update_position();


    // Set the background color
    clear(0x000000ff);

    // Draw the player character
    state.player.draw();
    
    for tile in &state.tiles {
        tile.draw();
    }

    // Save game state for the next frame
    state.save();
}
