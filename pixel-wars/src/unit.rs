use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UnitDisplay {
    //visual
    pub animator: Animator,
    pub damage_effect_timer: u32,
    pub blood_splatter: Option<AnimatedSprite>,
    pub is_facing_left: bool,

    //foot print status
    pub footprints: Vec<Footprint>,
    pub footprint_status: FootprintStatus,
    pub footprint_timer: i32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Unit {
    pub unit_type: String,
    pub data: UnitData,
    pub team: i32,
    pub id: u32,
    pub target_id: u32,
    pub health: f32,
    pub pos: (f32, f32),
    pub state: UnitState,
    pub target_pos: (f32, f32),
    pub attack_strategy: AttackStrategy,
    pub attack_timer: i32,
    pub status_effects: Vec<Status>,
    pub display: Option<UnitDisplay>,
}

impl Unit {
    pub fn new(
        unit_type: String,
        pos: (f32, f32),
        team: i32,
        store: &UnitDataStore,
        id: u32,
    ) -> Self {
        // Initialize default values
        let data = store.get_unit_data(&unit_type).unwrap_or_else(|| {
            panic!("Unit type not found in the data store");
        });
        Self {
            data: data.clone(),
            unit_type,
            team,
            id,
            target_id: 0,
            health: data.max_health,
            pos,
            state: UnitState::Idle,
            target_pos: (0., 0.),
            attack_strategy: AttackStrategy::AttackClosest,
            attack_timer: 0,
            status_effects: Vec::new(),
            display: Some(UnitDisplay {
                damage_effect_timer: 0,
                blood_splatter: None,
                footprints: Vec::new(),
                footprint_status: FootprintStatus::Clean,
                footprint_timer: 20,
                is_facing_left: false,
                //placeholder, gets overwritten when they are drawn, but I can't figure out how to do it more logically than this
                animator: Animator::new(Animation {
                    name: "placeholder".to_string(),
                    s_w: data.sprite_width,
                    num_frames: 0,
                    loops_per_frame: UNIT_ANIM_SPEED,
                    is_looping: true,
                }),
            }),
        }
    }
    pub fn update(&mut self) {
        if self.state == UnitState::MarchingIn {
            //move towards target
            self.pos = self.move_towards_target();
            //if you reached the target,
            if self.reached_target() {
                self.pos = self.target_pos;
                self.state = UnitState::Idle;
            }
            //set your position to the target and switch to idle
        }
        if self.state == UnitState::Moving {
            //move toward taget pos at some speed
            self.pos = self.move_towards_target();

            if self.reached_target() {
                self.state = UnitState::Idle;
            } else {
            }
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

        if self.state != UnitState::Dead {
            if self.display.as_ref().unwrap().footprint_status != FootprintStatus::Clean {
                self.display.as_mut().unwrap().footprint_timer -= 1;
                if self.display.as_ref().unwrap().footprint_timer == 0 {
                    self.create_footprint();
                    self.display.as_mut().unwrap().footprint_timer = 20;
                }
            }
            self.apply_status_effects();
        }
    }

    pub fn draw(&mut self) {
        // Calculate positions first before any mutable borrows
        let dp = self.draw_position();
        let flip_x = self.flip_x();

        // Early return if no display
        let display = match self.display.as_mut() {
            Some(d) => d,
            None => return,
        };

        let mut new_anim = Animation {
            name: self.unit_type.to_lowercase(),
            s_w: self.data.sprite_width,
            num_frames: 4,
            loops_per_frame: UNIT_ANIM_SPEED,
            is_looping: true,
        };

        if self.state == UnitState::Moving || self.state == UnitState::MarchingIn {
            new_anim.name += "_walk";
            display.animator.set_cur_anim(new_anim);
        } else if self.state == UnitState::Dead {
            new_anim.name += "_death";
            new_anim.is_looping = false;
            display.animator.set_cur_anim(new_anim);
            display.animator.next_anim = None;
        } else if self.state == UnitState::Attacking {
            //only set this once, when the attack starts.
            //That way when attack ends, they will idle (could change to reload or something later)
            if self.attack_timer == self.data.attack_time - 1 {
                new_anim.name += "_attack";
                new_anim.is_looping = false;
                display.animator.set_cur_anim(new_anim);
                let next_anim = Animation {
                    name: self.unit_type.to_lowercase() + "_idle",
                    s_w: self.data.sprite_width,
                    num_frames: 4,
                    loops_per_frame: UNIT_ANIM_SPEED,
                    is_looping: true,
                };
                display.animator.set_next_anim(Some(next_anim));
            }
        } else if self.state == UnitState::Idle {
            display.animator.cur_anim.is_looping = false;
            let next_anim = Animation {
                name: self.unit_type.to_lowercase() + "_idle",
                s_w: self.data.sprite_width,
                num_frames: 4,
                loops_per_frame: UNIT_ANIM_SPEED,
                is_looping: true,
            };
            display.animator.set_next_anim(Some(next_anim));
            //self.animator.set_cur_anim(new_anim);
        } else if self.state == UnitState::Cheer {
            display.animator.cur_anim.is_looping = false;
            let next_anim = Animation {
                name: self.unit_type.to_lowercase() + "_cheer",
                s_w: self.data.sprite_width,
                num_frames: 4,
                loops_per_frame: UNIT_ANIM_SPEED,
                is_looping: true,
            };
            display.animator.set_next_anim(Some(next_anim));
        }

        if display.damage_effect_timer > 0 {
            display.animator.change_tint_color(DAMAGE_TINT_RED);
            display.damage_effect_timer -= 1;
        } else if self.health <= 0. {
            display.animator.change_tint_color(0xFFFFFF75);
        } else {
            display.animator.change_tint_color(WHITE);
        }

        display.animator.update();
        display.animator.draw(dp, flip_x);

        if let Some(ref mut splatter) = display.blood_splatter {
            splatter.update();
            if splatter.animator.is_done() {
                display.blood_splatter = None;
            } else {
                splatter.draw();
            }
        }

        //TESTING FOR CENTER POSITION
        //circ!(x=self.pos.0, y=self.pos.1, d = 1, color = 0x000000ff);
        // sprite!("blood_16px_01", x=self.pos.0, y=self.pos.1);
        //TESTING FOR FOOT POSITION
        // circ!(
        //     x = self.foot_position().0,
        //     y = self.foot_position().1,
        //     d = 1,
        // );

        // circ!(
        //     x = self.head_position().0,
        //     y = self.head_position().1,
        //     d = 1,
        // );

        //TURN THIS ON TO SHOW HEALTH BARS
        // if self.state == UnitState::Dead {
        //     self.draw_health_bar();
        // }

        if self.state != UnitState::Dead {
            //self.draw_strategy_icon();
            self.draw_status_effects();
        }
    }

    pub fn set_march_position(&mut self) {
        //set target pos as pos
        self.target_pos = self.pos;
        if self.pos.0 > 100. {
            self.pos.0 += 40.;
        } else {
            self.pos.0 -= 40.;
        }
        self.state = UnitState::MarchingIn;
    }

    pub fn set_starting_strategy(&mut self, rng: &mut RNG) {
        //TODO: Clean this up a bit so its more flexible
        //set some different odds and check attributes to assign strategy
        if self.data.has_attribute(&Attribute::Flanker) {
            let flank_chance = 3;
            if rng.next() % flank_chance == 0 {
                self.attack_strategy = AttackStrategy::Flank {
                    stage: (FlankStage::Vertical),
                };
            }
        } else {
            let target_chance = 6;
            if rng.next() % target_chance == 0 {
                self.attack_strategy = AttackStrategy::SeekTarget;
            }
        }
    }

    pub fn start_cheering(&mut self) {
        self.state = UnitState::Cheer;
        //turn off flee status
        self.attack_strategy = AttackStrategy::AttackClosest;
        //turn off burning
        self.status_effects = Vec::new();
    }

    pub fn is_point_in_bounds(&self, point: (f32, f32)) -> bool {
        //get four corners of box
        let left = self.pos.0 - (0.5 * self.data.bounding_box.2 as f32);
        let right = left + self.data.bounding_box.2 as f32;
        let top = self.pos.1 - (0.5 * self.data.bounding_box.3 as f32);
        let bottom = top + self.data.bounding_box.3 as f32;
        // circ!(x = left, y = top, d = 1, color = 0x000000ff);
        // circ!(x = right, y = top, d = 1, color = 0x000000ff);
        // circ!(x = left, y = bottom, d = 1, color = 0x000000ff);
        // circ!(x = right, y = bottom, d = 1, color = 0x000000ff);
        point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom
    }

    pub fn draw_health_bar(&self) {
        let d_p = self.head_position();
        let x = d_p.0;
        let y = d_p.1;
        let y_bar = y - 2.;
        let w_bar = (0.06 * self.data.max_health).clamp(6.0, 15.0);
        let h_bar = 2;
        let x_bar = x - w_bar / 2.;
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

    pub fn draw_strategy_icon(&self) {
        match self.attack_strategy {
            AttackStrategy::Flank { .. } => {
                let draw_pos = self.head_position();
                text!(
                    "F",
                    x = draw_pos.0,
                    y = draw_pos.1 - 10.,
                    font = Font::S,
                    color = ACID_GREEN
                );
            }
            AttackStrategy::SeekTarget => {
                //
            }
            AttackStrategy::Flee { .. } => {
                let draw_pos = self.head_position();
                text!(
                    "!",
                    x = draw_pos.0,
                    y = draw_pos.1 - 10.,
                    font = Font::M,
                    color = DAMAGE_TINT_RED
                );
            }
            _ => {}
        }
    }

    pub fn draw_status_effects(&self) {
        let base_pos = self.head_position();
        let mut offset = 0.0;

        // Keep track of which status types we've already drawn
        let mut drawn_statuses = Vec::new();

        // First draw all status effects
        for status in &self.status_effects {
            // Convert status to a comparable type ignoring internal values
            let status_type = match status {
                Status::Poison => "poison",
                Status::Healing => "healing",
                Status::Freeze => "freeze",
                Status::Burn { .. } => "burn",
            };

            // Skip if we've already drawn this status type
            if drawn_statuses.contains(&status_type) {
                continue;
            }

            // Add to drawn list and draw the status
            drawn_statuses.push(status_type);

            let name = match status {
                Status::Poison => "status_poisoned",
                Status::Healing => "status_healing",
                Status::Freeze => "status_frozen",
                Status::Burn { .. } => "status_burning",
            };

            sprite!(
                name,
                x = base_pos.0 + offset - 4.,
                y = base_pos.1 - 4.,
                sw = 16,
                fps = fps::FAST
            );

            offset += 4.0;
        }

        // Then check for flee status and draw it if present
        if matches!(self.attack_strategy, AttackStrategy::Flee { .. }) {
            sprite!(
                "status_fleeing",
                x = base_pos.0 + offset - 4.,
                y = base_pos.1 - 4.,
                sw = 16,
                fps = fps::FAST
            );
        }
    }

    pub fn set_new_target_move_position(&mut self, target: &(f32, f32), rng: &mut RNG) {
        let mut dir_x = target.0 - self.pos.0;
        let dir_y = target.1 - self.pos.1;

        if dir_x > 0. {
            self.display.as_mut().unwrap().is_facing_left = false;
        } else if dir_x < 0. {
            self.display.as_mut().unwrap().is_facing_left = true;
        }
        //if you are already in range on the x axis, only move on the y axis
        //This looks better, especially for ranged units
        if !matches!(self.attack_strategy, AttackStrategy::Flee { .. })
            && dir_x.abs() < self.data.range
        {
            dir_x = 0.;
        }
        // Calculate the length (magnitude) of the direction vector
        let length = (dir_x * dir_x + dir_y * dir_y).sqrt().max(f32::EPSILON);

        // Normalize the direction vector
        let norm_dir_x = dir_x / length;
        let norm_dir_y = dir_y / length;

        let rand_x = rng.next_in_range(0, 10) as f32 * norm_dir_x.signum();

        let rand_y = rng.next_in_range(0, 8) as f32 * norm_dir_y.signum();
        let mut new_x = self.pos.0 + norm_dir_x * self.calculated_speed() + rand_x;
        let mut new_y = self.pos.1 + norm_dir_y * self.calculated_speed() + rand_y;
        //cap new_x, new_y within the bounds of the map so they don't go off screen
        new_x = new_x.clamp(MAP_BOUNDS.0, MAP_BOUNDS.1);
        new_y = new_y.clamp(MAP_BOUNDS.2, MAP_BOUNDS.3);
        self.target_pos = (new_x, new_y);
        self.state = UnitState::Moving;
    }

    // fn calculate_separation(&self, nearby_units: &[&Unit]) -> (f32, f32) {
    //     let mut separation = (0.0, 0.0);
    //     let mut repulsions = Vec::new();
    //     let separation_radius = 8.;
    //     for other in nearby_units {
    //         if distance_between(self.pos, other.pos) < separation_radius {
    //             let away = (self.pos.0 - other.pos.0, self.pos.1 - other.pos.1);
    //             let length = (away.0 * away.0 + away.1 * away.1).sqrt();
    //             let strength = 2.0 / length.max(0.0001);
    //             let normalized = (away.0 / length, away.1 / length);
    //             repulsions.push((strength, normalized));
    //         }
    //     }
    //     repulsions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    //     repulsions.truncate(3);
    //     for (strength, direction) in repulsions {
    //         separation.0 += direction.0 * strength.powf(2.0);
    //         separation.1 += direction.1 * strength.powf(2.0);
    //     }
    //     separation
    // }
    pub fn move_towards_target(&self) -> (f32, f32) {
        // Calculate direction towards target
        let dir_x = self.target_pos.0 - self.pos.0;
        let dir_y = self.target_pos.1 - self.pos.1;
        let length = (dir_x * dir_x + dir_y * dir_y).sqrt();
        let norm_dir_x = dir_x / length;
        let norm_dir_y = dir_y / length;
        // Calculate new position
        let mut new_x = self.pos.0 + norm_dir_x * self.calculated_speed() / 20.;
        let mut new_y = self.pos.1 + norm_dir_y * self.calculated_speed() / 20.;
        if dir_x > 0. {
            new_x = new_x.min(self.target_pos.0);
        } else {
            new_x = new_x.max(self.target_pos.0);
        }
        if dir_y > 0. {
            new_y = new_y.min(self.target_pos.1);
        } else {
            new_y = new_y.max(self.target_pos.1);
        }
        (new_x, new_y)
    }

    pub fn reached_target(&self) -> bool {
        let mut reached_target = false;
        if distance_between(self.pos, self.target_pos) < self.calculated_speed() / 20. {
            reached_target = true;
        }
        reached_target
    }
    //Not using this for now - but if we need some more control over movement we can
    pub fn new_target_position(&mut self, target: &(f32, f32), rng: &mut RNG) {
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

        let new_x = self.pos.0 + norm_dir_x * (self.calculated_speed() / 50.) + rand_x;
        let new_y = self.pos.1 + norm_dir_y * (self.calculated_speed() / 50.) + rand_y;
        self.target_pos = (new_x, new_y);
        self.state = UnitState::Moving;
    }
    pub fn apply_status_effects(&mut self) {
        // Create a vector to store statuses that should be removed
        let mut statuses_to_remove = Vec::new();
        let mut total_damage = 0.0;
        // Iterate through statuses with their indices
        for (index, status) in self.status_effects.iter_mut().enumerate() {
            match status {
                Status::Poison => {
                    // Apply poison damage
                }
                Status::Healing => {
                    // Apply healing
                }
                Status::Freeze => {
                    // Apply freeze effect
                }
                Status::Burn { timer } => {
                    // Apply burn damage
                    total_damage += 0.1;
                    *timer -= 1;

                    // If timer reaches 0, mark for removal
                    if *timer == 0 {
                        statuses_to_remove.push(index);
                    }
                }
            }
        }
        if total_damage > 0.0 {
            self.apply_damage(total_damage);
        }

        // Remove statuses in reverse order to maintain correct indices
        for index in statuses_to_remove.iter().rev() {
            self.status_effects.remove(*index);
        }
    }

    pub fn apply_damage(&mut self, damage: f32) {
        self.health -= damage;
        self.health = self.health.max(0.);
        self.display.as_mut().unwrap().damage_effect_timer = DAMAGE_EFFECT_TIME;
    }

    pub fn take_attack(&mut self, attack: &Attack, rng: &mut RNG) {
        if self.state != UnitState::Dead {
            self.apply_damage(attack.damage);
            //apply terrifying effect to cause units to flee
            if attack.attributes.contains(&Attribute::Terrifying)
                && !self.data.has_attribute(&Attribute::Stalwart)
            {
                let flee_chance = 3;
                if rng.next() % flee_chance == 0 {
                    self.attack_strategy = AttackStrategy::Flee { timer: (5) };
                    self.state = UnitState::Idle;
                }
            //Ranged units will sometimes flee when hit by melee units
            } else if self.data.has_attribute(&Attribute::Ranged)
                && !attack.attributes.contains(&Attribute::Ranged)
                && !self.data.has_attribute(&Attribute::Stalwart)
            {
                let flee_chance = 2;
                if rng.next() % flee_chance == 0 {
                    self.attack_strategy = AttackStrategy::Flee { timer: (5) };
                    self.state = UnitState::Idle;
                }
            }
            //if it is a fire effect, then add a burn status to this unit
            if !self.data.has_attribute(&Attribute::FireResistance)
                && attack.attributes.contains(&Attribute::FireEffect)
            {
                let new_status = Status::Burn { timer: (300) };
                self.status_effects.push(new_status);
            } else if self.data.has_attribute(&Attribute::FireResistance)
                && attack.attributes.contains(&Attribute::FireEffect)
            {
                turbo::println!("FIRE BLOCKED");
            }
            if self.display.as_ref().unwrap().blood_splatter.is_none() {
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
                self.display.as_mut().unwrap().blood_splatter = Some(new_splatter);
            }
        }
    }

    pub fn start_attack(&mut self, target_unit_id: u32) -> Attack {
        self.attack_timer = self.data.attack_time;
        self.state = UnitState::Attacking;
        //create the actual attack
        let size = 1;
        let attack = Attack::new(
            target_unit_id,
            2.,
            self.pos,
            self.data.damage,
            self.data.splash_area,
            size,
            self.data.attributes.clone(),
        );
        attack
    }

    pub fn create_footprint(&mut self) {
        let mut color = POO_BROWN;
        match self.display.as_ref().unwrap().footprint_status {
            FootprintStatus::Clean => {
                //do nothing
            }
            FootprintStatus::Poopy => {
                color = POO_BROWN;
            }
            FootprintStatus::Acid => {
                color = ACID_GREEN;
            }
        }
        let fp = Footprint {
            pos: self.foot_position(),
            color: color as u32,
            lifetime: FOOTPRINT_LIFETIME,
        };
        self.display.as_mut().unwrap().footprints.push(fp);
    }

    pub fn distance_to(&self, pos: &(f32, f32)) -> f32 {
        let dx = self.pos.0 - pos.0;
        let dy = self.pos.1 - pos.1;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn is_unit_in_range(&self, other: &Unit) -> bool {
        let other_pos = other.pos;
        let dx = (self.pos.0 - other_pos.0).abs();
        let dy = (self.pos.1 - other_pos.1).abs();
        if dx < self.data.range && dy < MAX_Y_ATTACK_DISTANCE {
            return true;
        }
        false
    }

    pub fn draw_position(&self) -> (f32, f32) {
        let mut d_x = -0.5 * self.data.bounding_box.2 as f32 - self.data.bounding_box.0 as f32;
        if self.flip_x() {
            d_x = -d_x - self.data.sprite_width as f32 + 0.5 * self.data.bounding_box.0 as f32;
        }
        let d_y = -0.5 * self.data.bounding_box.3 as f32 - self.data.bounding_box.1 as f32;
        (self.pos.0 + d_x, self.pos.1 + d_y)
    }

    pub fn foot_position(&self) -> (f32, f32) {
        let d_y = self.data.bounding_box.3 as f32 / 2.;
        return (self.pos.0, self.pos.1 + d_y);
    }

    pub fn head_position(&self) -> (f32, f32) {
        let d_y = self.data.bounding_box.3 as f32 / 2.;
        return (self.pos.0, self.pos.1 - d_y);
    }
    pub fn flip_x(&self) -> bool {
        //self.team == 1
        self.display.as_ref().unwrap().is_facing_left
    }

    pub fn calculated_speed(&self) -> f32 {
        //do an adjustment if you are fleeing or flanking
        let mut calc_speed = self.data.speed;
        let flank_adj = 1.2;
        let flee_adj = 2.0;
        match self.attack_strategy {
            AttackStrategy::Flank { .. } => {
                calc_speed = calc_speed * flank_adj;
            }
            AttackStrategy::Flee { .. } => calc_speed = calc_speed * flee_adj,
            _ => {}
        }
        calc_speed
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum AttackStrategy {
    AttackClosest,
    TargetLowestHealth,
    Flank { stage: FlankStage },
    SeekTarget,
    Flee { timer: i32 },
    MoveRandom,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum FlankStage {
    Vertical,
    Horizontal,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum Attribute {
    ExplodeOnDeath,
    ExplosiveAttack,
    Terrifying,
    Stalwart,
    FireEffect,
    FireResistance,
    Ranged,
    Flanker,
}

impl FromStr for Attribute {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Terrifying" => Ok(Attribute::Terrifying),
            "Stalwart" => Ok(Attribute::Stalwart),
            "FireEffect" => Ok(Attribute::FireEffect),
            "FireResistance" => Ok(Attribute::FireResistance),
            "ExplodeOnDeath" => Ok(Attribute::ExplodeOnDeath),
            "ExplosiveAttack" => Ok(Attribute::ExplosiveAttack),
            "Flanker" => Ok(Attribute::Flanker),
            "Ranged" => Ok(Attribute::Ranged),
            _ => Err(format!("Unknown attribute: {}", s)),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum Status {
    Poison,
    Healing,
    Freeze,
    Burn { timer: u32 },
}
/*
TODO: Apply burn when attack comes in
If burned - take damage and lessen burn timer
Give units a list of status effects
Whenever unit is idle, apply burn damage
visualize burn
make status effect a string for now. If you have X status, add whatever text, then render all of them
*/

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
    pub bounding_box: (i32, i32, i32, i32),
    pub attributes: Vec<Attribute>,
}

impl UnitData {
    pub fn has_attribute(&self, attr: &Attribute) -> bool {
        self.attributes.contains(attr)
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum UnitState {
    MarchingIn,
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
    pub data: UnitData,
    pub pos: (f32, f32),
    pub flip_x: bool,
    //pub bounding_box: (i32, i32, i32, i32),
    pub state: UnitState,
}

impl UnitPreview {
    pub fn new(unit_type: String, data: UnitData, pos: (f32, f32), flip_x: bool) -> Self {
        Self {
            unit_type, //placeholder, gets overwritten when they are drawn, but I can't figure out how to do it more logically than this
            animator: Animator::new(Animation {
                name: "placeholder".to_string(),
                s_w: 16,
                num_frames: 0,
                loops_per_frame: 0,
                is_looping: true,
            }),
            data,
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
            s_w: self.data.sprite_width,
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
        let mut d_x = -0.5 * self.data.bounding_box.2 as f32 - self.data.bounding_box.0 as f32;
        if self.flip_x {
            d_x = -d_x - self.data.sprite_width as f32 + 0.5 * self.data.bounding_box.0 as f32;
        }
        let d_y = -0.5 * self.data.bounding_box.3 as f32 - self.data.bounding_box.1 as f32;
        (self.pos.0 + d_x, self.pos.1 + d_y)
    }

    pub fn is_point_in_bounds(&self, point: (f32, f32)) -> bool {
        //get four corners of box
        let left = self.pos.0 - (0.5 * self.data.bounding_box.2 as f32);
        let right = left + self.data.bounding_box.2 as f32;
        let top = self.pos.1 - (0.5 * self.data.bounding_box.3 as f32);
        let bottom = top + self.data.bounding_box.3 as f32;
        // circ!(x = left, y = top, d = 1, color = 0x000000ff);
        // circ!(x = right, y = top, d = 1, color = 0x000000ff);
        // circ!(x = left, y = bottom, d = 1, color = 0x000000ff);
        // circ!(x = right, y = bottom, d = 1, color = 0x000000ff);
        point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom
    }

    pub fn draw_unit_details(&self) {
        //create a panel
        let pw = 100; // Made panel wider to accommodate text
        let ph = 80;
        let border_color = OFF_BLACK;
        let panel_color = LIGHT_GRAY;
        //TODO: Make this calculate based on actual sprite bounding box width
        let px = self.pos.0 + 20.;
        let py = self.pos.1;
        rect!(
            x = px,
            y = py,
            h = ph,
            w = pw,
            color = panel_color,
            border_color = border_color,
            border_radius = 2,
            border_width = 2
        );

        // Header
        text!("UNIT DETAILS", x = px + 5., y = py + 5.);

        // Stats rows - each line is 15 pixels apart
        let damage_text = format!("DAMAGE: {}", self.data.damage);
        let speed_text = format!("SPEED: {}", self.data.speed);
        let health_text = format!("HEALTH: {}", self.data.max_health);

        text!(&damage_text, x = px + 5., y = py + 25.);
        text!(&speed_text, x = px + 5., y = py + 35.);
        text!(&health_text, x = px + 5., y = py + 45.);
    }
}
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Footprint {
    pub pos: (f32, f32),
    pub color: u32,
    pub lifetime: u32,
}

impl Footprint {
    pub fn draw(&mut self) {
        if self.lifetime != 0 {
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
            rect!(
                x = self.pos.0,
                y = self.pos.1,
                color = draw_color,
                w = 1,
                h = 1
            );
        }
    }
}
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum FootprintStatus {
    Clean,
    Poopy,
    Acid,
}
