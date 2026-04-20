use rust_embed;
use rust_embed::{Embed};

#[derive(Embed)]
#[folder = "assets/"]
pub struct Asset;