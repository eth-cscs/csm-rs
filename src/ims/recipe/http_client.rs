use crate::{ShastaClient, error::Error};

use super::types::RecipeGetResponse;

impl ShastaClient {
  /// Fetch IMS recipes. If `recipe_id_opt` is `Some`, only that recipe is
  /// returned; otherwise all recipes.
  pub async fn ims_recipe_get(
    &self,
    recipe_id_opt: Option<&str>,
  ) -> Result<Vec<RecipeGetResponse>, Error> {
    let api_url = if let Some(recipe_id) = recipe_id_opt {
      format!("{}/ims/v2/recipes{}", self.base_url(), recipe_id)
    } else {
      format!("{}/ims/v2/recipes", self.base_url())
    };

    let response = self
      .http()
      .get(api_url)
      .bearer_auth(self.token())
      .send()
      .await?
      .error_for_status()?
      .json::<Vec<RecipeGetResponse>>()
      .await?;

    Ok(response)
  }
}
