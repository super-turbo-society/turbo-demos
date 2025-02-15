use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum BurstSource {
    Circle { center: (f32, f32), radius: f32 },
    Box { min: (f32, f32), max: (f32, f32) },
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Particle {
    pub pos: (f32, f32),
    pub vel: (f32, f32),
    pub color: u32,
    pub lifetime: f32,
    pub remaining_life: f32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct BurstConfig {
    pub source: BurstSource,
    pub direction: f32,
    pub spread: f32,
    pub speed: f32,
    pub speed_var: f32,
    pub color: u32,
    pub lifetime: f32,
    pub count: u32,
}

impl BurstConfig {}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct ParticleManager {
    pub bursts: Vec<Vec<Particle>>,
}

impl ParticleManager {
    pub fn new() -> Self {
        Self { bursts: Vec::new() }
    }

    pub fn create_burst(&mut self, config: &BurstConfig) {
        let mut burst = Vec::new();
        for _ in 0..config.count {
            burst.push(self.create_particle(config));
        }
        self.bursts.push(burst);
    }

    fn create_particle(&self, config: &BurstConfig) -> Particle {
        let pos = match &config.source {
            BurstSource::Circle { center, radius } => {
                let offset_angle = (rand() as f32 / u32::MAX as f32) * std::f32::consts::TAU;
                let offset_dist = (rand() as f32 / u32::MAX as f32) * radius;
                (
                    center.0 + offset_dist * offset_angle.cos(),
                    center.1 + offset_dist * offset_angle.sin(),
                )
            }
            BurstSource::Box { min, max } => {
                let rand_val = (rand() as f32 / u32::MAX as f32) * 2.0;
                turbo::println!("rand_val: {}", rand_val);
                let x = min.0 + rand_val * (max.0 - min.0);
                (x, min.1)
            }
        };

        let angle = config.direction + (rand() as f32 / u32::MAX as f32 - 0.5) * config.spread;
        let speed = config.speed + (rand() as f32 / u32::MAX as f32 - 0.5) * config.speed_var;

        Particle {
            pos,
            vel: (angle.cos() * speed, angle.sin() * speed),
            color: config.color,
            lifetime: config.lifetime,
            remaining_life: config.lifetime,
        }
    }

    pub fn update(&mut self) {
        for burst in &mut self.bursts {
            for particle in burst.iter_mut() {
                particle.pos.0 += particle.vel.0;
                particle.pos.1 += particle.vel.1;
                particle.remaining_life -= 1.0 / 60.0;
            }
        }
        self.bursts
            .retain(|burst| !burst.is_empty() && burst[0].remaining_life > 0.0);
    }

    pub fn draw(&self) {
        for burst in &self.bursts {
            for particle in burst {
                circ!(
                    x = particle.pos.0,
                    y = particle.pos.1,
                    d = 1,
                    color = particle.color
                );
            }
        }
    }
}
