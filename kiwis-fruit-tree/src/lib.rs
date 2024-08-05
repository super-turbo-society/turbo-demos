

// Define the game configuration using the turbo::cfg! macro
turbo::cfg! {r#"
    name = "Kiwi's Fruit Tree"
    version = "1.0.0"
    author = "Turbo"
    description = "Help Kiwi get his fruit back!"
    [settings]
    resolution = [384, 216]
"#}

//TODO: separate horizontal move speed and vertical move speed max
const PLAYER_MOVE_SPEED_MAX: f32 = 4.0;
const PLAYER_ACCELERATION: f32 = 1.0;
const PLAYER_DECELERATION: f32 = 0.5;
const PLAYER_JUMP_FORCE: f32 = 100.0;
const GRAVITY: f32 = 1.0;
const TILE_SIZE: i32 = 16;

turbo::init! {
    struct GameState {
        player: Player,
        tiles: Vec<Tile>,
    } = {
        let mut tiles = Vec::new();
        // Initialize tiles along the ground for 3 units and some other tiles to jump on
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

 #[derive(BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
struct Player {
    x: f32,
    y: f32,
    speed_x: f32,
    speed_y: f32,
    max_gravity: f32,
    is_falling: bool,
    is_facing_left: bool,
    is_landed: bool,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            speed_x: 0.0,
            speed_y: 0.0,
            max_gravity: 10.0,
            is_falling: false,
            is_facing_left: false,
            is_landed: false,
        }
    }
    fn handle_input(&mut self) {
        let gp = gamepad(0);
        if gp.up.just_pressed() && self.is_landed {
            self.speed_y -= PLAYER_JUMP_FORCE;
            self.is_landed = false;
        }
        if gp.left.pressed() {
            self.speed_x -= PLAYER_ACCELERATION;
            self.is_facing_left = true;
        }
        else if gp.right.pressed() {
            self.speed_x += PLAYER_ACCELERATION;
            self.is_facing_left = false;
        }
        else{
            if self.speed_x> 0.{
                self.speed_x -= PLAYER_DECELERATION
            }
            else if self.speed_x < 0.{
                self.speed_x += PLAYER_DECELERATION
            }
        }

        self.speed_x = self.speed_x.clamp(-PLAYER_MOVE_SPEED_MAX, PLAYER_MOVE_SPEED_MAX);
        self.speed_y = self.speed_y.clamp(-PLAYER_JUMP_FORCE, PLAYER_MOVE_SPEED_MAX);
        
        self.speed_y += GRAVITY;
        self.speed_y = self.speed_y.clamp(-self.max_gravity, self.max_gravity);

    }
   

    fn check_collision_tilemap(&mut self, tiles: &[Tile]) {
        // Check collision down
        if self.speed_y > 0.0 {
            if let Some(collision) = check_collision(self.x, self.y, Direction::Down, tiles) {
                self.speed_y = 0.0;
                self.y = collision.y-16.;
                self.is_landed = true;
            }
            else{
                self.is_landed = false;
            }
        }
        
        // Check collision up
        if self.speed_y < 0.0 {
            if let Some(collision) = check_collision(self.x, self.y, Direction::Up, tiles) {
                self.speed_y = 0.0;
            }
        }

        // Check collision right
        if self.speed_x > 0.0 {
            if let Some(collision) = check_collision(self.x+1., self.y, Direction::Right, tiles) {
                self.speed_x = 0.0;
                let mut check_x = collision.x - (TILE_SIZE + 1) as f32;
                while check_collision(check_x, self.y, Direction::Right, tiles).is_some() {
                    check_x -= 1.0;
                }
                self.x = check_x;
                //return;
            }
        }

        // Check collision left
        if self.speed_x < 0.0 {
            if let Some(collision) = check_collision(self.x-1., self.y, Direction::Left, tiles) {
                self.speed_x = 0.0;
                let mut check_x = collision.x + 1.0;
                while check_collision(check_x, self.y, Direction::Left, tiles).is_some() {
                    check_x += 1.0;
                }
                self.x = check_x;
                //return;
            }
        }
    }

    fn update_position(&mut self){
        self.x += self.speed_x;
        self.y += self.speed_y;
    }

    fn draw(&self) {
        sprite!("kiwi_idle", x = self.x as i32, y = self.y as i32, flip_x = self.is_facing_left, fps=fps::MEDIUM);
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

struct Collision {
    x: f32,
    y: f32,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn check_collision(player_x: f32, player_y: f32, direction: Direction, tiles: &[Tile]) -> Option<Collision> {
    let (check_x1, check_y1, check_x2, check_y2) = match direction {
        Direction::Up => (player_x+1., player_y, player_x + 15.0, player_y),
        Direction::Down => (player_x+1., player_y + 16.0, player_x + 15.0, player_y + 16.0),
        Direction::Left => (player_x-1., player_y+1.0, player_x-1., player_y + 15.0),
        Direction::Right => (player_x + 17.0, player_y+1., player_x + 17.0, player_y + 15.0),
    };

    for tile in tiles {
        if tile.contains(check_x1, check_y1) || tile.contains(check_x2, check_y2) {
            return Some(Collision { x: check_x1, y: tile.grid_y as f32 * (TILE_SIZE as f32) });
        }
    }
    None
}

fn update_camera(p_x: f32, p_y: f32){
    let x_move_point: f32 = 32.;
    let y_move_point: f32 = 32.;
    let mut cam_x = cam!().0 as f32;
    let mut cam_y = cam!().1 as f32;
    let cam_speed = PLAYER_MOVE_SPEED_MAX;
    if p_x - cam_x > x_move_point{
        cam_x+=cam_speed;
    }
    else if p_x - cam_x < - x_move_point{
        cam_x -=cam_speed;
    }
    if p_y - cam_y > y_move_point{
        cam_y+=cam_speed;
    }
    else if p_y - cam_y < - y_move_point{
        cam_y -=cam_speed;
    }
    set_cam!(x = cam_x as i32, y = cam_y as i32);
}

turbo::go! {
    let mut state = GameState::load();

    state.player.handle_input();

    state.player.check_collision_tilemap(&state.tiles);

    state.player.update_position();

    update_camera(state.player.x, state.player.y);

    clear(0xadd8e6ff);
    
    state.player.draw();
    
    for tile in &state.tiles {
        tile.draw();
    }

    state.save();
}
