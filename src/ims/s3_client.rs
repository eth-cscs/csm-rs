use aws_config::SdkConfig;
use hyper::client::HttpConnector;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde_json::Value;

use anyhow::Result;
use aws_sdk_s3::{Client, primitives::ByteStream};
use indicatif::{ProgressBar, ProgressStyle};

use crate::error::Error;

pub const BAR_FORMAT: &str = "[{elapsed_precise}] {bar:40.cyan/blue} ({bytes_per_sec}) {bytes:>7}/{total_bytes:7} {msg} [ETA {eta}]";
// Get a token for S3 and return the result
// If something breaks, return an error
pub async fn s3_auth(
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  socks5_proxy: Option<&str>,
) -> Result<Value, Error> {
  // STS
  let client =
    crate::common::http::build_client(shasta_root_cert, socks5_proxy)?;

  let api_url = shasta_base_url.to_owned() + "/sts/token";

  let resp = client
    .put(api_url)
    .bearer_auth(shasta_token)
    .send()
    .await?
    .error_for_status()
    .map_err(|e| {
      Error::Message(format!(
        "ERROR - could not authenticate to S3 server. Reason:\n{}",
        e
      ))
    })?;

  let sts_value = resp.json::<serde_json::Value>().await?;

  log::debug!("-- STS Token retrieved --");
  log::debug!("Debug - STS token:\n{:#?}", sts_value);

  // SET AUTH ENVS
  let session_token = sts_value
    .pointer("/Credentials/SessionToken")
    .and_then(Value::as_str)
    .ok_or_else(|| {
      Error::Message("Missing SessionToken in STS response".to_string())
    })?;

  let access_key_id = sts_value
    .pointer("/Credentials/AccessKeyId")
    .and_then(Value::as_str)
    .ok_or_else(|| {
      Error::Message("Missing AccessKeyId in STS response".to_string())
    })?;

  let secret_access_key = sts_value
    .pointer("/Credentials/SecretAccessKey")
    .and_then(Value::as_str)
    .ok_or_else(|| {
      Error::Message("Missing SecretAccessKey in STS response".to_string())
    })?;

  unsafe {
    std::env::set_var("AWS_SESSION_TOKEN", session_token);
    std::env::set_var("AWS_ACCESS_KEY_ID", access_key_id);
    std::env::set_var("AWS_SECRET_ACCESS_KEY", secret_access_key);
  }

  Ok(sts_value)
}

async fn setup_client(
  sts_value: &Value,
  socks5_proxy: Option<&str>,
) -> Result<Client, Error> {
  use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;

  // Default provider fallback to us-east-1 since CSM doesn't use the concept of regions
  let region_provider =
    aws_config::meta::region::RegionProviderChain::default_provider()
      .or_else("us-east-1");
  let config: SdkConfig;

  if let Some(socks5_env) = socks5_proxy {
    log::debug!("SOCKS5 enabled");

    let mut http_connector: HttpConnector = hyper::client::HttpConnector::new();
    http_connector.enforce_http(false);

    let socks_http_connector = hyper_socks2::SocksConnector {
      proxy_addr: hyper::Uri::try_from(socks5_env)
        .map_err(|e| Error::Message(e.to_string()))?, // scheme is required by HttpConnector
      auth: None,
      connector: http_connector.clone(),
    };

    let http_client = HyperClientBuilder::new().build(socks_http_connector);

    config = aws_config::from_env()
      .region(region_provider)
      .http_client(http_client)
      .endpoint_url(
        sts_value
          .get("Credentials")
          .and_then(|credentials| credentials.get("EndpointURL"))
          .and_then(Value::as_str)
          .ok_or_else(|| {
            Error::Message("Missing EndpointURL in STS response".to_string())
          })?,
      )
      .app_name(aws_config::AppName::new("manta").map_err(|e| {
        Error::Message(format!("Error setting app name: {}", e))
      })?)
      // .no_credentials()
      .load()
      .await;
  } else {
    config = aws_config::from_env()
      .region(region_provider)
      .endpoint_url(
        sts_value
          .get("Credentials")
          .and_then(|credentials| credentials.get("EndpointURL"))
          .and_then(Value::as_str)
          .ok_or_else(|| {
            Error::Message("Missing EndpointURL in STS response".to_string())
          })?,
      )
      .app_name(aws_config::AppName::new("manta").map_err(|e| {
        Error::Message(format!("Error setting app name: {}", e))
      })?)
      .load()
      .await;
  }

  let client = aws_sdk_s3::Client::from_conf(
    aws_sdk_s3::Client::new(&config)
      .config()
      .to_builder()
      .force_path_style(true)
      .build(),
  );

  Ok(client)
}
/// Gets the size of a given object in S3
/// path of the object: s3://bucket/key
/// returns i64 or error
pub async fn s3_get_object_size(
  sts_value: &Value,
  socks5_proxy: Option<&str>,
  key: &str,
  bucket: &str,
) -> Result<i64, Error> {
  let client = setup_client(sts_value, socks5_proxy).await?;

  match client.get_object().bucket(bucket).key(key).send().await {
    Ok(object) => Ok(object.content_length().ok_or_else(|| {
      Error::Message("Error, content length not found".to_string())
    })?),
    Err(e) => Err(Error::Message(format!(
      "Error, unable to get object from s3. Error msg: {}",
      e
    ))),
  }
}

/// Download an object from S3 to a local directory.
///
/// Streams the object body to disk with a progress bar. Returns the
/// full path of the downloaded file.
///
/// # Arguments
///
/// - `sts_value` — temporary S3 credentials obtained from STS via
///   `s3_auth()`.
/// - `object_path` — path within `bucket`, e.g.
///   `392o1h-1-234-w1/manifest.json`.
/// - `bucket` — bucket containing the object.
/// - `destination_path` — local directory to write into; the file is
///   placed at `destination_path/<basename(object_path)>`. Created if
///   missing.
///
/// # Errors
///
/// Returns [`Error`] if the local directory or file cannot be created,
/// or if the S3 GET fails.
pub async fn s3_download_object(
  sts_value: &Value,
  socks5_proxy: Option<&str>,
  object_path: &str,
  bucket: &str,
  destination_path: &str,
) -> Result<String, Error> {
  let client = setup_client(sts_value, socks5_proxy).await?;

  let filename = Path::new(object_path).file_name().ok_or_else(|| {
    Error::Message(format!(
      "Error getting filename from S3 object path: {}",
      object_path
    ))
  })?;

  let file_path = Path::new(destination_path).join(filename);
  log::debug!("Create directory '{}'", destination_path);

  std::fs::create_dir_all(destination_path).map_err(|e| {
    Error::Message(format!(
      "Error creating directory {}: {}",
      destination_path, e
    ))
  })?;

  log::debug!("Created directory '{}' successfully", destination_path);

  let mut file = File::create(&file_path).map_err(|e| {
    Error::Message(format!(
      "Error creating file {}: {}",
      &file_path.to_string_lossy(),
      e
    ))
  })?;

  log::debug!(
    "Created file '{}' successfully",
    &file_path.to_string_lossy()
  );

  let mut object = client
    .get_object()
    .bucket(bucket)
    .key(object_path)
    .send()
    .await
    .map_err(|e| {
      Error::Message(format!(
        "ERROR - could not download S3 object.\nReason:\n{}",
        e,
      ))
    })?;

  let bar_size = object.content_length().ok_or_else(|| {
    Error::Message("ERROR - could not get S3 object size.".to_string())
  })?;

  let bar = ProgressBar::new(bar_size as u64);
  bar.set_style(ProgressStyle::with_template(BAR_FORMAT).map_err(|e| {
    Error::Message(format!(
      "ERROR - Could not create progress bar.\nReason:\n{}",
      e
    ))
  })?);

  while let Some(bytes) = object.body.try_next().await.map_err(|e| {
    Error::Message(format!(
      "ERROR - Could not finish s3 object download.\nReason:\n{}",
      e
    ))
  })? {
    let bytes = file.write(&bytes)?;
    bar.inc(bytes as u64);
  }

  bar.finish();

  Ok(file_path.to_string_lossy().to_string())
}

/// Upload a local file to S3 in a single request.
///
/// Returns the ETag of the uploaded object.
///
/// # Arguments
///
/// - `sts_value` — temporary S3 credentials obtained from STS via
///   `s3_auth()`.
/// - `object_path` — path within `bucket` to upload to.
/// - `bucket` — destination bucket.
/// - `file_path` — local file to upload.
///
/// For large files prefer [`s3_multipart_upload_object`].
pub async fn s3_upload_object(
  sts_value: &Value,
  socks5_proxy: Option<&str>,
  object_path: &str,
  bucket: &str,
  file_path: &str,
) -> Result<String, Error> {
  let client = setup_client(sts_value, socks5_proxy).await?;

  let body = ByteStream::from_path(Path::new(&file_path)).await?;

  let put_s3_object = client
    .put_object()
    .bucket(bucket)
    .key(object_path)
    .body(body)
    .send()
    .await
    .map_err(|e| {
      Error::Message(format!(
        "ERROR - could not upload S3 object.\nReason:\n{}",
        e
      ))
    })?;

  put_s3_object.e_tag.ok_or_else(|| {
    Error::Message("ERROR - could not get ETag from upload.".to_string())
  })
}

/// Delete an object from S3.
///
/// # Arguments
///
/// - `sts_value` — temporary S3 credentials obtained from STS via
///   `s3_auth()`.
/// - `object_path` — path within `bucket` to delete.
/// - `bucket` — source bucket.
pub async fn s3_remove_object(
  sts_value: &Value,
  socks5_proxy: Option<&str>,
  object_path: &str,
  bucket: &str,
) -> Result<String, Error> {
  let client = setup_client(sts_value, socks5_proxy).await?;

  match client
    .delete_object()
    .bucket(bucket)
    .key(object_path)
    .send()
    .await
  {
    Ok(_file) => {
      log::debug!("Cleaned file '{}' successfully", &object_path);
      Ok(String::from("client"))
    }
    Err(error) => Err(Error::Message(format!(
      "Error cleaning file {}: {}",
      &object_path, error
    ))),
  }
}

/// Upload a local file to S3 using the multipart-upload protocol.
///
/// Splits `file_path` into chunks and uploads them with a progress bar.
/// Use this for files that exceed the single-PUT limit; for small files
/// [`s3_upload_object`] is simpler.
///
/// # Arguments
///
/// - `sts_value` — temporary S3 credentials obtained from STS via
///   `s3_auth()`.
/// - `object_path` — path within `bucket` to upload to.
/// - `bucket` — destination bucket.
/// - `file_path` — local file to upload.
pub async fn s3_multipart_upload_object(
  sts_value: &Value,
  socks5_proxy: Option<&str>,
  object_path: &str,
  bucket: &str,
  file_path: &str,
) -> Result<String, Error> {
  use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput;
  use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
  use aws_smithy_types::byte_stream::Length;

  let client = setup_client(sts_value, socks5_proxy).await?;

  //In bytes, minimum chunk size of 5MB. Increase CHUNK_SIZE to send larger chunks.
  const CHUNK_SIZE: u64 = 1024 * 1024 * 5;
  const MAX_CHUNKS: u64 = 10000;

  // create multipart upload
  let multipart_upload_res: CreateMultipartUploadOutput = client
    .create_multipart_upload()
    .bucket(bucket)
    .key(object_path)
    .send()
    .await
    .map_err(|e| {
      Error::Message(format!(
        "ERROR - Could not create multipart object.\nReason:\n{}",
        e
      ))
    })?;

  let upload_id = multipart_upload_res.upload_id().ok_or_else(|| {
    Error::Message("ERROR - Could not get upload ID.".to_string())
  })?;

  // Get details of the upload, this is needed because multipart uploads
  // are tricky and have a minimum chunk size of 5MB
  let path = Path::new(&file_path);
  let file_size = std::fs::metadata(path)
    .map_err(|e| {
      Error::Message(format!(
        "ERROR - Could not get file size from '{}'.\nReason\n{}",
        file_path, e
      ))
    })?
    .len();

  let mut chunk_count = (file_size / CHUNK_SIZE) + 1;
  let mut size_of_last_chunk = file_size % CHUNK_SIZE;
  if size_of_last_chunk == 0 {
    size_of_last_chunk = CHUNK_SIZE;
    chunk_count -= 1;
  }

  let bar = ProgressBar::new(file_size);
  bar.set_style(ProgressStyle::with_template(BAR_FORMAT).map_err(|e| {
    Error::Message(format!(
      "ERROR - Could not create progress bar.\nReason:\n{}",
      e
    ))
  })?);

  if file_size == 0 {
    return Err(Error::Message("Bad file size.".to_string()));
  }
  if chunk_count > MAX_CHUNKS {
    return Err(Error::Message(
      "Too many chunks! Try increasing your chunk size.".to_string(),
    ));
  }

  let mut upload_parts: Vec<CompletedPart> = Vec::new();

  for chunk_index in 0..chunk_count {
    let this_chunk = if chunk_count - 1 == chunk_index {
      size_of_last_chunk
    } else {
      CHUNK_SIZE
    };
    let stream = ByteStream::read_from()
      .path(path)
      .offset(chunk_index * CHUNK_SIZE)
      .length(Length::Exact(this_chunk))
      .build()
      .await
      .map_err(|e| {
        Error::Message(format!(
          "ERROR - Could not read file '{}'.\nReason:\n{}",
          path.display(),
          e
        ))
      })?;

    //Chunk index needs to start at 0, but part numbers start at 1.
    let part_number = (chunk_index as i32) + 1;

    let upload_part_res = client
      .upload_part()
      .key(object_path)
      .bucket(bucket)
      .upload_id(upload_id)
      .body(stream)
      .part_number(part_number)
      .send()
      .await
      .map_err(|e| {
        Error::Message(format!(
          "ERROR - could not upload to S3.\nReason:\n{}",
          e
        ))
      })?;

    upload_parts.push(
      CompletedPart::builder()
        .e_tag(upload_part_res.e_tag.ok_or_else(|| {
          Error::Message(
            "ERROR - could not get ETag from upload part.".to_string(),
          )
        })?)
        .part_number(part_number)
        .build(),
    );
    bar.inc(this_chunk);
  }
  // complete the multipart upload
  let completed_multipart_upload: CompletedMultipartUpload =
    CompletedMultipartUpload::builder()
      .set_parts(Some(upload_parts))
      .build();

  let complete_multipart_upload_res = client
    .complete_multipart_upload()
    .bucket(bucket)
    .key(object_path)
    .multipart_upload(completed_multipart_upload)
    .upload_id(upload_id)
    .send()
    .await
    .map_err(|e| {
      Error::Message(format!("ERROR - could not upload to S3.\nReason:\n{}", e))
    })?;

  bar.finish();

  complete_multipart_upload_res.e_tag.ok_or_else(|| {
    Error::Message("ERROR - could not get ETag from upload.".to_string())
  })
}
