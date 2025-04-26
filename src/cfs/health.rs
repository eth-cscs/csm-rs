use std::time::Duration;

use crate::error::Error;

pub async fn test_connectivity_to_backend(shasta_base_url: &str) -> Result<(), Error> {
    let client;

    let client_builder = reqwest::Client::builder().connect_timeout(Duration::new(3, 0));

    // Build client
    client = client_builder.build().unwrap();

    let api_url = shasta_base_url.to_owned() + "/cfs/healthz";

    log::info!("Validate CSM token against {}", api_url);

    client
        .get(api_url)
        .send()
        .await?
        .error_for_status()
        .map(|_| ())
        .map_err(|e| Error::Message(e.to_string()))
}
