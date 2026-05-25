//! Legacy SAT `images` section shapes retained for backward
//! compatibility with older SAT files.

use serde::{Deserialize, Serialize};

/// Legacy `ims:` block referencing an IMS recipe or image by ID.
#[derive(Deserialize, Serialize, Debug)]
pub struct Ims {
  is_recipe: bool,
  id: String,
}

/// Legacy `product:` block referencing a product catalog entry by
/// name + version + type.
#[derive(Deserialize, Serialize, Debug)]
pub struct Product {
  name: String,
  version: String,
  r#type: String,
}
