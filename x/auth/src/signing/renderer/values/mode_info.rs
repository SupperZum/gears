use database::Database;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{mode_info::ModeInfo, screen::Screen};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<SK: StoreKey, DB: Database> ValueRenderer<SK, DB> for ModeInfo {
    fn format(
        &self,
        _ctx: &Context<'_, '_, DB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        // I don't see that mode ino is used in screen formatin for now, but leave this as things may change
        Ok(Vec::new())
    }
}
