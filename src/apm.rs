use std::{
    fs::{create_dir_all, write},
    path::Path,
    process,
};

use async_trait::async_trait;
use clap::{ArgMatches, Args, Command, FromArgMatches, Subcommand};
use log::info;

use crate::{
    constants::{
        APM_CONFIGS, APM_MONITOR_PROMETHEUS_TEMPLATE, APM_MONITOR_VARS_TEMPLATE,
        DEFAULT_APM_MONITOR_PATH, DEFAULT_APM_PATH,
    },
    sub_command::SubCommand,
    types::Result,
    utils::read_or_create_plain_template,
};

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct StartApmMonitorArgs {
    /// the working path of APM
    #[clap(short, long, default_value=*DEFAULT_APM_PATH)]
    path: String,

    /// the working path of APM monitor
    #[clap(short, long, default_value=*DEFAULT_APM_MONITOR_PATH)]
    monitor_path: String,
}

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct StopApmMonitorArgs {
    /// the working path of APM
    #[clap(short, long, default_value=*DEFAULT_APM_PATH)]
    path: String,
}

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct StartApmAgentArgs {
    /// the working path of APM
    #[clap(short, long, default_value=*DEFAULT_APM_PATH)]
    path: String,

    /// specify inventory host path or comma separated host list.
    #[clap(short, long)]
    inventory: Option<String>,
}

#[derive(Args, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct StopApmAgentArgs {
    /// the working path of APM
    #[clap(short, long, default_value=*DEFAULT_APM_PATH)]
    path: String,

    /// specify inventory host path or comma separated host list.
    #[clap(short, long)]
    inventory: Option<String>,
}

#[derive(Subcommand, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum ApmMonitorActions {
    /// Start APM monitor
    Start(StartApmMonitorArgs),

    /// Stop APM monitor
    Stop(StopApmMonitorArgs),
}

#[derive(Subcommand, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum ApmAgentActions {
    /// Start APM agent(s)
    Start(StartApmAgentArgs),

    /// Stop APM agent(s)
    Stop(StopApmAgentArgs),
}

#[derive(Subcommand, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum ApmActions {
    /// Manage APM monitor
    #[clap(subcommand)]
    Monitor(ApmMonitorActions),

    /// Manage APM agent(s)
    #[clap(subcommand)]
    Agent(ApmAgentActions),
}

#[derive(Debug, Default)]
pub struct Apm {}

#[async_trait]
impl SubCommand for Apm {
    fn get_command(&self) -> Command<'static> {
        ApmActions::augment_subcommands(Command::new("apm"))
            .about("Manage APM (Application Performance Management)")
    }

    async fn exec_command(&mut self, matches: &ArgMatches) -> Result<()> {
        match ApmActions::from_arg_matches(matches)? {
            ApmActions::Monitor(ApmMonitorActions::Start(StartApmMonitorArgs { path, .. }))
            | ApmActions::Monitor(ApmMonitorActions::Stop(StopApmMonitorArgs { path, .. }))
            | ApmActions::Agent(ApmAgentActions::Start(StartApmAgentArgs { path, .. }))
            | ApmActions::Agent(ApmAgentActions::Stop(StopApmAgentArgs { path, .. })) => {
                extract_apm_configs(path)?;
            }
        };

        match ApmActions::from_arg_matches(matches)? {
            ApmActions::Monitor(ApmMonitorActions::Start(args)) => {
                start_monitor(args)?;
            }
            ApmActions::Monitor(ApmMonitorActions::Stop(args)) => {
                stop_monitor(args)?;
            }
            ApmActions::Agent(ApmAgentActions::Start(args)) => {
                start_agent(args)?;
            }
            ApmActions::Agent(ApmAgentActions::Stop(args)) => {
                stop_agent(args)?;
            }
        };

        Ok(())
    }
}

fn extract_apm_configs(path: impl AsRef<Path>) -> Result<()> {
    if path.as_ref().exists() {
        return Ok(());
    }

    create_dir_all(&path)?;
    APM_CONFIGS.extract(path)?;

    Ok(())
}

fn start_monitor(args: StartApmMonitorArgs) -> Result<()> {
    let StartApmMonitorArgs {
        path: path_str,
        monitor_path,
    } = args;
    let path: &Path = path_str.as_ref();

    let vars = read_or_create_plain_template(
        path.join("apm_monitor_vars_template.yaml"),
        APM_MONITOR_VARS_TEMPLATE,
    )?
    .replace("{MONITOR_PATH}", &monitor_path);
    let prometheus = read_or_create_plain_template(
        path.join("apm_monitor_prometheus_template.yaml"),
        APM_MONITOR_PROMETHEUS_TEMPLATE,
    )?
    .replace("{MONITOR_PATH}", &monitor_path);

    write(
        path.join("deploy")
            .join("roles")
            .join("monitor")
            .join("vars")
            .join("main.yml"),
        vars.as_bytes(),
    )?;
    write(
        path.join("monitor")
            .join("config")
            .join("promethues")
            .join("prometheus.yml"),
        prometheus.as_bytes(),
    )?;

    info!(
        "\
        ansible-playbook \
        deploy_monitor.yml \
        --tags config,down,start \
        --ask-become-pass\
    "
    );

    let output = process::Command::new("ansible-playbook")
        .arg("deploy_monitor.yml")
        .args(["--tags", "config,down,start"])
        .arg("--ask-become-pass")
        .current_dir(path.join("deploy"))
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);

    Ok(())
}

fn stop_monitor(args: StopApmMonitorArgs) -> Result<()> {
    let StopApmMonitorArgs { path: path_str } = args;
    let path: &Path = path_str.as_ref();

    info!(
        "\
        ansible-playbook \
        deploy_monitor.yml \
        --tags down,clean \
        --ask-become-pass\
    "
    );

    let output = process::Command::new("ansible-playbook")
        .arg("deploy_monitor.yml")
        .args(["--tags", "down,clean"])
        .arg("--ask-become-pass")
        .current_dir(path.join("deploy"))
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);

    Ok(())
}

fn start_agent(args: StartApmAgentArgs) -> Result<()> {
    let StartApmAgentArgs {
        path: path_str,
        inventory,
    } = args;
    let path: &Path = path_str.as_ref();

    let mut command = process::Command::new("ansible-playbook");

    match inventory {
        None => {
            info!(
                "\
                ansible-playbook \
                deploy_monitor_agent.yml \
                --tags config,down,start \
                --ask-become-pass\
            "
            );
        }
        Some(inventory) => {
            info!(
                "\
                ansible-playbook \
                -i {inventory} \
                deploy_monitor_agent.yml \
                --tags config,down,start \
                --ask-become-pass\
            "
            );
            command.args(["-i", &inventory]);
        }
    };

    let output = command
        .arg("deploy_monitor_agent.yml")
        .args(["--tags", "config,down,start"])
        .arg("--ask-become-pass")
        .current_dir(path.join("deploy"))
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);

    Ok(())
}

fn stop_agent(args: StopApmAgentArgs) -> Result<()> {
    let StopApmAgentArgs {
        path: path_str,
        inventory,
    } = args;
    let path: &Path = path_str.as_ref();

    let mut command = process::Command::new("ansible-playbook");

    match inventory {
        None => {
            info!(
                "\
                ansible-playbook \
                deploy_monitor_agent.yml \
                --tags down,clean \
                --ask-become-pass\
            "
            );
        }
        Some(inventory) => {
            info!(
                "\
                ansible-playbook \
                -i {inventory} \
                deploy_monitor_agent.yml \
                --tags down,clean \
                --ask-become-pass\
            "
            );
            command.args(["-i", &inventory]);
        }
    };

    let output = process::Command::new("ansible-playbook")
        .arg("deploy_monitor_agent.yml")
        .args(["--tags", "down,clean"])
        .arg("--ask-become-pass")
        .current_dir(path.join("deploy"))
        .output()?;

    println!("{}", std::str::from_utf8(&output.stdout)?);

    Ok(())
}
