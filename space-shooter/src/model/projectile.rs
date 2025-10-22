use super::*;
 
// Enum to determine who fired the projectile
#[turbo::serialize]
#[derive(PartialEq)]
pub enum ProjectileOwner {
    Enemy,
    Player,
}
 
#[turbo::serialize]
pub struct Projectile {
    pub hitbox: Bounds,
    x: f32, 
    y: f32, // use f32s to track xy positions for more precise movement
    pub velocity: f32,
    angle: f32,
 
    anim_key: String, // unique, randomly generated key to be used for SpriteAnimations
 
    pub collided: bool, // Used to control the sprite and update state
    pub destroyed: bool, // Used to remove projectile from game
 
    pub damage: u32,
    pub projectile_owner: ProjectileOwner,
}
 
impl Projectile {
    pub fn new(x: f32, y: f32, velocity: f32, angle: f32, projectile_owner: ProjectileOwner) -> Self {
        let audio = match projectile_owner {
            ProjectileOwner::Enemy => "projectile_enemy",
            ProjectileOwner::Player => "projectile_player",
        };
        audio::play(audio);
        Projectile {
            // Initialize all fields with default values
            hitbox: Bounds::new(x, y, 6, 6),
            x,
            y,
            velocity,
            angle,
 
            anim_key: random::u32().to_string(),
            
            destroyed: false,
            collided: false,
            
            damage: 1,
            projectile_owner,
        }
    }
 
    pub fn update(&mut self, player: &mut Player, enemies: &mut Vec<Enemy>) { 
        // If the projectile hasn't collided, update it as normal
        if !self.collided {
            // update projectile position
            let radian_angle = self.angle.to_radians();
            self.x += self.velocity * radian_angle.cos();
            self.y += self.velocity * radian_angle.sin();
    
            // flag the projectile to be destroyed if it goes off screen
            if self.y < -(self.hitbox.h() as f32)
            && self.x < -(self.hitbox.w() as f32)
            && self.x > screen().w() as f32
            && self.y > screen().h() as f32
            {
                self.destroyed = true;
            }

            // Checking for collisions with player or enemies based on projectile owner 
            match self.projectile_owner { 
                // Check collision with player 
                ProjectileOwner::Enemy => { 
                    if self.hitbox.intersects(&player.hitbox) 
                    && player.hp > 0
                    && player.hit_timer == 0 { // player doesn't have i-frames
                        player.take_damage(self.damage);  
                        audio::play("projectile_hit");  
                        self.collided = true; 
                    } 
                } 
                // Check collision with enemies 
                ProjectileOwner::Player => { 
                    for enemy in enemies.iter_mut() { 
                        if self.hitbox.intersects(&enemy.hitbox) && !enemy.destroyed { 
                            enemy.take_damage(player, self.damage); 

                            audio::play("projectile_hit"); 
                            self.collided = true; 
                            break; // Exit loop after first collision 
                        } 
                    } 
                } 
            } 
        // if the projectile has collided, 
        } else {
            // get reference to the SpriteAnimation of the projectile
            let anim = animation::get(&self.anim_key);
            // flag projectile as destroyed when the hit animation is done
            if anim.done() {
                self.destroyed = true;
            }
        }
    
        // Set hitbox position based on float xy values
        self.hitbox = self.hitbox.position(self.x, self.y);
    }
 
    pub fn draw(&self) {
        // Get reference to SpriteAnimation for projectile
        let anim = animation::get(&self.anim_key);
        // Begin to construct the string for which sprite to use
        let owner = match self.projectile_owner {
            ProjectileOwner::Enemy => "enemy",
            ProjectileOwner::Player => "player",
        };
        if !self.collided {
            anim.use_sprite(&format!("projectile_{}", owner));
        } else {
            anim.use_sprite(&format!("projectile_{}_hit", owner));
            anim.set_repeat(0);
            anim.set_fill_forwards(true);
        }
        
        sprite!(
            animation_key = &self.anim_key,
            x = self.x,
            y = self.y
        );
    }
}