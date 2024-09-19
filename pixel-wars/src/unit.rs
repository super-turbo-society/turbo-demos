use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Unit {
    pub unit_type: String,
    pub data: UnitData,
    pub team: i32,
    pub health: f32,
    pub pos: (f32, f32),
    pub state: UnitState,
    pub move_tween_x: Tween<f32>,
    pub move_tween_y: Tween<f32>,
    pub target_pos: (f32, f32),
    pub attack_timer: i32,
    pub animator: Animator,
    pub damage_effect_timer: u32,
    pub blood_splatter: Option<AnimatedSprite>,
    pub is_facing_left: bool,
    //foot print status
    pub footprints: Vec<Footprint>,
    pub footprint_status: FootprintStatus,
    pub footprint_timer: i32,
}

impl Unit {
    pub fn new(unit_type: String, pos: (f32, f32), team: i32, store: &UnitDataStore) -> Self {
        // Initialize default values
        let data = store.get_unit_data(&unit_type).unwrap_or_else(|| {
            panic!("Unit type not found in the data store");
        });
        Self {
            data: data.clone(),
            unit_type,
            team,

            health: data.max_health,
            pos,
            state: UnitState::Idle,
            move_tween_x: Tween::new(0.),
            move_tween_y: Tween::new(0.),
            attack_timer: 0,
            damage_effect_timer: 0,
            blood_splatter: None,
            footprints: Vec::new(),
            footprint_status: FootprintStatus::Clean,
            footprint_timer: 20,
            is_facing_left: false,
            target_pos: (0.,0.),
            //placeholder, gets overwritten when they are drawn, but I can't figure out how to do it more logically than this
            animator: Animator::new(Animation {
                name: "placeholder".to_string(),
                s_w: data.sprite_width,
                num_frames: 4,
                loops_per_frame: UNIT_ANIM_SPEED,
                is_looping: true,
            }),
        }
    }
    pub fn update(&mut self) {
        if self.state == UnitState::Moving {
            //move toward taget pos at some speed
            //check if you
            self.pos = self.move_towards_target();
            if self.reached_target(){
                self.state = UnitState::Idle;
            } 
            // self.pos.0 = self.move_tween_x.get();
            // self.pos.1 = self.move_tween_y.get();
            // if self.move_tween_x.done() {
            //     self.state = UnitState::Idle;
            // }
        }
        if self.state == UnitState::Attacking {
            self.attack_timer -= 1;
            if self.attack_timer <= 0 {
                self.state = UnitState::Idle;
            }
        }
        if self.health <= 0. {
            self.state = UnitState::Dead;

        }

        if self.state != UnitState::Dead{
            if self.footprint_status != FootprintStatus::Clean{
                self.footprint_timer -= 1;
                if self.footprint_timer == 0{
                    self.create_footprint();
                    self.footprint_timer = 20;
                }
            }
        }
    }

    pub fn draw(&mut self) {
        let mut new_anim = Animation {
            name: self.unit_type.to_lowercase(),
            s_w: self.data.sprite_width,
            num_frames: 4,
            loops_per_frame: UNIT_ANIM_SPEED,
            is_looping: true,
        };
        if self.state == UnitState::Moving {
            new_anim.name += "_walk";
            self.animator.set_cur_anim(new_anim);
        } else if self.state == UnitState::Dead {
            new_anim.name += "_death";
            new_anim.is_looping = false;
            self.animator.set_cur_anim(new_anim);
            self.animator.next_anim = None;
        } else if self.state == UnitState::Attacking {
            //only set this once, when the attack starts.
            //That way when attack ends, they will idle (could change to reload or something later)
            if self.attack_timer == self.data.attack_time - 1 {
                new_anim.name += "_attack";
                new_anim.is_looping = false;
                self.animator.set_cur_anim(new_anim);
                let next_anim = Animation {
                    name: self.unit_type.to_lowercase() + "_idle",
                    s_w: self.data.sprite_width,
                    num_frames: 4,
                    loops_per_frame: UNIT_ANIM_SPEED,
                    is_looping: true,
                };
                self.animator.set_next_anim(Some(next_anim));
            }
        } else if self.state == UnitState::Idle {
            self.animator.cur_anim.is_looping = false;
            let next_anim = Animation {
                name: self.unit_type.to_lowercase() + "_idle",
                s_w: self.data.sprite_width,
                num_frames: 4,
                loops_per_frame: UNIT_ANIM_SPEED,
                is_looping: true,
            };
            self.animator.set_next_anim(Some(next_anim));
        }
        else if self.state == UnitState::Cheer{
            self.animator.cur_anim.is_looping = false;
            let next_anim = Animation {
                name: self.unit_type.to_lowercase() + "_cheer",
                s_w: self.data.sprite_width,
                num_frames: 4,
                loops_per_frame: UNIT_ANIM_SPEED,
                is_looping: true,
            };
            self.animator.set_next_anim(Some(next_anim));
        }
        if self.damage_effect_timer > 0 {
            self.animator.change_tint_color(DAMAGE_TINT_COLOR);
            self.damage_effect_timer -= 1;
        } else {
            self.animator.change_tint_color(COLOR_WHITE);
        }
        self.animator.update();
        self.animator.draw(self.draw_position(), self.flip_x());
        if let Some(ref mut splatter) = self.blood_splatter {
            splatter.update();
            if splatter.animator.is_done() {
                self.blood_splatter = None;
            } else {
                splatter.draw();
            }
        }
        //circ!(x=self.foot_position().0, y=self.foot_position().1, d=2,);
        //TESTING FOR center position
        // circ!(x=self.pos.0, y=self.pos.1, d = 2, color = 0x000000ff);
        // sprite!("blood_16px_01", x=self.pos.0, y=self.pos.1);

        //TURN THIS ON TO SHOW HEALTH BARS
        // if self.state == UnitState::Dead {
        //     self.draw_health_bar();
        // }
    }

    pub fn start_cheering(&mut self){
        self.state = UnitState::Cheer;
    }

    pub fn draw_health_bar(&self) {
        let d_p = self.draw_position();
        let x = d_p.0;
        let y = d_p.1;
        let x_bar = x;
        let y_bar = y - 2.;
        let w_bar = 0.06 * self.data.max_health;
        let h_bar = 2;
        let mut main_color: u32 = 0xc4f129ff;
        if self.team == 1 {
            main_color = 0xa69e9aff;
        }
        let back_color: u32 = 0xb9451dff;
        let mut health_width =
            (self.health as f32 / self.data.max_health as f32 * w_bar as f32) as i32;
        health_width = health_width.max(0);

        // Draw health bar background
        rect!(
            w = w_bar,
            h = h_bar,
            x = x_bar,
            y = y_bar,
            color = back_color
        );

        // Draw current health bar
        rect!(
            w = health_width,
            h = h_bar,
            x = x_bar,
            y = y_bar,
            color = main_color
        );

        // // Draw health bar border
        // rect!(
        //     w = w_bar + 2.,
        //     h = h_bar,
        //     x = x_bar - 1.,
        //     y = y_bar,
        //     color = 0,
        //     border_color = border_color,
        //     border_width = 1,
        //     border_radius = 2
        // )
    }

    pub fn new_target_tween_position(&mut self, target: &(f32, f32), rng: &mut RNG) {
       
        let mut dir_x = target.0 - self.pos.0;
        let dir_y = target.1 - self.pos.1;

        if dir_x > 0.{
            self.is_facing_left = false;
        }
        else if dir_x < 0.{
            self.is_facing_left = true;
        }
        //if you are already in range on the x access, only move on the y access
        //This looks better, especially for ranged units
        if dir_x.abs() < self.data.range{
            dir_x = 0.;
        }
        // Calculate the length (magnitude) of the direction vector
        let length = (dir_x * dir_x + dir_y * dir_y).sqrt();

        // Normalize the direction vector
        let norm_dir_x = dir_x / length;
        let norm_dir_y = dir_y / length;

        let rand_x = rng.next_in_range(0, 5) as f32 * norm_dir_x.signum();
        //turbo::println!("rand_x: {}", rand_x);

        let rand_y = rng.next_in_range(0, 8) as f32 * norm_dir_y.signum();
        //turbo::println!("rand_y: {}", rand_x);

        let new_x = self.pos.0 + norm_dir_x * self.data.speed + rand_x;
        let new_y = self.pos.1 + norm_dir_y * self.data.speed + rand_y;
        self.move_tween_x = Tween::new(self.pos.0).set(new_x).duration(20);
        self.move_tween_y = Tween::new(self.pos.1).set(new_y).duration(20);
        self.target_pos = (new_x,new_y);
        self.state = UnitState::Moving;
    }

    pub fn move_towards_target(&self) -> (f32, f32){
        let dir_x = self.target_pos.0- self.pos.0;
        let dir_y = self.target_pos.1 - self.pos.1;
        let length = (dir_x * dir_x + dir_y * dir_y).sqrt();
        let norm_dir_x = dir_x / length;
        let norm_dir_y = dir_y / length;
        let new_x = self.pos.0 + norm_dir_x * self.data.speed/20.;
        let new_y = self.pos.1 + norm_dir_y * self.data.speed/20.;
        (new_x,new_y)
    }

    pub fn reached_target(&self) -> bool {
        let mut reached_target = false;
        if distance_between(self.pos, self.target_pos) < self.data.speed / 20.{
            reached_target = true;
        }
        reached_target
    }
    //Not using this for now - but if we need some more control over movement we can
    pub fn new_target_position(&mut self, target:&(f32, f32), rng: &mut RNG){
        //Move toward the target xunits + some randomness
         // Calculate the direction vector from self.pos to target
         let dir_x = target.0 - self.pos.0;
         let dir_y = target.1 - self.pos.1;
 
         // Calculate the length (magnitude) of the direction vector
         let length = (dir_x * dir_x + dir_y * dir_y).sqrt();
 
         // Normalize the direction vector
         let norm_dir_x = dir_x / length;
         let norm_dir_y = dir_y / length;
 
         let rand_x = rng.next_f32() * norm_dir_x.signum() * 10.;
         //turbo::println!("rand_x: {}", rand_x);
 
         let rand_y = rng.next_f32() * norm_dir_y.signum() * 10.;
         //turbo::println!("rand_y: {}", rand_x);
        
         let new_x = self.pos.0 + norm_dir_x * (self.data.speed / 50.) + rand_x;
         let new_y = self.pos.1 + norm_dir_y * (self.data.speed / 50.) + rand_y;
         self.target_pos = (new_x, new_y);
         self.state = UnitState::Moving;

    }

    pub fn take_damage(&mut self, damage: f32){
        if self.state != UnitState::Dead{
            self.health -= damage;
            self.health = self.health.max(0.);
            self.damage_effect_timer = DAMAGE_EFFECT_TIME;
            if self.blood_splatter.is_none() {
                //make the splatter position the top-middle of the sprite
                let mut splat_pos = self.pos;
                //TODO: Figure out something better to do with these numbers, they do sort of just work for now
                if self.flip_x() {
                    splat_pos.0 -= 8.;
                } else {
                    splat_pos.0 -= 12.;
                }
                splat_pos.1 -= 12.;
                let mut new_splatter = AnimatedSprite::new(splat_pos, self.flip_x());
                let num = rand() % 8 + 1;
                let name = format!("blood_16px_0{}", num);
                new_splatter.set_anim(name, 16, 4, UNIT_ANIM_SPEED, false);
                self.blood_splatter = Some(new_splatter);
            }
        }
    }

    pub fn start_attack(&mut self, target_index: usize) -> Attack {
        self.attack_timer = self.data.attack_time;
        self.state = UnitState::Attacking;
        //create the actual attack
        let size = 1;
        let mut attack = Attack::new(
            target_index,
            2.,
            self.pos,
            self.data.damage,
            self.data.splash_area,
            size,
        );
        if self.unit_type == "bazooka" || self.unit_type == "tanker"{
            attack.is_explosive = true;
        }
        attack
    }

    pub fn create_footprint(&mut self){
        let mut color = POO_BROWN;
        match self.footprint_status {
            FootprintStatus::Clean => {
               //do nothing
            },
            FootprintStatus::Poopy => {
                color = POO_BROWN;
            },
            FootprintStatus::Acid => {
               color =  ACID_GREEN;
            },
        }
        let fp = Footprint{pos: self.foot_position(), color: color, lifetime: FOOTPRINT_LIFETIME};
        self.footprints.push(fp);
    }

    pub fn distance_to(&self, pos: &(f32, f32)) -> f32 {
        let dx = self.pos.0 - pos.0;
        let dy = self.pos.1 - pos.1;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn is_unit_in_range(&self, other: &Unit) -> bool{
        let other_pos = other.pos;
        let dx = (self.pos.0 - other_pos.0).abs();
        let dy = (self.pos.1 - other_pos.1).abs();
        if dx < self.data.range && dy < MAX_Y_ATTACK_DISTANCE{
            return true;
        }
        false
    }

    pub fn draw_position(&self) -> (f32, f32) {
        //TODO: I think this might need some work - we probably need to define an 'anchor' point
        //in the csv. I am trying to 'guess' about how far the body is from where the sprite is drawing
        //and since theres a lot of empty space on some sprites, when you flip_x you get a lot of empty space.
        let mut d_x = -8.;
        let d_y = self.data.sprite_width /2 * -1;
        // let mut d_y = -8.;
        // if self.data.sprite_width == 32{
        //     d_y += -16.;
        // }
        if self.flip_x() {
            d_x = 8. - self.data.sprite_width as f32;
        }
        return (self.pos.0 + d_x, self.pos.1 + d_y as f32);
    }

    pub fn foot_position(&self) -> (f32, f32){
        let d_y = self.data.sprite_width / 2 -1;
        return (self.pos.0, self.pos.1+d_y as f32);
    }

    pub fn flip_x(&self) -> bool {
        //self.team == 1
        self.is_facing_left
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UnitData {
    pub unit_type: String,
    pub damage: f32,
    pub max_health: f32,
    pub speed: f32,
    pub range: f32,
    pub attack_time: i32,
    pub splash_area: f32,
    pub sprite_width: i32,
    pub explode_on_death: bool,
}


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum UnitState {
    Moving,
    Attacking,
    Idle,
    Dead,
    Cheer,
}


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UnitPreview {
    //unit type as a string
    pub unit_type: String,
    //animator
    pub animator: Animator,
    pub s_w: i32,
    pub pos: (f32, f32),
    pub flip_x: bool,
    pub state: UnitState,
}

impl UnitPreview {
    pub fn new(unit_type: String, s_w: i32, pos: (f32, f32), flip_x: bool) -> Self {
        Self {
            unit_type, //placeholder, gets overwritten when they are drawn, but I can't figure out how to do it more logically than this
            animator: Animator::new(Animation {
                name: "placeholder".to_string(),
                s_w: s_w,
                num_frames: 0,
                loops_per_frame: 0,
                is_looping: true,
            }),
            s_w,
            pos,
            flip_x,
            state: UnitState::Idle,
        }
    }
    //add walk to animator, then if its done, add the other one
    pub fn update(&mut self) {
        self.animator.update();
        let mut new_anim = Animation {
            name: self.unit_type.to_lowercase(),
            s_w: self.s_w,
            num_frames: 4,
            loops_per_frame: UNIT_ANIM_SPEED,
            is_looping: false,
        };
        if self.state == UnitState::Idle {
            self.state = UnitState::Moving;
            new_anim.name += "_walk";
            self.animator.set_cur_anim(new_anim);
        } else if self.animator.is_done() {
            if self.state == UnitState::Moving {
                self.state = UnitState::Attacking;
                new_anim.name += "_attack";
                self.animator.set_cur_anim(new_anim);
            } else if self.state == UnitState::Attacking {
                self.state = UnitState::Moving;
                new_anim.name += "_walk";
                self.animator.set_cur_anim(new_anim);
            }
        }
    }

    pub fn draw(&self) {
        self.animator.draw(self.draw_position(), self.flip_x);
    }

    pub fn draw_position(&self) -> (f32, f32) {
        let mut d_y = 0.;
        let mut d_x = 0.;
        d_y = 16. - self.s_w as f32;
        if self.flip_x {
          d_x = 16. - self.s_w as f32;
        }
        (self.pos.0 + d_x, self.pos.1 + d_y)
    }
    //draw from animator
}
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Footprint{
    pub pos: (f32, f32),
    pub color: u32,
    pub lifetime: u32,
}

impl Footprint{

    pub fn draw(&mut self)
    {

        if self.lifetime != 0{
            self.lifetime -= 1;

            let opacity = if self.lifetime > FOOTPRINT_LIFETIME - 100 {
                // Fully opaque for the first 100 seconds
                255
            } else {
                // Start fading after 100 seconds
                let fade_duration = FOOTPRINT_LIFETIME - 100;
                let fade_progress = self.lifetime as f32 / fade_duration as f32;
                (fade_progress * 255.0) as u32
            };
            
            let draw_color = (self.color & 0xffffff00) | opacity;
            rect!(x = self.pos.0, y = self.pos.1, color = draw_color, w = 1, h = 1);
        }
    }
}
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum FootprintStatus{
    Clean,
    Poopy,
    Acid,
}

