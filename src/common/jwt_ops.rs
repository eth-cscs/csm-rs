use crate::error::Error;
use base64::decode;
use serde_json::Value;

/* // FIXME: replace Error to my own one
#[deprecated(
    note = "Please, avoid using this function, if you need to get the list of HSM groups available to the user, then use `mesa::common::jwt_ops::get_hsm_name_available` because this function has the hack removing system wide hsm group names like alps, aplsm, alpse, etc. If you want the preffereed username, then use `mesa::common::jwt_ops::`mesa::common::jwt_ops::get_preferred_username"
)] */
fn get_claims_from_jwt_token(token: &str) -> Result<Value, Error> {
  let base64_claims = token
    .split(' ')
    .nth(1)
    .unwrap_or(token)
    .split('.')
    .nth(1)
    .unwrap_or("JWT Token not valid");

  let claims_u8 = decode(base64_claims).map_err(|e| {
    Error::Message(format!(
      "ERROR - could not get claims in JWT token. Reason:\n{}",
      e
    ))
  })?;

  let claims_str = std::str::from_utf8(&claims_u8).map_err(|_| {
    Error::Message("ERROR - could not convert JWT claims to string".to_string())
  })?;

  serde_json::from_str::<Value>(claims_str).map_err(|_| {
    Error::Message(
      "ERROR - could not convert JWT claims to a JSON object".to_string(),
    )
  })
}

pub fn get_name(token: &str) -> Result<String, Error> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_name = jwt_claims.get("name").and_then(Value::as_str);

  match jwt_name {
    Some(name) => Ok(name.to_string()),
    None => Err(Error::Message(
      "ERROR - claim 'name' not found in JWT auth token".to_string(),
    )),
  }
}

pub fn get_preferred_username(token: &str) -> Result<String, Error> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_preferred_username =
    jwt_claims.get("preferred_username").and_then(Value::as_str);

  match jwt_preferred_username {
    Some(name) => Ok(name.to_string()),
    None => Err(Error::Message(
      "ERROR - claim 'name' not found in JWT auth token".to_string(),
    )),
  }
}

/// Returns the list of available HSM groups in JWT user token. The list is filtered and system HSM
/// groups (eg alps, alpsm, alpse, etc)
pub fn get_roles(token: &str) -> Result<Vec<String>, Error> {
  // If JWT does not have `/realm_access/roles` claim, then we will assume, user is admin
  Ok(
    get_claims_from_jwt_token(token)?
      .pointer("/realm_access/roles")
      .unwrap_or(&serde_json::json!([]))
      .as_array()
      .cloned()
      .unwrap_or_default()
      .iter()
      .map(|role_value| role_value.as_str().map(str::to_string).unwrap())
      .collect(),
  )
}

/// This function will return true if the user is an admin, otherwise false
pub fn is_user_admin(shasta_token: &str) -> bool {
  let roles_rslt = get_roles(shasta_token);

  roles_rslt.is_ok_and(|roles| roles.contains(&"pa_admin".to_string()))
}
