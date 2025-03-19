use borsh::{BorshDeserialize, BorshSerialize};
use turbo::prelude::*;

pub const CANVAS_WIDTH: u32 = 256;
pub const CANVAS_HEIGHT: u32 = 144;
pub const DOGE_WIDTH: f32 = 16.0;
pub const DOGE_HEIGHT: f32 = 32.0;
pub const BORK_WIDTH: f32 = 8.0;
pub const BORK_HEIGHT: f32 = 8.0;
pub const ENEMY_WIDTH: f32 = 16.0;
pub const ENEMY_HEIGHT: f32 = 16.0;
pub const POWERUP_WIDTH: f32 = 16.0;
pub const POWERUP_HEIGHT: f32 = 16.0;
pub const BAT_RANGE: f32 = 10.0;

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Bork {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
}
impl Bork {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x + (DOGE_WIDTH / 2.),
            y: y - (8. - (DOGE_HEIGHT / 2.)),
            vel_x: 5.0, // Set a constant velocity for the bork
        }
    }

    // Method to update bork's position
    pub fn update(&mut self) {
        self.x += self.vel_x;
    }

    // Method to draw the bork
    pub fn draw(&self) {
        sprite!("bork", x = self.x, y = self.y);
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub hits: u32,
    pub max_hits: u32,
}
impl Enemy {
    // Constructor for an enemy
    pub fn new(vel_x: f32) -> Self {
        let max_height = CANVAS_HEIGHT as f32;
        let slots = (max_height / ENEMY_HEIGHT) as u32;
        let slot = rand() % slots;
        let y = (slot as f32) * ENEMY_HEIGHT;
        Self {
            x: 256.0,
            y: -20. + y,
            vel_x,
            hits: 0,
            max_hits: 1,
        }
    }

    // Method to update enemy's position
    pub fn update(&mut self) {
        self.x += self.vel_x;
    }

    // Method to draw the enemy
    pub fn draw(&self) {
        sprite!("enemy", x = self.x, y = self.y);
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Powerup {
    pub x: f32,
    pub y: f32,
    pub vel_y: f32,
    pub angle: f32,
    pub powerup_type: PowerupType,
}
impl Powerup {
    pub fn new(x: f32, y: f32, angle: f32, vel_y: f32, powerup_type: PowerupType) -> Self {
        Self {
            x,
            y,
            angle,
            vel_y,
            powerup_type,
        }
    }

    // Method to draw the powerup
    pub fn draw(&self) {
        match self.powerup_type {
            PowerupType::DoubleJump => {
                sprite!("double_jump", x = self.x, y = self.y);
                rect!(
                    w = POWERUP_WIDTH,
                    h = POWERUP_HEIGHT,
                    color = 0xff000fff,
                    x = self.x,
                    y = self.y
                );
            }
            PowerupType::SpeedBoost => {
                sprite!("speed_boost", x = self.x, y = self.y);
                rect!(
                    w = POWERUP_WIDTH,
                    h = POWERUP_HEIGHT,
                    color = 0xffff00ff,
                    x = self.x,
                    y = self.y
                );
            }
            PowerupType::MultiBork => {
                sprite!("multi_bork", x = self.x, y = self.y);
                rect!(
                    w = POWERUP_WIDTH,
                    h = POWERUP_HEIGHT,
                    color = 0xff00ffff,
                    x = self.x,
                    y = self.y
                );
            }
            PowerupType::Bat => {
                sprite!("coin", x = self.x, y = self.y);
                // rect!(
                //     w = POWERUP_WIDTH as u32,
                //     h = POWERUP_HEIGHT as u32,
                //     color = 0x0000ffff,
                //     x = self.x as i32,
                //     y = self.y as i32
                // );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum PowerupType {
    DoubleJump,
    SpeedBoost,
    MultiBork,
    Bat,
}
