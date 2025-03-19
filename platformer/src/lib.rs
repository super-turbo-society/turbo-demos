const TILE_SIZE: i32 = 16;
const GRAVITY: f32 = 0.6;

const PLAYER_MOVE_SPEED_MAX: f32 = 2.0;
const PLAYER_ACCELERATION: f32 = 1.0;
const PLAYER_DECELERATION: f32 = 0.5;
const PLAYER_MIN_JUMP_FORCE: f32 = 3.0;
const PLAYER_MAX_JUMP_FORCE: f32 = 5.5;
const PLAYER_JUMP_POWER_DUR: i32 = 6;
const PLAYER_COYOTE_TIMER_DUR: i32 = 3;

turbo::init! {
    struct GameState {
        player: Player,
        tiles: Vec<Tile>,
    } = {
        let mut tiles = Vec::new();

        //Bottom layer of tiles
        for x in 0..24 {
            tiles.push(Tile::new(x, 12));
        }
        //Side walls
        for y in 9..=11 {
            tiles.push(Tile::new(0, y));
            tiles.push(Tile::new(23, y));
        }
        //Some tiles to jump on
        tiles.push(Tile::new(5, 10));
        tiles.push(Tile::new(11, 9));
        tiles.push(Tile::new(17, 11));

        GameState {
            player: Player::new(200., 125.),
            tiles,
        }
    }
}

turbo::go!({
    let mut state = GameState::load();
    clear(0xadd8e6ff);
    for t in &mut state.tiles {
        t.draw();
    }
    state.player.handle_input();
    state.player.check_collision_tilemap(&state.tiles);
    state.player.update_position();
    camera::focus_rect(state.player.x, state.player.y, 16, 16);
    state.player.draw();
    state.save();
});

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
    coyote_timer: i32,
    is_powering_jump: bool,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            speed_x: 0.0,
            speed_y: 0.0,
            max_gravity: 15.0,
            is_falling: false,
            is_facing_left: true,
            is_landed: false,
            coyote_timer: 0,
            is_powering_jump: false,
        }
    }
    fn handle_input(&mut self) {
        let gp = gamepad(0);
        /////JUMPING LOGIC/////
        // If the player has just pressed jump -> add min jump force
        if (gp.up.just_pressed() || gp.start.just_pressed())
            && (self.is_landed || self.coyote_timer > 0)
            && self.speed_y >= 0.
        {
            if !self.is_powering_jump {
                self.speed_y = -PLAYER_MIN_JUMP_FORCE;
                self.is_powering_jump = true;
            }
        }
        // If they continue holding jump, continue adding jump force until they reach the maximum jump force
        if self.is_powering_jump && (gp.up.pressed() || gp.start.pressed()) && self.speed_y < 0. {
            self.speed_y -=
                (PLAYER_MAX_JUMP_FORCE - PLAYER_MIN_JUMP_FORCE) / (PLAYER_JUMP_POWER_DUR as f32);
            if self.speed_y <= -PLAYER_MAX_JUMP_FORCE {
                self.is_powering_jump = false;
            }
        } else {
            self.is_powering_jump = false;
        }

        if gp.left.pressed() {
            self.speed_x -= PLAYER_ACCELERATION;
            self.is_facing_left = true;
        } else if gp.right.pressed() {
            self.speed_x += PLAYER_ACCELERATION;
            self.is_facing_left = false;
        } else {
            if self.speed_x > 0. {
                self.speed_x -= PLAYER_DECELERATION
            } else if self.speed_x < 0. {
                self.speed_x += PLAYER_DECELERATION
            }
        }

        self.speed_x = self
            .speed_x
            .clamp(-PLAYER_MOVE_SPEED_MAX, PLAYER_MOVE_SPEED_MAX);
        if !self.is_powering_jump {
            self.speed_y += GRAVITY;
        }
        self.speed_y = self.speed_y.clamp(-PLAYER_MAX_JUMP_FORCE, self.max_gravity);

        if self.coyote_timer > 0 {
            self.coyote_timer -= 1;
        }
    }

    fn check_collision_tilemap(&mut self, tiles: &[Tile]) {
        // Check collision down
        if self.speed_y > 0.0 {
            if check_collision(self.x, self.y + self.speed_y, Direction::Down, tiles) {
                self.speed_y = 0.0;
                self.is_landed = true;
            } else {
                //if collision down is false, but is_landed is true, then we have just left the ground
                //by running off the ledge
                //so we set coyote_timer to coyote_timer_dur here, to give the player a chance to jump
                //in case they ran off the ledge a moment before they pressed the jump button
                if self.is_landed {
                    self.is_landed = false;
                    //Set this to the maximum value when you are no longer colliding downwards
                    self.coyote_timer = PLAYER_COYOTE_TIMER_DUR;
                }
            }
        }

        // Check collision up
        if self.speed_y < 0.0 {
            while self.speed_y < 0.0 {
                if check_collision(self.x, self.y + self.speed_y, Direction::Up, tiles) {
                    self.speed_y += 1.0;
                } else {
                    break;
                }
            }
        }

        // Check collision right
        if self.speed_x > 0.0 {
            while self.speed_x > 0.0 {
                if check_collision(self.x + self.speed_x, self.y, Direction::Right, tiles) {
                    self.speed_x -= 1.0;
                } else {
                    break;
                }
            }
        }

        // Check collision left
        if self.speed_x < 0.0 {
            while self.speed_x < 0.0 {
                if check_collision(self.x + self.speed_x, self.y, Direction::Left, tiles) {
                    self.speed_x += 1.0;
                } else {
                    break;
                }
            }
        }
    }

    fn update_position(&mut self) {
        self.x += self.speed_x;
        self.y += self.speed_y;
    }

    fn draw(&self) {
        if self.is_landed && self.speed_x != 0. {
            sprite!(
                "kiwi_walking",
                x = self.x as i32,
                y = self.y as i32,
                flip_x = self.is_facing_left,
            );
        } else {
            sprite!(
                "kiwi_idle",
                x = self.x as i32,
                y = self.y as i32,
                flip_x = self.is_facing_left,
            );
        }
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

    //Check if a point is contained inside this tile
    fn contains(&self, point_x: f32, point_y: f32) -> bool {
        let tile_x = self.grid_x as f32 * TILE_SIZE as f32;
        let tile_y = self.grid_y as f32 * TILE_SIZE as f32;
        point_x >= tile_x
            && point_x < tile_x + TILE_SIZE as f32
            && point_y >= tile_y
            && point_y < tile_y + TILE_SIZE as f32
    }

    fn draw(&self) {
        let x = self.grid_x as i32 * TILE_SIZE;
        let y = self.grid_y as i32 * TILE_SIZE;

        sprite!("tile", x = x, y = y);
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

//check collision betwen the player and the tilemap
fn check_collision(player_x: f32, player_y: f32, direction: Direction, tiles: &[Tile]) -> bool {
    //Width and height of sprite art.
    let w: f32 = 12.;
    let h: f32 = 12.;
    //Padding between top and left for where sprite art begins
    let pad_x: f32 = 2.;
    let pad_y: f32 = 3.;
    let (check_x1, check_y1, check_x2, check_y2) = match direction {
        Direction::Up => (
            player_x + pad_x,
            player_y + pad_y,
            player_x + pad_x + w,
            player_y + pad_y,
        ),
        Direction::Down => (
            player_x + pad_x,
            player_y + pad_y + h,
            player_x + pad_x + w,
            player_y + pad_y + h,
        ),
        Direction::Left => (
            player_x + pad_x - 1.,
            player_y + pad_y,
            player_x - 1.,
            player_y + pad_y + h,
        ),
        Direction::Right => (
            player_x + pad_x + w + 1.,
            player_y + pad_y,
            player_x + pad_x + w + 1.,
            player_y + pad_y + h,
        ),
    };

    for tile in tiles {
        if tile.contains(check_x1, check_y1) || tile.contains(check_x2, check_y2) {
            return true;
        }
    }
    false
}
