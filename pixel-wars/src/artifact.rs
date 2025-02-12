use crate::*;

pub const ARTIFACT_KINDS: &[ArtifactKind] = &[
    ArtifactKind::StrengthOfTheFallen {
        percent_per_unit: 1.0,
    },
    ArtifactKind::SnipersFocus {
        percent_per_pixel: 0.5,
    },
    ArtifactKind::BloodSucker { steal_factor: 0.1 },
    ArtifactKind::GiantSlayer { boost_factor: 2.0 },
    ArtifactKind::SeeingGhosts {
        chance_to_occur: 40,
    },
    ArtifactKind::SpeedRunner {
        change_to_occur: 40,
    },
    ArtifactKind::DoctorsIn { num_kits: 6 },
    ArtifactKind::Necromancer { revival_chance: 25 },
];

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy, EnumIter, Display)]
pub enum ArtifactKind {
    StrengthOfTheFallen { percent_per_unit: f32 },
    SnipersFocus { percent_per_pixel: f32 },
    FlameWard { resistance_percent: f32 },
    BloodSucker { steal_factor: f32 },
    GiantSlayer { boost_factor: f32 },
    SeeingGhosts { chance_to_occur: u32 },
    SpeedRunner { change_to_occur: u32 },
    DoctorsIn { num_kits: u8 },
    Necromancer { revival_chance: u8 },
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Artifact {
    pub artifact_kind: ArtifactKind,
    pub text: String,
    pub team: u8,
    pub animator: Animator,
}

impl Artifact {
    pub fn new(kind: ArtifactKind, team: u8) -> Self {
        let text = match kind {
            ArtifactKind::StrengthOfTheFallen { .. } => {
                "Increase damage for each dead unit on your team"
            }
            ArtifactKind::SnipersFocus { .. } => "Increase damage for all ranged attacks",
            ArtifactKind::FlameWard { .. } => "Give all of your units fire resistance",
            ArtifactKind::BloodSucker { .. } => "Suck life from your enemies",
            ArtifactKind::GiantSlayer { .. } => "Deal double damage to large enemies",
            ArtifactKind::SeeingGhosts { .. } => "Some enemies will get scared",
            ArtifactKind::SpeedRunner { .. } => "All your units start with Haste",
            ArtifactKind::DoctorsIn { .. } => "Deploy Medkits on your team's side",
            ArtifactKind::Necromancer { .. } => "Revive some of your units after the battle",
        }
        .to_string();
        let display_name = format!("{:?}", kind)
            .split('{')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        Self {
            artifact_kind: kind,
            text,
            team,
            animator: Animator::new(Animation {
                name: display_name,
                s_w: 16,
                num_frames: 1,
                loops_per_frame: 8,
                is_looping: false,
            }),
        }
    }

    pub fn is_hovered(&self, pos: (i32, i32), mouse_pos: (i32, i32)) -> bool {
        let (mouse_x, mouse_y) = mouse_pos;
        let (pack_x, pack_y) = pos;
        let width = 80;
        let height = 80;
        mouse_x >= pack_x as i32
            && mouse_x <= pack_x as i32 + width as i32
            && mouse_y >= pack_y as i32
            && mouse_y <= pack_y as i32 + height as i32
    }

    pub fn draw_card(&mut self, pos: (i32, i32), mouse_pos: (i32, i32)) {
        //do some card stuff, with a position
        let pw = 80; // Made panel wider to accommodate text
        let ph = 80;
        let border_color = OFF_BLACK;
        let mut panel_color = DARK_GRAY;
        if self.is_hovered(pos, mouse_pos) {
            panel_color = LIGHT_GRAY;
        }
        let px = pos.0;
        let py = pos.1;
        rect!(
            x = px,
            y = py,
            h = ph,
            w = pw,
            color = panel_color,
            border_color = border_color,
            border_radius = 6,
            border_width = 2
        );
        let sprite_pos = (pos.0 + 32, pos.1 + 10);
        let text_pos = (pos.0 + 3, pos.1 + 36);
        self.draw_sprite_scaled(sprite_pos, 2.0);
        self.draw_effect_text(text_pos);
    }

    pub fn draw_sprite(&mut self, pos: (i32, i32)) {
        self.draw_sprite_scaled(pos, 1.0);
    }

    pub fn draw_sprite_scaled(&mut self, pos: (i32, i32), scale: f32) {
        let sprite_name = self.artifact_kind.to_string();
        //turbo::println!("A_N {}", sprite_name);
        // match self.artifact_kind {
        //     ArtifactKind::StrengthOfTheFallen => {
        //         color = POO_BROWN;
        //     }
        //     ArtifactKind::SnipersFocus => {
        //         color = OFF_BLACK as usize;
        //     }
        //     ArtifactKind::FlameWard => {
        //         color = DAMAGE_TINT_RED;
        //     }
        //     ArtifactKind::TrapArtist => {
        //         color = WHITE;
        //     }
        //     ArtifactKind::ShotOutACannon => {
        //         color = ACID_GREEN;
        //     }
        //     ArtifactKind::BloodSucker => {
        //         color = DAMAGE_TINT_RED;
        //     }
        //     ArtifactKind::GiantSlayer => {
        //         color = WHITE;
        //     }
        //     ArtifactKind::SeeingGhosts => {
        //         color = WHITE;
        //     }
        // }
        // //match on the artifact type to get the sprite
        // let d = 12.0 * scale;
        //sprite!(&sprite_name, x = pos.0, y = pos.1, sw = 16);
        //circ!(color = color, x = pos.0, y = pos.1, d = d);
        if scale == 2.0 {
            sprite!(
                &sprite_name,
                x = pos.0 - 8,
                y = pos.1 - 8,
                scale = scale,
                sw = 16
            );
        } else {
            self.animator.draw((pos.0 as f32, pos.1 as f32), false);
            //log!("animator info: {:?}", self.animator.cur_anim);

            self.animator.update();
        }
    }

    pub fn icon_is_hovered(&mut self, pos: (i32, i32), mouse_pos: (i32, i32)) -> bool {
        let (icon_x, icon_y) = pos;
        let (mouse_x, mouse_y) = mouse_pos;

        mouse_x >= icon_x && mouse_x < icon_x + 12 && mouse_y >= icon_y && mouse_y < icon_y + 12
    }

    pub fn draw_name(&mut self, pos: (i32, i32)) {
        // Get the name of the artifact from artifact kind
        let name = format!("{:?}", self.artifact_kind)
            .split('{')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        // Calculate width based on character length (each character = 5 pixels)
        let width = name.len() as i32 * 5;

        // Position and dimensions
        let (x, y) = pos;
        let height = 7;
        let margin = 4;

        // Draw background box
        rect!(
            x = x - margin - 12,
            y = y - margin,
            w = width + 2 * margin,
            h = height + 2 * margin,
            border_radius = 2,
            border_color = SHADOW_COLOR,
            border_width = 1,
            color = DARK_GRAY
        );

        // Draw text
        text!(&name, x = x - 12, y = y, color = WHITE);
    }

    pub fn play_effect(&mut self) {
        let anim = Animation {
            name: self.artifact_kind.to_string(),
            s_w: 16,
            num_frames: 6,
            loops_per_frame: 8,
            is_looping: false,
        };

        let next_anim = Animation {
            name: self.artifact_kind.to_string(),
            s_w: 16,
            num_frames: 1,
            loops_per_frame: 8,
            is_looping: false,
        };
        self.animator.set_cur_anim(anim);
        self.animator.set_next_anim(Some(next_anim));
    }

    pub fn draw_effect_text(&self, pos: (i32, i32)) {
        let texts = split_text_at_spaces(&self.text);

        for (i, line) in texts.iter().enumerate() {
            let y_offset = pos.1 + (i as i32 * 8);
            text!(line, x = pos.0, y = y_offset);
        }
    }
}
