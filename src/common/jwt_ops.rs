//! JWT decoding helpers (RFC 7519 base64url-aware) used to introspect Shasta tokens without verifying their signature.

use crate::error::Error;
use base64::{
  Engine,
  engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use serde_json::Value;

fn get_claims_from_jwt_token(token: &str) -> Result<Value, Error> {
  let base64_claims = token
    .split(' ')
    .nth(1)
    .unwrap_or(token)
    .split('.')
    .nth(1)
    .unwrap_or("JWT Token not valid");

  // JWTs per RFC 7519 use base64url without padding; some non-conformant
  // tokens use standard base64. Try url-safe first, then fall back.
  let claims_u8 = URL_SAFE_NO_PAD
    .decode(base64_claims)
    .or_else(|_| STANDARD.decode(base64_claims))
    .map_err(|_| Error::JwtShape("could not base64-decode JWT claims"))?;

  let claims_str = std::str::from_utf8(&claims_u8)
    .map_err(|_| Error::JwtShape("JWT claims are not valid UTF-8"))?;

  serde_json::from_str::<Value>(claims_str)
    .map_err(|_| Error::JwtShape("JWT claims are not valid JSON"))
}

/// Extract the `name` claim from a Keycloak JWT (typically the user's
/// display name). Only used by the SAT-file admin workflow, so gated
/// behind the `commands-admin` Cargo feature.
#[cfg(feature = "commands-admin")]
pub fn get_name(token: &str) -> Result<String, Error> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_name = jwt_claims.get("name").and_then(Value::as_str);

  match jwt_name {
    Some(name) => Ok(name.to_string()),
    None => Err(Error::JwtShape(
      "claim 'name' not found in JWT auth token",
    )),
  }
}

/// Extract the `preferred_username` claim from a Keycloak JWT — the
/// stable login identifier. Only used by the SAT-file admin workflow,
/// so gated behind the `commands-admin` Cargo feature.
#[cfg(feature = "commands-admin")]
pub fn get_preferred_username(token: &str) -> Result<String, Error> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_preferred_username =
    jwt_claims.get("preferred_username").and_then(Value::as_str);

  match jwt_preferred_username {
    Some(name) => Ok(name.to_string()),
    None => Err(Error::JwtShape(
      "claim 'preferred_username' not found in JWT auth token",
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
      .filter_map(|role_value| role_value.as_str().map(str::to_string))
      .collect(),
  )
}

/// This function will return true if the user is an admin, otherwise false
pub fn is_user_admin(shasta_token: &str) -> bool {
  let roles_rslt = get_roles(shasta_token);

  roles_rslt
    .is_ok_and(|roles| roles.contains(&crate::hsm::group::hacks::PA_ADMIN.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use base64::{Engine, engine::general_purpose::STANDARD};
  use serde_json::json;

  /// Build a synthetic JWT-shaped string: `<header>.<base64(claims_json)>.<sig>`.
  /// The header and signature are dummy; only the middle claims segment is read.
  fn jwt_with_claims(claims: serde_json::Value) -> String {
    let claims_b64 = STANDARD.encode(claims.to_string());
    format!("dummy-header.{}.dummy-sig", claims_b64)
  }

  // ---------- get_name ----------

  #[cfg(feature = "commands-admin")]
  #[test]
  fn get_name_returns_name_claim() {
    let token = jwt_with_claims(json!({"name": "Alice Example"}));
    assert_eq!(get_name(&token).unwrap(), "Alice Example");
  }

  #[cfg(feature = "commands-admin")]
  #[test]
  fn get_name_errors_when_name_missing() {
    let token = jwt_with_claims(json!({"sub": "abc"}));
    assert!(get_name(&token).is_err());
  }

  #[cfg(feature = "commands-admin")]
  #[test]
  fn get_name_errors_on_malformed_token() {
    // Not three dot-separated parts; the implementation tolerates 1-part
    // input and treats it as the base64 segment, but garbage base64 fails.
    assert!(get_name("not-a-jwt").is_err());
  }

  // ---------- get_preferred_username ----------

  #[cfg(feature = "commands-admin")]
  #[test]
  fn get_preferred_username_returns_claim() {
    let token = jwt_with_claims(json!({"preferred_username": "alice"}));
    assert_eq!(get_preferred_username(&token).unwrap(), "alice");
  }

  #[cfg(feature = "commands-admin")]
  #[test]
  fn get_preferred_username_errors_when_missing() {
    let token = jwt_with_claims(json!({"name": "Alice"}));
    assert!(get_preferred_username(&token).is_err());
  }

  // ---------- get_roles ----------

  #[test]
  fn get_roles_extracts_realm_access_roles() {
    let token = jwt_with_claims(json!({
      "realm_access": { "roles": ["zinal", "Compute", "pa_admin"] }
    }));
    let roles = get_roles(&token).unwrap();
    assert_eq!(roles, vec!["zinal", "Compute", "pa_admin"]);
  }

  #[test]
  fn get_roles_returns_empty_when_realm_access_missing() {
    let token = jwt_with_claims(json!({"sub": "user1"}));
    assert!(get_roles(&token).unwrap().is_empty());
  }

  #[test]
  fn get_roles_returns_empty_when_roles_missing() {
    let token = jwt_with_claims(json!({"realm_access": {}}));
    assert!(get_roles(&token).unwrap().is_empty());
  }

  #[test]
  fn get_roles_skips_non_string_role_entries() {
    let token = jwt_with_claims(json!({
      "realm_access": { "roles": ["valid", 42, true, "another"] }
    }));
    let roles = get_roles(&token).unwrap();
    assert_eq!(roles, vec!["valid", "another"]);
  }

  // ---------- is_user_admin ----------

  #[test]
  fn is_user_admin_true_when_pa_admin_role_present() {
    let token = jwt_with_claims(json!({
      "realm_access": { "roles": ["zinal", "pa_admin"] }
    }));
    assert!(is_user_admin(&token));
  }

  #[test]
  fn is_user_admin_false_when_pa_admin_role_absent() {
    let token = jwt_with_claims(json!({
      "realm_access": { "roles": ["zinal", "Compute"] }
    }));
    assert!(!is_user_admin(&token));
  }

  #[test]
  fn is_user_admin_false_when_token_malformed() {
    // get_roles returns Err, so is_user_admin returns false (not a panic).
    assert!(!is_user_admin("garbage"));
  }

  // ---------- bearer-style "Bearer <jwt>" prefix handling ----------

  #[cfg(feature = "commands-admin")]
  #[test]
  fn get_name_strips_bearer_prefix() {
    // The implementation calls .split(' ').nth(1) first, then falls back to
    // the original token. So "Bearer <jwt>" should also work.
    let claims = json!({"name": "Alice"});
    let claims_b64 = STANDARD.encode(claims.to_string());
    let jwt = format!("dummy-header.{}.dummy-sig", claims_b64);
    let bearer = format!("Bearer {}", jwt);
    assert_eq!(get_name(&bearer).unwrap(), "Alice");
  }
}
