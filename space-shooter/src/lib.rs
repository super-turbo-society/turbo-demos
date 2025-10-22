use turbo::*;
mod model;
pub use model::*;
 
#[turbo::serialize] 
#[derive(PartialEq)] 
enum Screen { 
    Menu, 
    Game, 
} 

#[turbo::game]
struct GameState {
    screen: Screen, 
    start_tick: usize, 

    player: Player, 
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>, 
}
 
impl GameState {
    fn new() -> Self {
        Self {
            screen: Screen::Menu, 
            start_tick: 0, 

            player: Player::new(),
            enemies: vec![],
            projectiles: vec![], 
        }
    }
 
    fn update(&mut self) {
        self.draw();
        // Menu
        if self.screen == Screen::Menu { 
            if gamepad::get(0).start.just_pressed() || gamepad::get(0).a.just_pressed() { 
                self.screen = Screen::Game; // transition scene
                self.start_tick = time::tick(); 
            } 
        // Game
        } else { 
            self.player.update(&mut self.projectiles);
            self.spawn_enemies(); 
            self.enemies.retain_mut(|enemy| { 
                enemy.update(&mut self.projectiles); 
                !enemy.destroyed 
            }); 
            self.projectiles.retain_mut(|projectile| {
                projectile.update(&mut self.player, &mut self.enemies);
                !projectile.destroyed
            });
            // Game Over
            if self.player.hp == 0 { 
                if gamepad::get(0).start.just_pressed() || gamepad::get(0).a.just_pressed() { 
                    *self = GameState::new(); 
                } 
            } 
        } 
    }
 
    fn draw(&self) {
        self.player.draw();
        for projectile in self.projectiles.iter() { 
            projectile.draw(); 
        } 
        for enemy in self.enemies.iter() { 
            enemy.draw(); 
        } 

        self.player.draw_hud(); 
        self.draw_screen();     
    }

    fn spawn_enemies(&mut self) { 
        let spawn_rate = 100; 
        // Spawn a new enemy if the tick is a multiple of the spawn rate and there are less than 24 enemies already spawned 
        if time::tick() % spawn_rate == 0 && self.enemies.len() < 24 { 
            // Spawn a random enemy with these probabilities 
            self.enemies.push( 
                Enemy::new(EnemyType::Turret) 
            ); 
        } 
    } 

    fn draw_screen(&self) {
        // If in menu or game over state
        if self.screen == Screen::Menu || self.player.hp == 0 {
            // Determine title string
            let title = 
                if self.screen == Screen::Menu { "SPACE SHOOTER" } 
                else { "GAME OVER" };
            // Draw title
            text!(
                &title,
                x = (screen().w() as i32 / 2) - (title.chars().count() as i32 * 4),
                y = (screen().h() as i32 / 2) - 16,
                font = "large"
            );
            // Draw prompt to start game, blinking every half second
            if time::tick() / 4 % 8 < 4 {
                text!(
                    "Press A or Start",
                    x = (screen().w() as i32 / 2) - 38,
                    y = (screen().h() as i32 / 2) + 8,
                );
            }
        }
    }
}