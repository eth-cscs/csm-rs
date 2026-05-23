use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Ims {
  is_recipe: bool,
  id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Product {
  name: String,
  version: String,
  r#type: String,
}
