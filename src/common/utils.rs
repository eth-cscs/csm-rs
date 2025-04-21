


/* pub async fn get_configurations_sessions_bos_sessiontemplates_images(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    get_cfs_configuration: bool,
    get_cfs_session: bool,
    get_bos_sessiontemplate: bool,
    get_ims_image: bool,
) -> Result<
    (
        Option<Vec<CfsConfigurationResponse>>,
        Option<Vec<CfsSessionGetResponse>>,
        Option<Vec<BosSessionTemplate>>,
        Option<Vec<Image>>,
    ),
    Error,
> {
    let start = Instant::now();

    let handle_cfs_configuration_opt = if get_cfs_configuration {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            cfs::configuration::http_client::v2::get(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
                None,
            )
            .await
            // .unwrap()
        }))
    } else {
        None
    };

    let handle_cfs_session_opt = if get_cfs_session {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            cfs::session::get_and_sort(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            // .unwrap()
        }))
    } else {
        None
    };

    let handle_bos_sessiontemplate_opt = if get_bos_sessiontemplate {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            bos::template::http_client::v2::get_all(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
            )
            .await
            // .unwrap()
        }))
    } else {
        None
    };

    let handle_ims_image_opt = if get_ims_image {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            ims::image::http_client::get_all(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
            )
            .await
            // .unwrap()
        }))
    } else {
        None
    };

    /* let cfs_configuration_vec = if let Some(handle) = handle_cfs_configuration_opt {
        Some(handle.await.unwrap())
    } else {
        None
    }; */
    let cfs_configuration_vec = if let Some(handle) = handle_cfs_configuration_opt {
        match handle.await.unwrap() {
            Ok(cfs_configuration_vec) => Some(cfs_configuration_vec),
            Err(e) => return Err(e),
        }
    } else {
        None
    };

    /* let cfs_session_vec = if let Some(handle) = handle_cfs_session_opt {
        Some(handle.await.unwrap())
    } else {
        None
    }; */
    let cfs_session_vec = if let Some(handle) = handle_cfs_session_opt {
        match handle.await.unwrap() {
            Ok(cfs_session_vec) => Some(cfs_session_vec),
            Err(e) => return Err(e),
        }
    } else {
        None
    };

    /* let bos_sessiontemplate_vec = if let Some(handle) = handle_bos_sessiontemplate_opt {
        Some(handle.await.unwrap())
    } else {
        None
    }; */
    let bos_sessiontemplate_vec = if let Some(handle) = handle_bos_sessiontemplate_opt {
        match handle.await.unwrap() {
            Ok(bos_sessiontemplate_vec) => Some(bos_sessiontemplate_vec),
            Err(e) => return Err(e),
        }
    } else {
        None
    };

    /* let ims_image_vec = if let Some(handle) = handle_ims_image_opt {
        handle.await.unwrap()
    } else {
        None
    }; */
    let ims_image_vec = if let Some(handle) = handle_ims_image_opt {
        match handle.await.unwrap() {
            Ok(ims_image_vec) => Some(ims_image_vec),
            Err(e) => return Err(e),
        }
    } else {
        None
    };

    let duration = start.elapsed();
    log::info!("Time elapsed to get CFS configurations, CFS sessions, BSS bootparameters and images bundle is: {:?}", duration);

    Ok((
        cfs_configuration_vec,
        cfs_session_vec,
        bos_sessiontemplate_vec,
        ims_image_vec,
    ))
} */

/* pub async fn get_configurations_sessions_bos_sessiontemplates_images_components_bootparameters(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    get_cfs_configuration: bool,
    get_cfs_session: bool,
    get_bos_sessiontemplate: bool,
    get_ims_image: bool,
    get_cfs_component: bool,
    get_bss_bootparameters: bool,
) -> (
    Option<Result<Vec<CfsConfigurationResponse>, Error>>,
    Option<Result<Vec<CfsSessionGetResponse>, Error>>,
    Option<Result<Vec<BosSessionTemplate>, Error>>,
    Option<Result<Vec<Image>, Error>>,
    Option<Result<Vec<Component>, Error>>,
    Option<Result<Vec<BootParameters>, Error>>,
) {
    let start = Instant::now();

    let handle_cfs_component_opt = OptionFuture::from(get_cfs_component.then(|| {
        cfs::component::http_client::v2::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
        )
    }));

    let handle_cfs_configuration_opt = OptionFuture::from(get_cfs_configuration.then(|| {
        cfs::configuration::http_client::v2::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
        )
    }));

    let handle_cfs_session_opt = OptionFuture::from(get_cfs_session.then(|| {
        cfs::session::get_and_sort(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
            None,
            None,
            None,
        )
    }));

    let handle_bos_sessiontemplate_opt = OptionFuture::from(get_bos_sessiontemplate.then(|| {
        bos::template::http_client::v2::get_all(shasta_token, shasta_base_url, shasta_root_cert)
    }));

    let handle_ims_image_opt = OptionFuture::from(get_ims_image.then(|| {
        ims::image::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
    }));

    let handle_bss_bootparameters_opt = OptionFuture::from(
        get_bss_bootparameters
            .then(|| bss::http_client::get(shasta_token, shasta_base_url, shasta_root_cert, &[])),
    );

    let value = tokio::join!(
        handle_cfs_configuration_opt,
        handle_cfs_session_opt,
        handle_bos_sessiontemplate_opt,
        handle_ims_image_opt,
        handle_cfs_component_opt,
        handle_bss_bootparameters_opt
    );

    let duration = start.elapsed();
    log::info!("Time elapsed to get CFS configurations, CFS sessions, BOS sessiontemplate, IMS images and BSS bootparameters bundle is: {:?}", duration);

    value
} */

/* pub async fn get_configurations_sessions_bos_sessiontemplates_images_components(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    get_cfs_configuration: bool,
    get_cfs_session: bool,
    get_bos_sessiontemplate: bool,
    get_ims_image: bool,
    get_cfs_component: bool,
) -> (
    Option<Vec<CfsConfigurationResponse>>,
    Option<Vec<CfsSessionGetResponse>>,
    Option<Vec<BosSessionTemplate>>,
    Option<Vec<Image>>,
    Option<Vec<Component>>,
) {
    let start = Instant::now();

    let handle_cfs_component_opt = if get_cfs_component {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            cfs::component::http_client::v2::get(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
                None,
                None,
            )
            .await
            .unwrap()
        }))
    } else {
        None
    };

    let handle_cfs_configuration_opt = if get_cfs_configuration {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            cfs::configuration::http_client::v2::get(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
                None,
            )
            .await
            .unwrap()
        }))
    } else {
        None
    };

    let handle_cfs_session_opt = if get_cfs_session {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            cfs::session::get_and_sort(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap()
        }))
    } else {
        None
    };

    let handle_bos_sessiontemplate_opt = if get_bos_sessiontemplate {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            bos::template::http_client::v2::get_all(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
            )
            .await
            .unwrap()
        }))
    } else {
        None
    };

    let handle_ims_image_opt = if get_ims_image {
        let shasta_token_string = shasta_token.to_string();
        let shasta_base_url_string = shasta_base_url.to_string();
        let shasta_root_cert_vec = shasta_root_cert.to_vec();

        Some(task::spawn(async move {
            ims::image::http_client::get_all(
                &shasta_token_string,
                &shasta_base_url_string,
                &shasta_root_cert_vec,
            )
            .await
            .unwrap()
        }))
    } else {
        None
    };

    let cfs_configuration_vec = if let Some(handle) = handle_cfs_configuration_opt {
        Some(handle.await.unwrap())
    } else {
        None
    };

    let cfs_session_vec = if let Some(handle) = handle_cfs_session_opt {
        Some(handle.await.unwrap())
    } else {
        None
    };

    let bos_sessiontemplate_vec = if let Some(handle) = handle_bos_sessiontemplate_opt {
        Some(handle.await.unwrap())
    } else {
        None
    };

    let ims_image_vec = if let Some(handle) = handle_ims_image_opt {
        Some(handle.await.unwrap())
    } else {
        None
    };

    let cfs_component_vec = if let Some(handle) = handle_cfs_component_opt {
        Some(handle.await.unwrap())
    } else {
        None
    };

    let duration = start.elapsed();
    log::info!("Time elapsed to get CFS configurations, CFS sessions, BSS bootparameters and images bundle is: {:?}", duration);

    (
        cfs_configuration_vec,
        cfs_session_vec,
        bos_sessiontemplate_vec,
        ims_image_vec,
        cfs_component_vec,
    )
} */

/* pub async fn get_configurations_sessions_bos_sessiontemplates_images_components_bootparameters_all(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
) -> (
    Result<Vec<Component>, Error>,
    Result<Vec<CfsConfigurationResponse>, Error>,
    Result<Vec<CfsSessionGetResponse>, Error>,
    Result<Vec<BosSessionTemplate>, Error>,
    Result<Vec<Image>, Error>,
    Result<Vec<BootParameters>, Error>,
) {
    let start = Instant::now();

    let values = tokio::join!(
        crate::cfs::component::http_client::v2::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
        ),
        crate::cfs::configuration::http_client::v2::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
        ),
        crate::cfs::session::get_and_sort(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
            None,
            None,
            None,
        ),
        crate::bos::template::http_client::v2::get_all(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        ),
        crate::ims::image::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert,),
        crate::bss::http_client::get(shasta_token, shasta_base_url, shasta_root_cert, &[],)
    );

    let duration = start.elapsed();
    log::info!("Time elapsed to get CFS configurations, CFS sessions, BOS sessiontemplate, IMS images and BSS bootparameters bundle is: {:?}", duration);

    values
} */
