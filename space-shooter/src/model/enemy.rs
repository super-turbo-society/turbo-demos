use super::*;
 
// Different types of enemies
#[turbo::serialize]
pub enum EnemyType {
    Tank,
    Shooter,
    Turret,
    Zipper,
    Meteor,
}
 
#[turbo::serialize]
// Struct for Enemies
pub struct Enemy {
    enemy_type: EnemyType,
    
    pub hitbox: Bounds,
    x: f32,
    y: f32, 
 
    hit_timer: u32, // used for hit animation
    pub destroyed: bool,
 
    pub hp: u32,
    pub angle: f32,
}
 
impl Enemy {
    // Initialize different enemy types with different properties
    pub fn new(enemy_type: EnemyType) -> Self {
        let (x,y) = ((random::u32() % screen().w()).saturating_sub(32) as f32, -32.0);
        // Set initial properties based on enemy type
        Self {
            enemy_type: enemy_type,
            
            hitbox: Bounds::new(x, y, 16, 16),
            x, 
            y,
 
            hit_timer: 0,
            destroyed: false,
            
            hp: 8,
            angle: 0.0,
        }
    }
 
    pub fn update(&mut self, projectiles: &mut Vec<Projectile>) {
        // Move down
        let speed = 0.5;
        self.y += speed;
        // Random chance to fire projectile
        if random::u32() % 250 == 0 {
            // Create and shoot projectiles from enemy towards the player
            projectiles.push(Projectile::new(
                self.x + (self.hitbox.w() as f32 * 0.5) - (self.hitbox.w() as f32 * 0.5),
                self.y + (self.hitbox.h() as f32),
                2.5,
                90.0,
                ProjectileOwner::Enemy,
            ));
        }
        // Flag to destroy if moved offscreen 
        if self.y > (screen().h() + self.hitbox.h()) as f32 {
            self.destroyed = true;
        }
        // Set hitbox position based on float xy values
        self.hitbox = self.hitbox.position(self.x, self.y);

        self.hit_timer = self.hit_timer.saturating_sub(1);
    }
    
    pub fn draw(&self) {
        // Construct the string for which sprite to use
        let sprite = match self.enemy_type {
            EnemyType::Tank => "tank",
            EnemyType::Shooter => "shooter",
            EnemyType::Turret => "turret",
            EnemyType::Zipper => "zipper",
            EnemyType::Meteor => "meteor",
        };
        // Draw sprite
        sprite!(
            &sprite,
            x = self.hitbox.x(),
            y = self.hitbox.y(),
        );
    }

    pub fn take_damage(&mut self, player: &mut Player, damage: u32) {
        // reduce HP by damage amount and set hit timer for hit effect
        self.hp = self.hp.saturating_sub(damage);
        self.hit_timer = 15; // frames to show hit effect
        // Destroy enemy and increase player score if hp is 0
        if self.hp == 0 {
            self.destroyed = true;
            player.score += 20;
        }
    }
}