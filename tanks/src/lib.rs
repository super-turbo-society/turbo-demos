turbo::init! {
    struct GameState {
        winner: Option<enum Winner {
            P1,
            P2,
            Draw,
        }>,
        tanks: Vec<struct Tank {
            color: u32,
            x: f32,
            y: f32,
            vel: f32,
            rot: f32,
            missiles: Vec<struct Missile {
                x: f32,
                y: f32,
                vel: f32,
                rot: f32,
            }>
        }>,
        blocks: Vec<struct Block {
            x: f32,
            y: f32,
            width: u32,
            height: u32,
        }>
    } = {
        let wh = resolution();
        let w = wh[0] as f32;
        let h = wh[1] as f32;
                Self {
            winner: None,
            tanks: vec![
                Tank {
                    color: 0xffff00ff,
                    x: 32.,
                    y: (h/2.),
                    vel: 0.,
                    rot: 0.,
                    missiles: vec![]
                },
                Tank {
                    color: 0xff00ffff,
                    x: w - 32.,
                    y: (h/2.),
                    vel: 0.,
                    rot: std::f32::consts::PI, // 180deg in radians
                    missiles: vec![]
                }
            ],
            blocks: create_mirrored_blocks(&[
                (32.0, 0.0, 16, 16),
                (128.0, 0.0, 8, 32),
                (72.0, 40.0, 16, 64),
                (128.0, 112.0, 8, 32),
                (32.0, 128.0, 16, 16),
            ])
        }
    }
}

turbo::go! {
    // Load game state
    let mut state = GameState::load();
    let mut tanks = state.tanks.iter_mut();
    let mut tank1 = tanks.next().unwrap();
    let mut tank2 = tanks.next().unwrap();

    // Draw stuff
    canvas::rect!(w = 256,h = 144, color = 0x222222ff);
    draw_blocks(&state.blocks);
    draw_tank(&tank1);
    draw_tank(&tank2);

    // Update tank positions, rotations, and firing
    let gp1 = gamepad(0);
    let gp2 = gamepad(1);

    if let Some(winner) = &state.winner {
        // Show winner message
        canvas::text!("WINNER {:#?}", winner; font = "large");
    } else {
        // Update tanks and check for missile collisions
        update_tank(&gp1, &mut tank1, &state.blocks);
        update_tank(&gp2, &mut tank2, &state.blocks);
        let tank1_got_hit = did_hit_missile(tank1, &tank2.missiles);
        let tank2_got_hit = did_hit_missile(tank2, &tank1.missiles);
        state.winner = match (tank1_got_hit, tank2_got_hit) {
            (false, false) => None,
            (true, true) => Some(Winner::Draw),
            (true, false) => Some(Winner::P2),
            (false, true) => Some(Winner::P1),
        }
    };

    // Save the game state
    state.save();
}

fn did_hit_missile(tank: &Tank, missiles: &[Missile]) -> bool {
    let tank_hitbox = tank.hitbox();
    for missile in missiles {
        let missile_hitbox = missile.hitbox();
        if tank_hitbox.intersects(&missile_hitbox) {
            return true;
        }
    }
    return false;
}

fn update_tank(gp: &Gamepad<Button>, tank: &mut Tank, blocks: &[Block]) {
    // Tank movement
    if gp.up.pressed() {
        tank.vel += 0.02; // Increase velocity (forward)
    }
    if gp.down.pressed() {
        tank.vel -= 0.01; // Decrease velocity (reverse)
    }

    // Update tank position
    tank.x += tank.vel * tank.rot.cos();
    tank.y += tank.vel * tank.rot.sin();

    // Check block collisions
    let tank_hitbox = tank.hitbox();
    for block in blocks {
        let block_hitbox = block.hitbox();
        // Undo position update if colliding with a block
        if tank_hitbox.intersects(&block_hitbox) {
            tank.x -= tank.vel * tank.rot.cos();
            tank.y -= tank.vel * tank.rot.sin();
            break;
        }
    }

    // Decelerate tank
    tank.vel *= 0.97;

    // Tank rotation
    if gp.left.pressed() {
        tank.rot -= 0.05; // Rotate counter-clockwise
    }
    if gp.right.pressed() {
        tank.rot += 0.05; // Rotate clockwise
    }

    // Update tank's missiles
    tank.missiles.retain_mut(|missile| {
        let missile_hitbox = missile.hitbox();
        for block in blocks {
            let block_hitbox = block.hitbox();
            // Remove missiles that hit blocks
            if missile_hitbox.intersects(&block_hitbox) {
                return false;
            }
        }
        // Update missil position
        missile.x += missile.vel * missile.rot.cos();
        missile.y += missile.vel * missile.rot.sin();
        // Remove out-of-bounds missiles
        missile.x >= 0. && missile.x <= 256. && missile.y >= 0. && missile.y <= 144.
    });

    // Handle firing missiles
    if gp.a.just_pressed() {
        tank.missiles.push(Missile {
            x: tank.x,
            y: tank.y,
            vel: 5.0,
            rot: tank.rot,
        });
    }
}

fn draw_tank(tank: &Tank) {
    // Draw tank's missiles
    for missile in &tank.missiles {
        canvas::rect!(
            x = (missile.x - 3.) as i32,
            y = (missile.y - 3.) as i32,
            w = 6,
            h = 6,
            color = tank.color
        );
    }

    // Calculate tank's position on the screen
    let tank_x = tank.x as i32;
    let tank_y = tank.y as i32;

    // Draw tank body
    canvas::circ!(x = tank_x - 8, y = tank_y - 8, d = 16, color = tank.color);

    // Draw tank turret
    for i in 8..16 {
        let turret_length = i as f32;
        let turret_end_x = tank_x as f32 + turret_length * tank.rot.cos();
        let turret_end_y = tank_y as f32 + turret_length * tank.rot.sin();
        canvas::rect!(
            x = (turret_end_x - 2.) as i32,
            y = (turret_end_y - 2.) as i32,
            w = 4,
            h = 4,
            color = tank.color
        );
    }
}

fn draw_blocks(blocks: &[Block]) {
    for block in blocks {
        canvas::rect!(
            x = block.x as i32,
            y = block.y as i32,
            w = block.width,
            h = block.height,
            color = 0x777777ff
        );
    }
}

fn create_mirrored_blocks(positions: &[(f32, f32, u32, u32)]) -> Vec<Block> {
    let mut mirrored_blocks = Vec::with_capacity(positions.len() * 2);

    for &(x, y, width, height) in positions {
        let mirrored_x = 256.0 - x - width as f32;
        let block = Block {
            x,
            y,
            width,
            height,
        };
        let mirrored_block = Block {
            x: mirrored_x,
            y,
            width,
            height,
        };
        mirrored_blocks.push(block);
        mirrored_blocks.push(mirrored_block);
    }

    mirrored_blocks
}

struct Rect {
    width: u32,
    height: u32,
    x: f32,
    y: f32,
}

impl Rect {
    fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width as f32
            && self.x + self.width as f32 > other.x
            && self.y < other.y + other.height as f32
            && self.y + self.height as f32 > other.y
    }
}

impl Tank {
    fn hitbox(&self) -> Rect {
        Rect {
            x: self.x - 8.,
            y: self.y - 8.,
            width: 16,
            height: 16,
        }
    }
}

impl Missile {
    fn hitbox(&self) -> Rect {
        Rect {
            x: self.x - 3.,
            y: self.y - 3.,
            width: 6,
            height: 6,
        }
    }
}

impl Block {
    fn hitbox(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }
}
