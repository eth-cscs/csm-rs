//! Bidirectional `From` impls between csm-rs's IMS image types and the
//! dispatcher's mirrors. Gated behind the `manta-dispatcher` Cargo
//! feature so users not on Manta don't pull the dispatcher dep.

use manta_backend_dispatcher::types::ims::{
  Image as FrontEndImage,
  ImsImageRecord2Update as FrontEndImsImageRecord2Update, Link as FrontEndLink,
  PatchImage as FrontEndPatchImage, PatchMetadata as FrontEndPatchMetadata,
};

use super::types::{Image, ImsImageRecord2Update, Link, PatchImage, PatchMetadata};

// Provenance metadata keys stamped by the apply_sat_file flow. Keep in
// sync with `commands::i_apply_sat_file::utils::images::META_*` —
// duplicated here because that module is gated on `commands-admin`
// while this conversion is gated on `manta-dispatcher`.
const META_BASE: &str = "manta.image_session.base";
const META_GROUPS: &str = "manta.image_session.groups";
const META_CONFIG: &str = "manta.image_session.configuration";

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

impl From<FrontEndImage> for Image {
  fn from(frontend_image: FrontEndImage) -> Self {
    Self {
      id: frontend_image.id,
      created: frontend_image.created,
      name: frontend_image.name,
      link: frontend_image.link.map(std::convert::Into::into),
      arch: frontend_image.arch,
      metadata: frontend_image.metadata,
    }
  }
}

impl From<Image> for FrontEndImage {
  fn from(val: Image) -> Self {
    let base = val
      .metadata
      .as_ref()
      .and_then(|m| m.get(META_BASE).cloned());
    let configuration = val
      .metadata
      .as_ref()
      .and_then(|m| m.get(META_CONFIG).cloned());
    let groups = val.metadata.as_ref().and_then(|m| {
      m.get(META_GROUPS)
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
    });
    FrontEndImage {
      id: val.id,
      created: val.created,
      name: val.name,
      link: val.link.map(std::convert::Into::into),
      arch: val.arch,
      metadata: val.metadata,
      groups,
      base,
      configuration,
    }
  }
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

impl From<FrontEndPatchImage> for PatchImage {
  fn from(frontend_patch_image: FrontEndPatchImage) -> Self {
    Self {
      link: frontend_patch_image.link.map(std::convert::Into::into),
      arch: frontend_patch_image.arch,
      metadata: frontend_patch_image.metadata,
    }
  }
}

impl From<PatchImage> for FrontEndPatchImage {
  fn from(val: PatchImage) -> Self {
    FrontEndPatchImage {
      link: val.link.map(std::convert::Into::into),
      arch: val.arch,
      metadata: val.metadata,
    }
  }
}
