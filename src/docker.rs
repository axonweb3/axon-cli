use std::{fmt::Display, path::Path};

use docker_api::{
    docker::Docker,
    errors,
    errors::Result,
    models::{ContainerInspect200Response, ImageBuildChunk, ImageInspect, Network},
    opts::{ContainerCreateOpts, NetworkCreateOpts, PublishPort, PullOpts, RmContainerOpts},
    Id,
};
use futures::StreamExt;
use log::{debug, error, info};
use serde::ser::Serialize;

use crate::constants::*;

fn parse_inspect_result<T>(result: Result<T>) -> Result<Option<T>> {
    match result {
        Ok(result) => Ok(Some(result)),
        Err(err) => {
            if let errors::Error::Fault { code, .. } = err {
                if code.as_str() == "404" {
                    return Ok(None);
                }
            }

            Err(err)
        }
    }
}

pub struct StartAxonArgs<
    S0: AsRef<str>,
    S1: AsRef<str>,
    S2: AsRef<str>,
    S3: AsRef<str>,
    S4: AsRef<str>,
    P: AsRef<Path>,
> {
    pub name:            S0,
    pub config_path:     S1,
    pub genesis_path:    S2,
    pub network:         S3,
    pub data_volume:     S4,
    pub path:            P,
    pub port:            u32,
    pub collecting_port: u32,
    pub p2p_port:        u32,
}

pub struct DockerApi {
    docker: Docker,
}

impl DockerApi {
    pub fn new(uri: String) -> Result<Self> {
        Ok(Self {
            docker: Docker::new(uri)?,
        })
    }

    pub async fn find_network(&self, name: impl Into<Id>) -> Result<Option<Network>> {
        let inspect = self.docker.networks().get(name).inspect().await;

        parse_inspect_result(inspect)
    }

    pub async fn ensure_network(&self, name: impl AsRef<str>) -> Result<()> {
        debug!("Checking for network {}...", name.as_ref());
        let network = self.find_network(name.as_ref()).await?;

        if let Some(network) = network {
            debug!(
                "Network {} is existed, id: {}",
                name.as_ref(),
                network.id.unwrap_or_default(),
            );
            return Ok(());
        }

        info!("Network {} does't exist, creating...", name.as_ref());
        let id = self
            .docker
            .networks()
            .create(&NetworkCreateOpts::builder(name.as_ref()).build())
            .await?
            .id()
            .to_string();
        info!("Network {} has been created, id: {}", name.as_ref(), id);

        Ok(())
    }

    pub async fn find_image(&self, name: impl Into<Id>) -> Result<Option<ImageInspect>> {
        let inspect = self.docker.images().get(name).inspect().await;

        parse_inspect_result(inspect)
    }

    async fn ensure_image<S0: Serialize + Display, S1: Serialize + Display>(
        &self,
        name: S0,
        tag: S1,
    ) -> Result<()> {
        let image_name = format!("{}:{}", name, tag);
        debug!("Checking for image {}...", image_name);
        let image = self.find_image(&image_name).await?;

        if let Some(image) = image {
            debug!(
                "Image {} is existed, id: {}",
                image_name,
                image.id.unwrap_or_default(),
            );
            return Ok(());
        }

        info!("Image {} does't exist, pulling...", image_name);
        let opts = PullOpts::builder().image(name).tag(tag).build();
        let images = self.docker.images();
        let mut pulling = images.pull(&opts);

        while let Some(chunk) = pulling.next().await {
            match chunk? {
                ImageBuildChunk::PullStatus {
                    status,
                    id,
                    progress,
                    ..
                } => {
                    if progress.is_none() {
                        info!("{} {}", id.unwrap_or_default(), status);
                    }
                }
                ImageBuildChunk::Update { stream } => {
                    info!("Pulling image {} {}...", image_name, stream);
                }
                ImageBuildChunk::Digest { aux } => {
                    info!("Pulling image {} aux: {}...", image_name, aux.id);
                }
                ImageBuildChunk::Error { error, .. } => {
                    error!("Pulling image {} {}...", image_name, error);
                }
            };
        }

        Ok(())
    }

    pub async fn find_container(
        &self,
        name: impl Into<Id>,
    ) -> Result<Option<ContainerInspect200Response>> {
        let inspect = self.docker.containers().get(name).inspect().await;

        parse_inspect_result(inspect)
    }

    pub async fn get_container(
        &self,
        name: impl AsRef<str>,
    ) -> Result<Option<ContainerInspect200Response>> {
        let container = self.find_container(name.as_ref()).await?;

        if container.is_none() {
            error!("Container {} does't exist", name.as_ref());
        };

        Ok(container)
    }

    pub async fn remove_one_container(&self, name: impl AsRef<str>, force: bool) -> Result<()> {
        let container = self.get_container(name.as_ref()).await?;

        let container = match container {
            None => return Ok(()),
            Some(val) => val,
        };

        let id = container.id.unwrap_or_default();
        if !force {
            if let Some(state) = container.state {
                if state.running == Some(true) {
                    error!(
                        "Can't remove running container {}, id: {}",
                        name.as_ref(),
                        id
                    );
                    return Ok(());
                }
            }
        }

        let opts = RmContainerOpts::builder().force(force).build();

        info!("Removing container {}, id: {}...", name.as_ref(), id);
        self.docker
            .containers()
            .get(Id::from(name.as_ref()))
            .remove(&opts)
            .await?;
        info!("Removed container {}, id: {}", name.as_ref(), id);

        Ok(())
    }

    pub async fn remove_containers<S: AsRef<str>, T: IntoIterator<Item = S>>(
        &self,
        names: T,
        force: bool,
    ) -> Result<()> {
        futures::future::join_all(
            names
                .into_iter()
                .map(|name| self.remove_one_container(name, force)),
        )
        .await
        .into_iter()
        .collect::<Result<()>>()
    }

    pub async fn stop_one_container(&self, name: impl AsRef<str>) -> Result<()> {
        let container = self.get_container(name.as_ref()).await?;

        let container = match container {
            Some(val) => val,
            None => return Ok(()),
        };

        let id = container.id.unwrap_or_default();
        if let Some(state) = container.state {
            if state.running == Some(false) {
                error!("Can't stop stopped container {}, id: {}", name.as_ref(), id);
                return Ok(());
            }
        }

        info!("Stopping container {}, id: {}...", name.as_ref(), id);
        self.docker
            .containers()
            .get(name.as_ref())
            .stop(None)
            .await?;
        info!("Stopped container {}, id: {}", name.as_ref(), id);

        Ok(())
    }

    pub async fn stop_containers<S: AsRef<str>, T: IntoIterator<Item = S>>(
        &self,
        names: T,
    ) -> Result<()> {
        futures::future::join_all(names.into_iter().map(|name| self.stop_one_container(name)))
            .await
            .into_iter()
            .collect::<Result<()>>()
    }

    pub async fn ensure_container_running(
        &self,
        image: impl AsRef<str>,
        tag: impl AsRef<str>,
        name: impl AsRef<str>,
        get_opts: impl FnOnce() -> ContainerCreateOpts,
    ) -> Result<()> {
        self.ensure_image(image.as_ref(), tag.as_ref()).await?;

        let container = self.find_container(name.as_ref()).await?;

        let id = if let Some(container) = container {
            let id = container.id.unwrap_or_default();
            debug!("Container {} is existed, id: {}", name.as_ref(), id);

            if let Some(state) = container.state {
                if state.running == Some(true) {
                    error!("Container {} is already running, id: {}", name.as_ref(), id);
                    return Ok(());
                }
            }

            id
        } else {
            info!("Container {} does't exist, creating...", name.as_ref());
            let container = self.docker.containers().create(&get_opts()).await?;
            let id = container.id().to_string();
            info!("Container {} has been created, id: {}", name.as_ref(), id);

            id
        };

        self.docker.containers().get(name.as_ref()).start().await?;
        info!("Container {} has started, id: {}", name.as_ref(), id);

        Ok(())
    }

    pub async fn inspect_one_container(&self, name: impl AsRef<str>) -> Result<()> {
        let container = self.get_container(name.as_ref()).await?;

        let container = match container {
            Some(val) => val,
            None => return Ok(()),
        };

        info!(
            "Container {} is {}, id: {}",
            name.as_ref(),
            container
                .state
                .map(|s| s.status.unwrap_or_default())
                .unwrap_or_default(),
            container.id.unwrap_or_default(),
        );

        Ok(())
    }

    pub async fn inspect_containers<S: AsRef<str>, T: IntoIterator<Item = S>>(
        &self,
        names: T,
    ) -> Result<()> {
        futures::future::join_all(
            names
                .into_iter()
                .map(|name| self.inspect_one_container(name)),
        )
        .await
        .into_iter()
        .collect::<Result<()>>()
    }

    pub async fn remove_one_volume(&self, name: impl AsRef<str>) -> Result<()> {
        let remove = self.docker.volumes().get(name.as_ref()).delete().await;

        match parse_inspect_result(remove)? {
            Some(_) => {
                info!("Volume {} removed", name.as_ref());
            }
            None => {
                error!("Volume {} doesn't exist", name.as_ref());
            }
        }

        Ok(())
    }

    pub async fn start_axon<
        S0: AsRef<str>,
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        S4: AsRef<str>,
        P: AsRef<Path>,
    >(
        &self,
        args: StartAxonArgs<S0, S1, S2, S3, S4, P>,
    ) -> Result<()> {
        let StartAxonArgs {
            name,
            config_path,
            genesis_path,
            network,
            data_volume,
            path,
            port,
            p2p_port,
            collecting_port,
        } = args;

        self.ensure_container_running(AXON_IMAGE_NAME, AXON_IMAGE_TAG, &name, || {
            let cmd = [
                "./axon".to_string(),
                format!("-c=/app/nodes/{}", config_path.as_ref()),
                format!("-g=/app/nodes/{}", genesis_path.as_ref()),
            ];

            let config_path = path.as_ref().join("nodes");
            let logs_path = path.as_ref().join("logs");
            let volumes = [
                format!("{}:/app/nodes/data", data_volume.as_ref()),
                format!("{}:/app/nodes", config_path.to_str().unwrap()),
                format!("{}:/app/logs", logs_path.to_str().unwrap()),
            ];

            ContainerCreateOpts::builder(format!("{}:{}", AXON_IMAGE_NAME, AXON_IMAGE_TAG))
                .name(name.as_ref())
                .cmd(cmd)
                .restart_policy("always", 0)
                .volumes(volumes)
                .working_dir("/app")
                .network_mode(network.as_ref())
                .expose(PublishPort::tcp(8000), port)
                .expose(PublishPort::tcp(8001), p2p_port)
                .expose(PublishPort::tcp(8100), collecting_port)
                .build()
        })
        .await
    }

    pub async fn start_benchmark(
        &self,
        path: impl AsRef<Path>,
        http_endpoint: impl AsRef<str>,
        network: impl AsRef<str>,
    ) -> Result<()> {
        self.ensure_container_running(
            BENCHMARK_IMAGE_NAME,
            BENCHMARK_IMAGE_TAG,
            "benchmark",
            || {
                let config_path = path.as_ref().join("config.json");
                let logs_path = path.as_ref().join("logs");
                let vols = vec![
                    format!("{}:/benchmark/config.json", config_path.to_str().unwrap()),
                    format!("{}:/benchmark/logs", logs_path.to_str().unwrap()),
                ];

                ContainerCreateOpts::builder(BENCHMARK_IMAGE_NAME)
                    .name("benchmark")
                    .cmd([
                        "node",
                        "index.js",
                        &format!("--http_endpoint={}", http_endpoint.as_ref()),
                    ])
                    .volumes(vols)
                    .network_mode(network.as_ref())
                    .build()
            },
        )
        .await
    }
}
