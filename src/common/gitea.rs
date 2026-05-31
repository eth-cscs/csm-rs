//! Small client for the embedded CSM Gitea instance used by CFS configuration layers.

/// HTTP helpers for the embedded CSM Gitea instance.
pub mod http_client {

  use crate::{common::http, error::Error};
  use serde_json::Value;

  /// Extract the repo name (the path after `vcs/cray/`) from a Gitea
  /// URL, trimming the trailing `.git` if present.
  pub fn get_repo_name_from_url(repo_url: &str) -> Result<String, Error> {
    if repo_url.starts_with("https://api-gw-service-nmn.local") {
      let gitea_internal_base_url =
        "https://api-gw-service-nmn.local/vcs/cray/";

      Ok(
        repo_url
          .trim_start_matches(gitea_internal_base_url)
          .trim_end_matches(".git")
          .to_string(),
      )
    } else if repo_url.starts_with("https://vcs.cmn.alps.cscs.ch") {
      let gitea_external_base_url = "https://vcs.cmn.alps.cscs.ch/vcs/cray/";

      Ok(
        repo_url
          .trim_start_matches(gitea_external_base_url)
          .trim_end_matches(".git")
          .to_string(),
      )
    } else if repo_url.starts_with("https://api.cmn.alps.cscs.ch") {
      let gitea_external_base_url = "https://api.cmn.alps.cscs.ch/vcs/cray/";

      Ok(
        repo_url
          .trim_start_matches(gitea_external_base_url)
          .trim_end_matches(".git")
          .to_string(),
      )
    } else {
      Err(Error::Message(
        "repo url provided does not match gitea internal or external URL"
          .to_string(),
      ))
    }
  }

  /// Get all refs for a repository
  /// Used when getting repo details
  pub async fn get_all_refs_from_repo_url(
    gitea_base_url: &str,
    gitea_token: &str,
    repo_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
  ) -> Result<Vec<Value>, Error> {
    let repo_name = get_repo_name_from_url(repo_url)?;

    get_all_refs(
      gitea_base_url,
      gitea_token,
      &repo_name,
      shasta_root_cert,
      socks5_proxy,
    )
    .await
  }

  /// Get all refs for a repository
  /// Used when getting repo details
  pub async fn get_all_refs(
    gitea_base_url: &str,
    gitea_token: &str,
    repo_name: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
  ) -> Result<Vec<Value>, Error> {
    let client = http::build_client(shasta_root_cert, socks5_proxy)?;
    let api_url = format!(
      "{}/api/v1/repos/cray/{}/git/refs",
      gitea_base_url, repo_name
    );

    log::debug!("Get refs in gitea using through API call: {}", api_url);

    let response = client
      .get(api_url)
      .header("Authorization", format!("token {}", gitea_token))
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Get most commit id (sha) pointed by a branch
  pub async fn get_commit_pointed_by_branch(
    gitea_base_url: &str,
    gitea_token: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    repo_url: &str,
    branch_name: &str,
  ) -> Result<String, Error> {
    let all_ref_vec = get_all_refs_from_repo_url(
      gitea_base_url,
      gitea_token,
      repo_url,
      shasta_root_cert,
      socks5_proxy,
    )
    .await?;

    let want = format!("refs/heads/{}", branch_name);
    let ref_details_opt = all_ref_vec.into_iter().find(|ref_details| {
      ref_details
        .get("ref")
        .and_then(Value::as_str)
        .is_some_and(|r| r == want)
    });

    match ref_details_opt {
      Some(ref_details) => ref_details
        .get("object")
        .and_then(|object| object.get("sha"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
          Error::Message(
            "Gitea response: ref object is missing 'sha'".to_string(),
          )
        }),
      None => Err(Error::Message("SHA for branch not found".to_string())),
    }
  }

  /// Returns the commit id (sha) related to a tag name
  /// Used to translate CFS configuration layer tag name into commit id values when processing
  /// SAT files
  pub async fn get_tag_details(
    repo_url: &str,
    tag: &str,
    gitea_token: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    site_name: &str,
  ) -> Result<Value, Error> {
    let gitea_internal_base_url = "https://api-gw-service-nmn.local/vcs/";
    let gitea_external_base_url =
      format!("https://api.cmn.{}.cscs.ch/vcs/", site_name);

    let gitea_api_base_url = gitea_internal_base_url.to_owned() + "api/v1";

    let repo_name = repo_url
      .trim_start_matches(gitea_internal_base_url)
      .trim_end_matches(".git");
    let repo_name = repo_name
      .trim_start_matches(&gitea_external_base_url)
      .trim_end_matches(".git");

    let client = http::build_client(shasta_root_cert, socks5_proxy)?;
    let api_url =
      format!("{}/repos/{}/tags/{}", gitea_api_base_url, repo_name, tag);

    log::debug!("Request to {}", api_url);

    let response = client
      .get(api_url)
      .header("Authorization", format!("token {}", gitea_token))
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Returns the commit id (sha) related to a tag name
  /// Used to translate CFS configuration layer tag name into commit id values when processing
  /// SAT files
  pub async fn get_commit_from_tag(
    gitea_api_tag_url: &str,
    tag: &str,
    gitea_token: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    site_name: &str,
  ) -> Result<Value, Error> {
    let external_vcs_base_url = format!(
      "https://vcs.cmn.{}.cscs.ch/vcs/api/v1/repos/cray/",
      site_name
    );
    let repo_name: &str = gitea_api_tag_url
      .trim_start_matches(&external_vcs_base_url)
      .split('/')
      .next()
      .ok_or_else(|| Error::Message("Invalid repo URL".to_string()))?;

    let api_url = format!(
      "https://api.cmn.{}.cscs.ch/vcs/api/v1/repos/cray/{}/tags/{}",
      site_name, repo_name, tag
    );

    let client = http::build_client(shasta_root_cert, socks5_proxy)?;

    log::debug!("Request to {}", api_url);

    let response = client
      .get(api_url)
      .header("Authorization", format!("token {}", gitea_token))
      .send()
      .await
      .map_err(Error::NetError)?;

    http::handle_json_or_text_response(response).await
  }

  /// Fetch commit details for `commitid` from the site's external Gitea
  /// URL (`api.cmn.<site>.cscs.ch/vcs/`).
  ///
  /// Note: `repo_name` must NOT contain the group prefix (e.g. CSCS
  /// gitlab has `alps/csm-config/template-management` whereas Gitea
  /// expects just `template-management`).
  pub async fn get_commit_details_from_external_url(
    // repo_url: &str,
    repo_name: &str,
    commitid: &str,
    gitea_token: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
    site_name: &str,
  ) -> Result<Value, crate::error::Error> {
    let gitea_external_base_url =
      format!("https://api.cmn.{}.cscs.ch/vcs/", site_name);

    get_commit_details(
      &gitea_external_base_url,
      repo_name,
      commitid,
      gitea_token,
      shasta_root_cert,
      socks5_proxy,
    )
    .await
  }

  /// Fetch commit details for `commitid` from an arbitrary Gitea base
  /// URL. Lower-level companion to
  /// [`get_commit_details_from_external_url`].
  pub async fn get_commit_details(
    gitea_base_url: &str,
    repo_name: &str,
    commitid: &str,
    gitea_token: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
  ) -> Result<Value, crate::error::Error> {
    let client = http::build_client(shasta_root_cert, socks5_proxy)?;
    let api_url = format!(
      "{}api/v1/repos/{}/git/commits/{}",
      gitea_base_url, repo_name, commitid
    );

    log::info!("url to get commit details: {}", api_url);

    let response = client
      .get(api_url)
      .header("Authorization", format!("token {}", gitea_token))
      .send()
      .await?;

    if response.status().is_success() {
      response.json().await.map_err(Error::NetError)
    } else {
      // Bespoke: wraps the text body in a synthetic JSON object so callers
      // can match on `CsmError`. status=0 marks "no real HTTP status from
      // CSM" — this is a gitea error smuggled through CsmError; a future
      // cleanup should introduce a proper GiteaError variant.
      let status = response.status().as_u16();
      let payload = response.text().await?;
      Err(Error::csm_from_response(
        status,
        serde_json::json!({ "detail": payload }),
      ))
    }
  }

  /// Return the most-recent commit in `repo_name` sorted by committer
  /// date.
  pub async fn get_last_commit_from_repo_name(
    gitea_api_base_url: &str,
    repo_name: &str,
    gitea_token: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
  ) -> core::result::Result<Value, Error> {
    let repo_url =
      gitea_api_base_url.to_owned() + "/api/v1/repos" + repo_name + "/commits";

    let client = http::build_client(shasta_root_cert, socks5_proxy)?;

    let mut resp: Vec<Value> = client
      .get(repo_url)
      .header("Authorization", format!("token {}", gitea_token))
      .send()
      .await?
      .error_for_status()?
      .json()
      .await?;

    resp.sort_by(|a, b| {
      a["commit"]["committer"]["date"]
        .to_string()
        .cmp(&b["commit"]["committer"]["date"].to_string())
    });

    resp
      .last()
      .ok_or_else(|| Error::Message("No commits found".to_string()))
      .cloned()
  }

  /// Return the most-recent commit in a repo identified by URL —
  /// resolves the URL to a repo name via [`get_repo_name_from_url`]
  /// before delegating to [`get_last_commit_from_repo_name`].
  pub async fn get_last_commit_from_url(
    gitea_api_base_url: &str,
    repo_url: &str,
    gitea_token: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
  ) -> core::result::Result<Value, Error> {
    let repo_name = repo_url
      .trim_start_matches("https://api-gw-service-nmn.local/vcs/")
      .trim_end_matches(".git");

    get_last_commit_from_repo_name(
      gitea_api_base_url,
      repo_name,
      gitea_token,
      shasta_root_cert,
      socks5_proxy,
    )
    .await
  }
}
