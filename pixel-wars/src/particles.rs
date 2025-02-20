use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum BurstSource {
    Point(f32, f32),
    Circle { center: (f32, f32), radius: f32 },
    Rectangle { min: (f32, f32), max: (f32, f32) },
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Particle {
    pub pos: (f32, f32),
    pub vel: (f32, f32),
    pub size: u32,
    pub color: u32,
    pub lifetime: f32,
    pub remaining_life: f32,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct BurstConfig {
    pub source: BurstSource,
    pub x_velocity: (f32, f32), // (min, max)
    pub y_velocity: (f32, f32),
    pub lifetime: (f32, f32),
    pub color: u32,
    pub size: (u32, u32),
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
        // Get position based on source type
        let pos = match &config.source {
            BurstSource::Point(x, y) => (*x, *y),
            BurstSource::Circle { center, radius } => {
                let angle = (rand() as f32 / u32::MAX as f32) * std::f32::consts::TAU;
                let dist = (rand() as f32 / u32::MAX as f32) * radius;
                (center.0 + dist * angle.cos(), center.1 + dist * angle.sin())
            }
            BurstSource::Rectangle { min, max } => {
                let x_range = (max.0 - min.0) as u32;
                let y_range = (max.1 - min.1) as u32;

                let x = if x_range == 0 {
                    min.0
                } else {
                    min.0 + (rand() % x_range) as f32
                };

                let y = if y_range == 0 {
                    min.1
                } else {
                    min.1 + (rand() % y_range) as f32
                };

                (x, y)
            }
        };

        // Get random velocities between min and max
        let vx = config.x_velocity.0
            + (rand() as f32 / u32::MAX as f32) * (config.x_velocity.1 - config.x_velocity.0);
        let vy = config.y_velocity.0
            + (rand() as f32 / u32::MAX as f32) * (config.y_velocity.1 - config.y_velocity.0);

        // Get random lifetime between min and max
        let lifetime = config.lifetime.0
            + (rand() as f32 / u32::MAX as f32) * (config.lifetime.1 - config.lifetime.0);

        let size = config.size.0 + (rand() / u32::MAX) * (config.size.1 - config.size.0);

        Particle {
            pos,
            vel: (vx, vy),
            color: config.color,
            lifetime,
            remaining_life: lifetime,
            size,
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
                    d = particle.size,
                    color = particle.color
                );
            }
        }
    }
}
