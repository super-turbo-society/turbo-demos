use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum AttributeType {
    Damage,
    Speed,
    Health,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum UnitPackType {
    Normal {
        unit_type: UnitType,
        unit_preview: UnitPreview,
        unit_count: u32,
    },
    FallenUnits {
        fallen_unit_types: Vec<UnitType>,
    },
    Artifact {
        kind: ArtifactKind,
    },
    Transform {
        original_unit_type: UnitType,
        original_unit_preview: UnitPreview,
        new_unit_type: UnitType,
        new_unit_preview: UnitPreview,
        is_upgrade: bool,
    },
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct UnitPack {
    pub is_picked: bool,
    pub pos: (f32, f32),
    pub width: u32,
    pub height: u32,
    pub pack_type: UnitPackType, // New field to distinguish pack types
}

impl UnitPack {
    pub fn new_normal(
        unit_type: UnitType,
        unit_preview: UnitPreview,
        unit_count: u32,
        pos: (f32, f32),
    ) -> Self {
        UnitPack {
            pack_type: UnitPackType::Normal {
                unit_preview,
                unit_count,
                unit_type,
            },
            is_picked: false,
            pos,
            width: 80,
            height: 80,
        }
    }

    pub fn new_fallen_units(fallen_unit_types: Vec<UnitType>, pos: (f32, f32)) -> Self {
        UnitPack {
            pack_type: UnitPackType::FallenUnits { fallen_unit_types },
            is_picked: false,
            pos,
            width: 80,
            height: 80,
        }
    }

    pub fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    pub fn is_hovered(&self, mouse_pos: (i32, i32)) -> bool {
        let (mouse_x, mouse_y) = mouse_pos;
        let (pack_x, pack_y) = self.pos;

        mouse_x >= pack_x as i32
            && mouse_x <= pack_x as i32 + self.width as i32
            && mouse_y >= pack_y as i32
            && mouse_y <= pack_y as i32 + self.height as i32
    }

    pub fn draw_pack_card(&self, mouse_pos: (i32, i32)) {
        //create a panel
        let pw = 80;
        let ph = 80;
        let border_color = OFF_BLACK;
        let (panel_color, hover_color) = match self.pack_type {
            UnitPackType::Normal { .. } => (DARK_GRAY, LIGHT_GRAY),
            UnitPackType::FallenUnits { .. } => (COLOR_BRONZE, COLOR_LIGHT_BRONZE),
            UnitPackType::Artifact { .. } => (COLOR_ARTIFACT_PANEL, COLOR_ARTIFACT_PANEL_HOVER),
            UnitPackType::Transform { .. } => (COLOR_TRANSFORM_PANEL, COLOR_TRANSFORM_PANEL_HOVER),
        };

        let mut current_panel_color = panel_color;
        if self.is_hovered(mouse_pos) {
            current_panel_color = hover_color;
        }
        let px = self.pos.0;
        let py = self.pos.1;
        rect!(
            x = px,
            y = py,
            h = ph,
            w = pw,
            color = current_panel_color,
            border_color = border_color,
            border_radius = 6,
            border_width = 2
        );

        match &self.pack_type {
            UnitPackType::Normal {
                unit_preview,
                unit_count,
                unit_type,
            } => {
                // Header
                let c = Self::capitalize(&unit_type.as_string());
                let txt = format!("{} {}s", unit_count, c);
                text!(&txt, x = px + 5., y = py + 5.);

                // Stats rows - each line is 15 pixels apart
                let damage_text = format!("DAMAGE: {}", unit_preview.data.damage);
                let damage_text_length = damage_text.len() as i32 * 5;
                let speed_text = format!("SPEED: {}", unit_preview.data.speed);
                let speed_text_length = speed_text.len() as i32 * 5;
                let health_text = format!("HEALTH: {}", unit_preview.data.max_health);
                let health_text_length = health_text.len() as i32 * 5;

                text!(&damage_text, x = px + 5., y = py + 25.);
                self.draw_attributes(
                    (px - 2. + damage_text_length as f32, py + 25.0),
                    AttributeType::Damage,
                    &unit_preview.data.attributes,
                );
                text!(&speed_text, x = px + 5., y = py + 35.);
                self.draw_attributes(
                    (px - 2. + speed_text_length as f32, py + 35.0),
                    AttributeType::Speed,
                    &unit_preview.data.attributes,
                );
                text!(&health_text, x = px + 5., y = py + 45.);
                self.draw_attributes(
                    (px - 2. + health_text_length as f32, py + 45.0),
                    AttributeType::Health,
                    &unit_preview.data.attributes,
                );
            }
            UnitPackType::FallenUnits { fallen_unit_types } => {
                power_text!("Revive", x = px, y = py + 5., center_width = self.width);

                // Count occurrences of each unit type
                let mut unit_counts: HashMap<UnitType, usize> = HashMap::new();
                for unit_type in fallen_unit_types {
                    *unit_counts.entry(*unit_type).or_insert(0) += 1;
                }

                // Sort the unit types to ensure consistent display
                let mut sorted_units: Vec<_> = unit_counts.into_iter().collect();
                sorted_units.sort_by_key(|&(k, _)| k);

                // Display unit counts
                let mut y_offset = 25.0;
                for (unit_type, count) in sorted_units {
                    let capitalized_type = Self::capitalize(&unit_type.as_string());
                    let unit_text = format!("{}x {}", count, capitalized_type);
                    text!(&unit_text, x = px + 5., y = py + y_offset);
                    y_offset += 15.0;
                }
            }
            UnitPackType::Artifact { kind } => {
                //artifact header
                power_text!("Artifact", x = px, y = py + 5., center_width = self.width);
                //artifact image
                let sprite_name = kind.to_string();
                sprite!(
                    &sprite_name,
                    x = px - 16.0 + 40.0,
                    y = py - 16.0 + 26.0,
                    sw = 16,
                    scale = 2.0
                );
                //artifact text
                let text = Artifact::artifact_text(*kind);
                let texts = split_text_at_spaces(&text);
                for (i, line) in texts.iter().enumerate() {
                    let y_offset = py + (i as f32 * 8.0) + 40.0;
                    text!(line, x = px + 4.0, y = y_offset);
                }
            }
            UnitPackType::Transform {
                original_unit_preview,
                new_unit_preview,
                is_upgrade,
                ..
            } => {
                //upgrade header
                let mut t = "Transform";
                if *is_upgrade {
                    t = "Evolve";
                }
                power_text!(&t, x = px, y = py + 5., center_width = self.width);

                // Stats rows - each line is 15 pixels apart
                let dmg_change = new_unit_preview.data.damage - original_unit_preview.data.damage;
                let damage_sign = if dmg_change >= 0. { "+" } else { "" }; // Empty string because negative sign will be included automatically
                let damage_text = format!("DAMAGE{}{}", damage_sign, dmg_change);
                let damage_text_length = damage_text.len() as i32 * 5;

                let spd_change = new_unit_preview.data.speed - original_unit_preview.data.speed;
                let speed_sign = if spd_change >= 0. { "+" } else { "" };
                let speed_text = format!("SPEED{}{}", speed_sign, spd_change);
                let speed_text_length = speed_text.len() as i32 * 5;

                let health_change =
                    new_unit_preview.data.max_health - original_unit_preview.data.max_health;
                let health_sign = if health_change >= 0. { "+" } else { "" };
                let health_text = format!("HEALTH{}{}", health_sign, health_change);
                let health_text_length = health_text.len() as i32 * 5;

                text!(&damage_text, x = px + 5., y = py + 25.);
                self.draw_attributes(
                    (px - 2. + damage_text_length as f32, py + 25.0),
                    AttributeType::Damage,
                    &new_unit_preview.data.attributes,
                );
                text!(&speed_text, x = px + 5., y = py + 35.);
                self.draw_attributes(
                    (px - 2. + speed_text_length as f32, py + 35.0),
                    AttributeType::Speed,
                    &new_unit_preview.data.attributes,
                );
                text!(&health_text, x = px + 5., y = py + 45.);
                self.draw_attributes(
                    (px - 2. + health_text_length as f32, py + 45.0),
                    AttributeType::Health,
                    &new_unit_preview.data.attributes,
                );

                text!("-->", x = px + 31.0, y = py + 46.0 + 20.0);
            }
        }
    }

    pub fn draw_attributes(
        &self,
        pos: (f32, f32),
        attribute_type: AttributeType,
        attributes: &Vec<Attribute>,
    ) {
        let mut offset_x = 0.0;
        let (x, y) = pos;

        // Collect matching attributes first
        let matching_attrs: Vec<_> = attributes
            .iter()
            .filter_map(|&attr| match (attribute_type, attr) {
                // Damage attributes
                (AttributeType::Damage, Attribute::FireAttack) => Some("status_burning"),
                (AttributeType::Damage, Attribute::FreezeAttack) => Some("status_frozen"),
                (AttributeType::Damage, Attribute::PoisonAttack) => Some("status_poisoned"),
                (AttributeType::Damage, Attribute::Berserk) => Some("status_berserk"),

                // Speed attributes
                (AttributeType::Speed, Attribute::Stealth) => Some("status_invisible"),

                // Health attributes
                (AttributeType::Health, Attribute::Shielded) => Some("status_shield"),

                // If no match, return None
                _ => None,
            })
            .collect();

        // If there are matching attributes, draw the "+" sign
        if !matching_attrs.is_empty() {
            text!("+", x = x + 7.0, y = y, color = WHITE);

            // Draw sprites after the "+" sign
            for sprite_name in matching_attrs {
                offset_x += 5.0; // Add 5 pixels after the "+"
                sprite!(sprite_name, x = x + offset_x, y = y, sw = 16);
                offset_x += 2.0;
            }
        }
    }

    pub fn draw(&mut self, mouse_pos: (i32, i32)) {
        if !self.is_picked {
            self.draw_pack_card(mouse_pos);
            if let UnitPackType::Normal { unit_preview, .. } = &mut self.pack_type {
                unit_preview.pos.0 = self.pos.0 + 30.;
                unit_preview.pos.1 = self.pos.1 + self.height as f32 - 10.;
                unit_preview.update();
                unit_preview.draw();
            } else if let UnitPackType::Transform {
                original_unit_type,
                original_unit_preview,
                new_unit_type,
                new_unit_preview,
                is_upgrade,
            } = &mut self.pack_type
            {
                original_unit_preview.pos.0 = self.pos.0 + 20.0;
                original_unit_preview.pos.1 = self.pos.1 + self.height as f32 + 20.;
                original_unit_preview.update();
                original_unit_preview.draw();
                new_unit_preview.pos.0 = self.pos.0 + 50.0;
                new_unit_preview.pos.1 = self.pos.1 + self.height as f32 + 20.;
                new_unit_preview.update();
                new_unit_preview.draw();
            }
        }
    }
}

pub fn select_unit_pack(pack_index: usize, state: &mut GameState) {
    let pack = &mut state.shop[pack_index];
    if state.teams.len() == 0 {
        let team = initialize_first_team(state.data_store.as_ref().unwrap().clone());
        state.teams.push(team);
    }

    match &pack.pack_type {
        UnitPackType::Normal {
            unit_count,
            unit_type,
            ..
        } => {
            let mut i = 0;
            while i < *unit_count {
                state.teams[0].add_unit(unit_type.clone());
                i += 1;
            }
        }
        UnitPackType::FallenUnits { fallen_unit_types } => {
            // Directly add fallen unit types to the team
            for unit_type in fallen_unit_types {
                state.teams[0].add_unit(unit_type.clone());
            }
        }
        UnitPackType::Artifact { kind } => {
            let a = Artifact::new(*kind, 0);
            state.artifacts.push(a);
        }
        UnitPackType::Transform {
            original_unit_type,
            new_unit_type,
            is_upgrade,
            ..
        } => {
            if *is_upgrade {
                state.teams[0].upgraded_units.push(*original_unit_type);
            }

            // Transform the units on the team currently
            for team_unit in state.teams[0].units.iter_mut() {
                if *team_unit == *original_unit_type {
                    *team_unit = *new_unit_type
                }
            }
        }
    }

    pack.is_picked = true;
}
