use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    migrate_backup::MigrateBackupTrait, migrate_restore::MigrateRestoreTrait,
  },
};

use super::Csm;

impl MigrateRestoreTrait for Csm {
  async fn migrate_restore(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file: Option<&str>,
    cfs_file: Option<&str>,
    hsm_file: Option<&str>,
    ims_file: Option<&str>,
    image_dir: Option<&str>,
    overwrite_group: bool,
    overwrite_configuration: bool,
    overwrite_image: bool,
    overwrite_template: bool,
  ) -> Result<(), Error> {
    crate::commands::migrate_restore::exec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      bos_file,
      cfs_file,
      hsm_file,
      ims_file,
      image_dir,
      overwrite_group,
      overwrite_configuration,
      overwrite_image,
      overwrite_template,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }
}

impl MigrateBackupTrait for Csm {
  async fn migrate_backup(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> Result<(), Error> {
    crate::commands::migrate_backup::exec(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      bos,
      destination,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))
  }
}
