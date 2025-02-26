use crate::*;
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum Attribute {
    ExplodeOnDeath,
    ExplosiveAttack,
    Terrifying,
    Stalwart,
    FireAttack,
    FireResistance,
    Ranged,
    Flanker,
    Defender,
    Shielded,
    FreezeAttack,
    PoisonAttack,
    Berserk,
    Stealth,
    Trample,
    Large,
    ParabolicAttack,
}

impl FromStr for Attribute {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Terrifying" => Ok(Attribute::Terrifying),
            "Stalwart" => Ok(Attribute::Stalwart),
            "FireAttack" => Ok(Attribute::FireAttack),
            "FireResistance" => Ok(Attribute::FireResistance),
            "ExplodeOnDeath" => Ok(Attribute::ExplodeOnDeath),
            "ExplosiveAttack" => Ok(Attribute::ExplosiveAttack),
            "Flanker" => Ok(Attribute::Flanker),
            "Ranged" => Ok(Attribute::Ranged),
            "Defender" => Ok(Attribute::Defender),
            "Shielded" => Ok(Attribute::Shielded),
            "FreezeAttack" => Ok(Attribute::FreezeAttack),
            "PoisonAttack" => Ok(Attribute::PoisonAttack),
            "Berserk" => Ok(Attribute::Berserk),
            "Stealth" => Ok(Attribute::Stealth),
            "Trample" => Ok(Attribute::Trample),
            "Large" => Ok(Attribute::Large),
            "ParabolicAttack" => Ok(Attribute::ParabolicAttack),
            _ => Err(format!("Unknown attribute: {}", s)),
        }
    }
}
