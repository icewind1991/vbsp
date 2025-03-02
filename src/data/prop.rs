use crate::{Handle, StaticPropLump};
use vbsp_common::{AsPropPlacement, PropPlacement};

impl<'a> AsPropPlacement<'a> for Handle<'a, StaticPropLump> {
    fn as_prop_placement(&self) -> PropPlacement<'a> {
        PropPlacement {
            model: self.model(),
            rotation: self.rotation(),
            scale: 1.0,
            origin: self.origin,
            skin: self.skin,
        }
    }
}
