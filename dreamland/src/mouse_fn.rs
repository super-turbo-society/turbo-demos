use crate::*;

// Trait to handle mouse interaction
pub trait Hover {
    // Function to return reference to Self if mouse is hovering over
    fn hover(&mut self, hitbox: (i32, i32, i32, i32), mx: i32, my: i32) -> Option<&mut Self> {
        if mx >= hitbox.0 && mx <= hitbox.0 + hitbox.2 
        && my >= hitbox.1 && my <= hitbox.1 + hitbox.3 {
            Hover::hover_state(self, true);
            return Some(self)
        } else {
            Hover::hover_state(self, false);
            return None
        }
    }

    fn hover_state(&mut self, _hover: bool) {}
}

impl Hover for Dreamer{}
impl Hover for VialSource{}
impl Hover for Vial{}
impl Hover for SandTap{}
impl Hover for UIButton{
    fn hover_state(&mut self, hover: bool) {
        if hover {
            self.hovered = true;
        } else {
            self.hovered = false;
        }
    }
}