use std::path::PathBuf;

use anyhow::Result;

use crate::{buildkite, config::Config, utils::load_file};

use super::Pipeline;

pub enum Mode {
    Buildkite,
}

pub fn print(pipeline: String, config: Option<PathBuf>, mode: Mode) -> Result<()> {
    let pipeline = {
        let yaml = load_file(&pipeline)?;
        Pipeline::from_yaml(&yaml)?
    };

    let config = match config {
        Some(path) => Config::from_path(&path)?,
        None => Config::default(),
    };

    match mode {
        Mode::Buildkite => print_as_buildkite(&pipeline, &config)?,
    }

    Ok(())
}

fn print_as_buildkite(pipeline: &Pipeline, config: &Config) -> Result<()> {
    let mut command = buildkite::Command::default();
    command.agent("queue", "default");
    let mut docker_plugin = buildkite::DockerPlugin::default();
    docker_plugin
        .always_pull(true)
        .environment("ANDROID_HOME")
        .environment("ANDROID_NDK_HOME")
        .environment("BUILDKITE_ARTIFACT_UPLOAD_DESTINATION")
        .network("host")
        .privileged(true)
        .propagate_environment(true)
        .propagate_uid_gid(true)
        .volume("/etc/group:/etc/group:ro")
        .volume("/etc/passwd:/etc/passwd:ro")
        .volume("/opt:/opt:ro")
        .volume("/var/lib/buildkite-agent:/var/lib/buildkite-agent")
        .volume("/var/lib/gitmirrors:/var/lib/gitmirrors:ro")
        .volume("/var/run/docker.sock:/var/run/docker.sock");
    docker_plugin.image("gcr.io/bazel-public/ubuntu1804-java11");

    command.docker_plugin("3.8.0", docker_plugin);

    let mut buildkite_pipeline = buildkite::Pipeline::default();
    buildkite_pipeline.command_step(command);

    print!("{}", buildkite_pipeline.to_yaml()?);

    Ok(())
}
