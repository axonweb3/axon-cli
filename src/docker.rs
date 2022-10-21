use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

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
    pub fn new_default() -> Result<Self> {
        Self::new(get_default_docker_uri().to_string())
    }

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
                network.id.unwrap_or_else(|| "".to_string()),
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
                image.id.unwrap_or_else(|| "".to_string()),
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
                        info!("{} {}", id.unwrap_or_else(|| "".to_string()), status);
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

        let id = container.id.unwrap_or_else(|| "".to_string());
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

        let id = container.id.unwrap_or_else(|| "".to_string());
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
            let id = container.id.unwrap_or_else(|| "".to_string());
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
                .map(|s| s.status.unwrap_or_else(|| "".to_string()))
                .unwrap_or_else(|| "".to_string()),
            container.id.unwrap_or_else(|| "".to_string()),
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

    pub fn get_var(var_path: &str, key: &str) -> std::io::Result<Option<String>> {
        let file = File::open(var_path)?;
        let lines = BufReader::new(file).lines();
        let mut var_val: Option<String> = None;
        for line in lines.flatten() {
            // println!("{}", line);
            let words: Vec<&str> = line.split(':').collect();
            if words[0] == key {
                var_val = Some(words[1].trim().to_string());
                break;
            }
        }
        Ok(var_val)
    }

    pub async fn start_monitor(&self, path: &str) {
        let monitor_var_path = path.to_owned() + "/deploy/roles/monitor/vars/main.yaml";
        println!("monitor var path {}", monitor_var_path);
        let var_key = "monitor_dir";
        if let Ok(monitor_dir_opt) = DockerApi::get_var(monitor_var_path.as_str(), var_key) {
            if let Some(monitor_dir) = monitor_dir_opt {
                // println!("monitor dir:{}", monitor_dir);
                self.start_grafana(&monitor_dir).await;
                self.start_grafana_renderer().await;
                self.start_prometheus(&monitor_dir).await;
                self.start_elasticsearch(&monitor_dir).await;
                self.start_jaeger_collector().await;
                self.start_jaeger_query().await;
                self.start_elastalert(&monitor_dir).await;
            } else {
                println!("Key {} not exist!", var_key);
            }
        } else {
            println!("File {} open err!", monitor_var_path);
        }
    }

    pub async fn start_agent(&self, path: &str) {
        let agent_var_path = path.to_owned() + "/deploy/roles/agent/vars/main.yaml";
        println!("agent var path {}", agent_var_path);
        let var_key = "monitor_agent_dir";
        if let Ok(agent_dir_opt) = DockerApi::get_var(agent_var_path.as_str(), var_key) {
            if let Some(agent_dir) = agent_dir_opt {
                // println!("agent_dir:{}", agent_dir);
                let log_path = DockerApi::get_var(agent_var_path.as_str(), "log_path")
                    .unwrap()
                    .unwrap();
                let mut monitor_address =
                    DockerApi::get_var(agent_var_path.as_str(), "monitor_address")
                        .unwrap()
                        .unwrap();
                monitor_address += ":8201";

                self.start_node_exporter().await;
                self.start_jaeger_agent(&monitor_address).await;
                self.start_promtail(&agent_dir, &log_path).await;
                self.start_filebeat(&agent_dir, &log_path).await;
            } else {
                println!("Key {} not exist!", var_key);
            }
        } else {
            println!("File {} open err!", agent_var_path);
        }
    }

    pub async fn stop_monitor(&self) {
        self.stop_containers([
            "axon-grafana",
            "axon-grafana-image-renderer",
            "prometheus",
            "elasticsearch",
            "jaeger-collector",
            "jaeger-query",
            "elk-elastalert",
        ])
        .await;
    }

    pub async fn stop_agent(&self) {
        self.stop_containers([
            "axon-node-exporter",
            "jaeger-agent",
            "axon-promtail",
            "axon-filebeat",
        ])
        .await;
    }

    pub async fn clean(path: &str) {
        let monitor_var_path = path.to_owned() + "/deploy/roles/monitor/vars/main.yaml";
        DockerApi::rm_data(monitor_var_path.as_str(), "monitor_dir");

        let agent_var_path = path.to_owned() + "/deploy/roles/agent/vars/main.yaml";
        DockerApi::rm_data(agent_var_path.as_str(), "monitor_agent_dir");
    }

    fn rm_data(var_path: &str, key: &str) {
        let data_path = DockerApi::get_var(var_path, key).unwrap().unwrap() + "/data";
        println!("data path:{}", data_path);
        let output = std::process::Command::new("rm")
            .args(["-rf", data_path.as_str()])
            .output();

        match output {
            Ok(_) => println!("Rm {} Successfully", data_path),
            Err(err) => println!("Err {:?}", err),
        }
    }

    async fn start_grafana(&self, dir: &str) {
        let image_name = "grafana/grafana";
        let image_tag = "master";
        self.ensure_image(image_name, image_tag).await;

        let vols = vec![
            dir.to_owned() + "/config/grafana/grafana.ini:/etc/grafana/grafana.ini",
            dir.to_owned() + "/config/grafana/dashboards:/var/lib/grafana/dashboards",
            dir.to_owned() + "/config/grafana/provisioning:/etc/grafana/provisioning",
            dir.to_owned() + "/data/grafana/log:/var/log/grafana",
        ];
        let env = vec![
            "GF_EXPLORE_ENABLED=true",
            "GF_RENDERING_SERVER_URL=http://renderer:8081/render",
            "GF_RENDERING_CALLBACK_URL=http://grafana:3000/",
            "GF_LOG_FILTERS=rendering:debug",
        ];

        let name = "axon-grafana";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .restart_policy("on-failure", 0)
            .expose(PublishPort::tcp(3000), 8600)
            .volumes(vols)
            .env(env)
            .network_mode("axon-monitor")
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_grafana_renderer(&self) {
        let image_name = "grafana/grafana-image-renderer";
        let image_tag = "2.0.0";
        self.ensure_image(image_name, image_tag).await;
        let name = "axon-grafana-image-renderer";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .expose(PublishPort::tcp(8081), 0)
            .network_mode("axon-monitor")
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_prometheus(&self, dir: &str) {
        let image_name = "prom/prometheus";
        let image_tag = "v2.32.1";
        self.ensure_image(image_name, image_tag).await;
        let vols = vec![
            dir.to_owned() + "/config/promethues/prometheus.yml:/etc/prometheus/prometheus.yml",
            dir.to_owned() + "/data/prometheus:/prometheus",
        ];
        let cmd = vec![
            "--config.file=/etc/prometheus/prometheus.yml",
            "--storage.tsdb.path=/prometheus",
            "--web.console.libraries=/usr/share/prometheus/console_libraries",
            "--web.console.templates=/usr/share/prometheus/consoles",
            "--web.enable-lifecycle",
        ];

        let name = "prometheus";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .restart_policy("on-failure", 0)
            .volumes(vols)
            .expose(PublishPort::tcp(9090), 9090)
            .cmd(cmd)
            .network_mode("axon-monitor")
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_elasticsearch(&self, dir: &str) {
        let image_name = "docker.elastic.co/elasticsearch/elasticsearch";
        let image_tag = "7.6.2";
        self.ensure_image(image_name, image_tag).await;

        let vols = vec![dir.to_owned() + "/data/es:/usr/share/elasticsearch/data"];
        let env = vec![
            "cluster.name=jaeger-cluster",
            "discovery.type=single-node",
            "http.host=0.0.0.0",
            "transport.host=127.0.0.1",
            "ES_JAVA_OPTS=-Xms8192m -Xmx8192m",
            "xpack.security.enabled=false",
        ];

        let name = "elasticsearch";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .expose(PublishPort::tcp(9200), 9200)
            .expose(PublishPort::tcp(9300), 9300)
            .restart_policy("on-failure", 0)
            .env(env)
            .volumes(vols)
            .network_mode("axon-monitor")
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_jaeger_collector(&self) {
        let image_name = "jaegertracing/jaeger-collector";
        let image_tag = "1.32";
        self.ensure_image(image_name, image_tag).await;

        let env = vec!["SPAN_STORAGE_TYPE=elasticsearch"];
        let cmd = vec![
            "--es.server-urls=http://elasticsearch:9200",
            "--es.num-shards=1",
            "--es.num-replicas=0",
            "--log-level=error",
        ];

        let name = "jaeger-collector";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .expose(PublishPort::tcp(14269), 14269)
            .expose(PublishPort::tcp(14268), 14268)
            .expose(PublishPort::tcp(14250), 8201)
            .expose(PublishPort::tcp(9411), 9411)
            .restart_policy("on-failure", 0)
            .env(env)
            .cmd(cmd)
            .network_mode("axon-monitor")
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_jaeger_query(&self) {
        let image_name = "jaegertracing/jaeger-query";
        let image_tag = "1.32";
        self.ensure_image(image_name, image_tag).await;

        let env = vec!["SPAN_STORAGE_TYPE=elasticsearch", "no_proxy=localhost"];
        let cmd = vec![
            "--es.server-urls=http://elasticsearch:9200",
            "--span-storage.type=elasticsearch",
            "--log-level=debug",
            "--query.max-clock-skew-adjustment=0",
        ];

        let name = "jaeger-query";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .env(env)
            .expose(PublishPort::tcp(16686), 8202)
            .expose(PublishPort::tcp(16687), 8203)
            .restart_policy("on-failure", 0)
            .cmd(cmd)
            .network_mode("axon-monitor")
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_elastalert(&self, dir: &str) {
        let image_name = "praecoapp/elastalert-server";
        let image_tag = "20210704";
        self.ensure_image(image_name, image_tag).await;

        let vols = vec![
            dir.to_owned() + "/config/elastalert2/elastalert.yaml:/opt/elastalert/config.yaml",
            dir.to_owned()
                + "/config/elastalert2/config.json:/opt/elastalert-server/config/config.json",
            dir.to_owned() + "/config/elastalert2/rules:/opt/elastalert/rules",
            dir.to_owned() + "/config/elastalert2/rule_templates:/opt/elastalert/rule_templates",
        ];

        let name = "elk-elastalert";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .expose(PublishPort::tcp(3330), 3330)
            .expose(PublishPort::tcp(3333), 3333)
            .user("1000:1000")
            .volumes(vols)
            .network_mode("axon-monitor")
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_node_exporter(&self) {
        let image_name = "quay.io/prometheus/node-exporter";
        let image_tag = "v0.18.1";
        self.ensure_image(image_name, image_tag).await;

        let cmd = vec![
            "--path.rootfs=/host",
            "--collector.tcpstat",
            "--web.listen-address=:8101",
        ];
        let vols = vec!["/:/host:ro,rslave"];

        let name = "axon-node-exporter";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .cmd(cmd)
            .restart_policy("on-failure", 0)
            .network_mode("host")
            .volumes(vols)
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_jaeger_agent(&self, addr: &str) {
        let image_name = "jaegertracing/jaeger-agent";
        let image_tag = "1.32";
        self.ensure_image(image_name, image_tag).await;

        // let cmd = vec!["--reporter.grpc.host-port=${JACGER_COLLECTOR_ADDRESS}"];
        let cmd = vec!["--reporter.grpc.host-port=".to_owned() + addr];

        let name = "jaeger-agent";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .restart_policy("on-failure", 0)
            .expose(PublishPort::tcp(14271), 14271)
            .expose(PublishPort::udp(5775), 5775)
            .expose(PublishPort::udp(6831), 6831)
            .expose(PublishPort::udp(6832), 6832)
            .expose(PublishPort::tcp(5778), 5778)
            .cmd(cmd)
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_promtail(&self, dir: &str, log_path: &str) {
        let image_name = "grafana/promtail";
        let image_tag = "master-9ad98df";
        self.ensure_image(image_name, image_tag).await;

        let vols = vec![
            dir.to_owned() + "/data/promtail/positions:/tmp/promtail/",
            dir.to_owned()
                + "/config/promtail/promtail-config.yaml:/etc/promtail/promtail-config.yaml",
            // "${AXON_LOG_PATH}:/var/logs",
            log_path.to_owned() + ":/var/logs",
        ];
        let cmd = vec!["-config.file=/etc/promtail/promtail-config.yaml"];

        let name = "axon-promtail";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .restart_policy("on-failure", 0)
            .expose(PublishPort::tcp(9080), 8102)
            .volumes(vols)
            .cmd(cmd)
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_filebeat(&self, dir: &str, log_path: &str) {
        let image_name = "docker.elastic.co/beats/filebeat";
        let image_tag = "7.2.0";
        self.ensure_image(image_name, image_tag).await;

        let vols = vec![
            "/var/run/docker.sock:/host_docker/docker.sock".to_string(),
            "/var/lib/docker:/host_docker/var/lib/docker".to_string(),
            dir.to_owned() + "/config/filebeat/filebeat.yml:/usr/share/filebeat/filebeat.yml:ro",
            // "${AXON_LOG_PATH}:/usr/share/filebeat/logs",
            log_path.to_owned() + ":/usr/share/filebeat/logs",
        ];
        let cmd = vec!["--strict.perms=false"];

        let name = "axon-filebeat";
        let opts = ContainerCreateOpts::builder(image_name.to_owned() + ":" + image_tag)
            .name(name)
            .user("root")
            .volumes(vols)
            .cmd(cmd)
            .attach_stdin(true)
            .build();
        // println!("opts {:?}", opts);
        println!("Start: {}", name);
        self.start_container(opts).await;
    }

    async fn start_container(&self, opts: ContainerCreateOpts) {
        match self.docker.containers().create(&opts).await {
            Ok(container) => {
                // println!("{:?}", container);
                match container.start().await {
                    Ok(_) => println!("Start ok"),
                    Err(err) => eprintln!("Start err {}", err),
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        };
    }
}
