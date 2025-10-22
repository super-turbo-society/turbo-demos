use super::*;
 
#[turbo::serialize]
pub struct Player {
    pub hitbox: Bounds,
    x: f32,
    y: f32,
    dx: f32, // dx and dy used for velocity
    dy: f32,
    
    pub hp: u32,
    
    pub hit_timer: u32, // used for invincibility frames and drawing
    shoot_timer: u32, // used for rate of fire
    shooting: bool, // used for shooting animation
 
    // variables used by the HUD to display information
    pub score: u32,
    notifications: Vec<String>, 
    notification_timer: usize, 
}
 
impl Player {
    pub fn new() -> Self {
        let x = ((screen().w() / 2) - 8) as f32;
        let y = (screen().h() - 64) as f32;
        Player {
            // Initialize all fields with default values
            hitbox: Bounds::new(x, y, 16, 16),
            x,
            y,
            dx: 0.0,
            dy: 0.0,
            hp: 3,
            
            hit_timer: 0,
            shoot_timer: 0,
            shooting: false,
            
            score: 0,
            notifications: vec![ 
                "Use arrow keys to move.".to_string(), 
                "Press SPACE or A to shoot.".to_string(), 
                "Defeat enemies and collect powerups.".to_string(), 
                "Try to not die. Good luck!".to_string(), 
            ], 
            notification_timer: 0, 
        }
    }
 
    pub fn update(&mut self, projectiles: &mut Vec<Projectile>) { 
        if self.hp > 0 {
            // Player movement
            let deceleration = 0.9; // Adjust this value to control deceleration speed
            self.dx *= deceleration; // Reduce xy delta by deceleration factor
            self.dy *= deceleration;
    
            // Record keyboard input
            let mut x_input = 0.0;
            let mut y_input = 0.0;
            if gamepad::get(0).up.pressed() {
                y_input = -1.0;
            }
            if gamepad::get(0).down.pressed() {
                y_input = 1.0;
            }
            if gamepad::get(0).left.pressed() {
                x_input = -1.0;
            }
            if gamepad::get(0).right.pressed() {
                x_input = 1.0;
            }
    
            // Apply input to dx and dy, normalizing diagonal movement
            let magnitude = ((x_input * x_input + y_input * y_input) as f32).sqrt();
            if x_input != 0.0 {
                self.dx = x_input / magnitude;
            }
            if y_input != 0.0 {
                self.dy = y_input / magnitude;
            }
    
            let speed = 2.0;
            self.x = (self.x + self.dx * speed) // Translate position by input delta multiplied by speed
                .clamp(0.0, (screen().w() - self.hitbox.w() - 2) as f32); // Clamp to screen bounds
            self.y = (self.y + self.dy * speed)
                .clamp(0.0, (screen().h() - self.hitbox.h() - 2) as f32);
    
            // Set hitbox position based on float xy values
            self.hitbox = self.hitbox.position(self.x, self.y);

            // Shooting projectiles 
            // check if shoot button is pressed 
            if gamepad::get(0).start.pressed() || gamepad::get(0).a.pressed() { 
                self.shooting = true; // flag shooting state for animation 
                // if shoot timer is 0, shoot a projectile 
                if self.shoot_timer == 0 { 
                    let fire_rate = 15;
                    self.shoot_timer += fire_rate; // add cooldown to shoot timer
                    let projectile_speed = 5.0;
                    for i in 0..=1 {
                        projectiles.push(
                            Projectile::new(
                                self.x + i as f32 * 13.0,
                                self.y - 8.0,
                                projectile_speed, 
                                -90.0, 
                                ProjectileOwner::Player, 
                            ) 
                        ); 
                    } 
                } 
            // if not shooting 
            } else { 
                self.shooting = false; // flag shooting state for animation 
            } 
            // decrement shoot timer 
            self.shoot_timer = self.shoot_timer.saturating_sub(1); 
        }

        self.hit_timer = self.hit_timer.saturating_sub(1);
        // Remove the camera shake
        if self.hit_timer == 0 {
            camera::remove_shake();
        }

        // Notifications timer 
        if self.notifications.len() > 0 { 
            self.notification_timer += 1; 
            // Remove current notification if timer expires 
            if self.notification_timer >= 120 - 1 { 
                self.notification_timer = 0; 
                let _ = self.notifications.remove(0); 
            } 
        } 
    }
 
    pub fn draw(&self) {
        if self.hp > 0 {
            // Get reference to SpriteAnimation for player
            let anim = animation::get("p_key");
            // Begin to construct the string for which sprite to use
            let mut sprite = "player".to_string();
            if self.shooting { 
                sprite.push_str("_shooting"); 
            } 
            // Assign the sprite string to the SpriteAnimation
            anim.use_sprite(&sprite);
            sprite!(
                animation_key = "p_key",
                x = self.x,
                y = self.y,
            );
        }
    }

    pub fn take_damage(&mut self, damage: u32) {
        self.hp = self.hp.saturating_sub(damage); // reduce HP by damage amount
        camera::shake(5.0); // camera shake
        self.hit_timer = 20; // invincibility frame timer and drawing flag
    }

    pub fn draw_hud(&self) {
        let hud_height = 16; // Height of the HUD panel
        let hud_padding = 4; // Padding inside the HUD
    
        // Background rectangle
        rect!(
            x = -1,
            y = -1,
            w = screen().w() + 2,
            h = hud_height + 2,
            border_size = 1,
            color = 0x000000ff,
            border_color = 0xffffffff,
        ); 
        
        // Display Health
        let health_text = format!("HP: {}", self.hp);
        text!(
            &health_text,
            x = hud_padding,
            y = hud_padding,
            font = "large",
            color = 0xffffffff
        );
    
        // Display Score
        let score_text = format!("SCORE: {:0>5}", self.score);
        let score_text_x = // anchor to the right side of the screen
            screen().w() as i32 - (score_text.chars().count() as i32 * 8) - hud_padding;
        text!(
            &score_text,
            x = score_text_x,
            y = hud_padding,
            font = "large",
            color = 0xffffffff
        );
    
        // Draw notifications
        for notif in self.notifications.iter() {
            // center the text based on width of characters
            let len = notif.chars().count();
            let w = len * 5;
            let x = (screen().w() as usize / 2) - (w / 2);
            rect!(
                x = x as i32 - 4,
                y = 24 - 2,
                w = w as u32 + 4,
                h = 12,
                color = 0x5fcde4ff
            );
            text!(
                &notif,
                x = x as i32,
                y = 24,
                font = "medium",
                color = 0xffffffff
            );
            break;
        }
    }
}