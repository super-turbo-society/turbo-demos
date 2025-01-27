use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum ArtifactConfig {
    DeadUnitDamageBoost { percent_per_unit: f32 },
    DistanceDamageBoost { percent_per_pixel: f32 },
    FireResistance { resistance_percent: f32 },
    TrapBoard { num_traps: u8 },
    TeamHaste,
    LifeSteal { steal_factor: f32 },
    LargeUnitDamageBoost { boost_factor: f32 },
    SuddenFright { chance_to_occur: u32 },
}

pub const ARTIFACT_KINDS: &[ArtifactKind] = &[
    ArtifactKind::StrengthOfTheFallen,
    ArtifactKind::SnipersFocus,
    ArtifactKind::FlameWard,
    ArtifactKind::TrapArtist,
    ArtifactKind::ShotOutACannon,
    ArtifactKind::BloodSucker,
    ArtifactKind::GiantSlayer,
    ArtifactKind::SeeingGhosts,
];

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy, EnumIter, Display)]

pub enum ArtifactKind {
    StrengthOfTheFallen,
    SnipersFocus,
    FlameWard,
    TrapArtist,
    ShotOutACannon,
    BloodSucker,
    GiantSlayer,
    SeeingGhosts,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Artifact {
    pub artifact_kind: ArtifactKind,
    pub config: ArtifactConfig,
    pub text: String,
    pub team: i32,
    pub animator: Animator,
}

impl Artifact {
    pub fn new(kind: ArtifactKind, team: i32) -> Self {
        // Match the name to get the preconfigured artifact
        let (config, text) = match kind {
            ArtifactKind::StrengthOfTheFallen => (
                ArtifactConfig::DeadUnitDamageBoost {
                    percent_per_unit: 1.0,
                },
                String::from("Increase damage for each dead unit on your team"),
            ),
            ArtifactKind::SnipersFocus => (
                ArtifactConfig::DistanceDamageBoost {
                    percent_per_pixel: 0.5,
                },
                String::from("Increase damage for all ranged attacks"),
            ),
            ArtifactKind::FlameWard => (
                ArtifactConfig::FireResistance {
                    resistance_percent: 50.0,
                },
                String::from("Give all of your units fire resistance"),
            ),
            ArtifactKind::TrapArtist => (
                ArtifactConfig::TrapBoard { num_traps: 12 },
                String::from("Add Traps to the board"),
            ),
            ArtifactKind::ShotOutACannon => (
                ArtifactConfig::TeamHaste,
                String::from("All your units start with Haste"),
            ),
            ArtifactKind::BloodSucker => (
                ArtifactConfig::LifeSteal { steal_factor: 0.1 },
                String::from("Suck life from your enemies"),
            ),
            ArtifactKind::GiantSlayer => (
                ArtifactConfig::LargeUnitDamageBoost { boost_factor: 2.0 },
                String::from("Deal double damage to large enemies"),
            ),
            ArtifactKind::SeeingGhosts => (
                ArtifactConfig::SuddenFright {
                    chance_to_occur: 40,
                },
                String::from("Some enemies will flee for no reason"),
            ),
            _ => panic!("Unknown artifact kind"),
        };

        Self {
            artifact_kind: kind,
            config,
            text,
            team,
            //TODO: Make this an option that doesn't show when you
            //sim on the server
            animator: Animator::new(Animation {
                name: kind.to_string(),
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
        self.draw_sprite(sprite_pos);
        self.draw_effect_text(text_pos);
    }

    pub fn draw_sprite(&mut self, pos: (i32, i32)) {
        self.draw_sprite_scaled(pos, 1.0);
    }

    pub fn draw_sprite_scaled(&mut self, pos: (i32, i32), scale: f32) {
        let mut color = ACID_GREEN;
        let sprite_name = self.artifact_kind.to_string();
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
        self.animator.draw((pos.0 as f32, pos.1 as f32), false);
        //log!("animator info: {:?}", self.animator.cur_anim);

        self.animator.update();
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
