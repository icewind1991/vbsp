use super::Handle;
use crate::data::*;

impl Handle<'_, StaticPropLump> {
    pub fn model(&self) -> &str {
        self.bsp.static_props.dict.name[self.prop_type as usize].as_str()
    }
}
