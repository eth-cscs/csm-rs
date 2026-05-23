use manta_backend_dispatcher::types::ims::{
  Image as FrontEndImage,
  ImsImageRecord2Update as FrontEndImsImageRecord2Update, Link as FrontEndLink,
  PatchImage as FrontEndPatchImage, PatchMetadata as FrontEndPatchMetadata,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ImsImageRecord2Update {
  pub link: Link,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arch: Option<String>,
}

impl From<FrontEndImsImageRecord2Update> for ImsImageRecord2Update {
  fn from(
    frontend_ims_image_record2_update: FrontEndImsImageRecord2Update,
  ) -> Self {
    Self {
      link: frontend_ims_image_record2_update.link.into(),
      arch: frontend_ims_image_record2_update.arch,
    }
  }
}

impl From<ImsImageRecord2Update> for FrontEndImsImageRecord2Update {
  fn from(val: ImsImageRecord2Update) -> Self {
    FrontEndImsImageRecord2Update {
      link: val.link.into(),
      arch: val.arch,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Link {
  pub path: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub etag: Option<String>,
  pub r#type: String,
}

impl From<FrontEndLink> for Link {
  fn from(frontend_link: FrontEndLink) -> Self {
    Self {
      path: frontend_link.path,
      etag: frontend_link.etag,
      r#type: frontend_link.r#type,
    }
  }
}

impl From<Link> for FrontEndLink {
  fn from(val: Link) -> Self {
    FrontEndLink {
      path: val.path,
      etag: val.etag,
      r#type: val.r#type,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Image {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created: Option<String>,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub link: Option<Link>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arch: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<HashMap<String, String>>,
}

impl From<FrontEndImage> for Image {
  fn from(frontend_image: FrontEndImage) -> Self {
    Self {
      id: frontend_image.id,
      created: frontend_image.created,
      name: frontend_image.name,
      link: frontend_image.link.map(|link| link.into()),
      arch: frontend_image.arch,
      metadata: frontend_image.metadata,
    }
  }
}

impl From<Image> for FrontEndImage {
  fn from(val: Image) -> Self {
    FrontEndImage {
      id: val.id,
      created: val.created,
      name: val.name,
      link: val.link.map(|link| link.into()),
      arch: val.arch,
      metadata: val.metadata,
    }
  }
}

pub struct PatchMetadata {
  pub operation: String,
  pub key: String,
  pub value: String,
}

impl From<FrontEndPatchMetadata> for PatchMetadata {
  fn from(frontend_patch_metadata: FrontEndPatchMetadata) -> Self {
    Self {
      operation: frontend_patch_metadata.operation,
      key: frontend_patch_metadata.key,
      value: frontend_patch_metadata.value,
    }
  }
}

impl From<PatchMetadata> for FrontEndPatchMetadata {
  fn from(val: PatchMetadata) -> Self {
    FrontEndPatchMetadata {
      operation: val.operation,
      key: val.key,
      value: val.value,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PatchImage {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub link: Option<Link>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arch: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<HashMap<String, String>>,
}

impl From<FrontEndPatchImage> for PatchImage {
  fn from(frontend_patch_image: FrontEndPatchImage) -> Self {
    Self {
      link: frontend_patch_image.link.map(|link| link.into()),
      arch: frontend_patch_image.arch,
      metadata: frontend_patch_image
        .metadata,
    }
  }
}

impl From<PatchImage> for FrontEndPatchImage {
  fn from(val: PatchImage) -> Self {
    FrontEndPatchImage {
      link: val.link.map(|link| link.into()),
      arch: val.arch,
      metadata: val.metadata,
    }
  }
}

impl From<Image> for PatchImage {
  fn from(patch_image: Image) -> Self {
    Self {
      link: patch_image.link,
      arch: patch_image.arch,
      metadata: patch_image.metadata,
    }
  }
}

impl From<PatchImage> for Image {
  fn from(val: PatchImage) -> Self {
    Image {
      id: None,
      created: None,
      name: String::default(),
      link: val.link,
      arch: val.arch,
      metadata: val.metadata,
    }
  }
}
