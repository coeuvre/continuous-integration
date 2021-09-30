use std::collections::HashMap;

use anyhow::Result;
use serde::{ser::SerializeSeq, Serialize};

#[derive(Default, Debug, PartialEq, Eq, Serialize)]
pub struct Pipeline {
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    agents: HashMap<String, String>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    steps: Vec<Step>,
}

impl Pipeline {
    pub fn command_step(&mut self, command: Command) -> &mut Self {
        self.steps.push(Step::Command(command));
        self
    }

    pub fn agent<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.agents.insert(key.into(), value.into());
        self
    }

    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Step {
    Command(Command),
}

#[derive(Default, Debug, PartialEq, Eq, Serialize)]
pub struct Command {
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    agents: HashMap<String, String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    command: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    commands: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    label: String,
    #[serde(skip_serializing_if = "Plugins::is_empty")]
    plugins: Plugins,
}

impl Command {
    pub fn agent<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.agents.insert(key.into(), value.into());
        self
    }

    pub fn command<T: Into<String>>(&mut self, command: T) -> &mut Self {
        if self.is_empty() {
            self.command = command.into();
        } else {
            if !self.command.is_empty() {
                let mut first_command = "".to_string();
                std::mem::swap(&mut first_command, &mut self.command);
                self.commands.push(first_command);
            }

            self.commands.push(command.into());
        }

        self
    }

    pub fn label<T: Into<String>>(&mut self, label: T) -> &mut Self {
        self.label = label.into();
        self
    }

    pub fn docker_plugin<S: Into<String>>(
        &mut self,
        version: S,
        plugin: DockerPlugin,
    ) -> &mut Self {
        self.plugins.0.push(Plugin::Docker {
            version: version.into(),
            properties: plugin,
        });
        self
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
            && self.command.is_empty()
            && self.commands.is_empty()
            && self.label.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Plugin {
    Docker {
        version: String,
        properties: DockerPlugin,
    },
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Plugins(Vec<Plugin>);

impl Plugins {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Serialize for Plugins {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for plugin in self.0.iter() {
            match plugin {
                Plugin::Docker {
                    version,
                    properties,
                } => {
                    let mut map = HashMap::with_capacity(1);
                    map.insert(format!("docker#{}", version), properties);
                    seq.serialize_element(&map)?;
                }
            }
        }
        seq.end()
    }
}

#[derive(Default, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DockerPlugin {
    pub always_pull: bool,
    pub environment: Vec<String>,
    pub image: String,
    pub network: String,
    pub privileged: bool,
    pub propagate_environment: bool,
    pub propagate_uid_gid: bool,
    pub volumes: Vec<String>,
}

impl DockerPlugin {
    pub fn always_pull(&mut self, always_pull: bool) -> &mut Self {
        self.always_pull = always_pull;
        self
    }

    pub fn environment<T: Into<String>>(&mut self, env: T) -> &mut Self {
        self.environment.push(env.into());
        self
    }

    pub fn image<T: Into<String>>(&mut self, image: T) -> &mut Self {
        self.image = image.into();
        self
    }

    pub fn network<T: Into<String>>(&mut self, network: T) -> &mut Self {
        self.network = network.into();
        self
    }

    pub fn privileged(&mut self, privileged: bool) -> &mut Self {
        self.privileged = privileged;
        self
    }

    pub fn propagate_environment(&mut self, propagate_environment: bool) -> &mut Self {
        self.propagate_environment = propagate_environment;
        self
    }

    pub fn propagate_uid_gid(&mut self, propagate_uid_gid: bool) -> &mut Self {
        self.propagate_uid_gid = propagate_uid_gid;
        self
    }

    pub fn volume<T: Into<String>>(&mut self, volume: T) -> &mut Self {
        self.volumes.push(volume.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_yaml_single_command_works() {
        let mut command = Command::default();
        command.command("command.sh");
        let mut pipeline = Pipeline::default();
        pipeline.command_step(command);

        let yaml = r#"---
steps:
  - command: command.sh
"#;

        assert_eq!(yaml, pipeline.to_yaml().unwrap());
    }

    #[test]
    fn to_yaml_multiple_commands_works() {
        let mut pipeline = Pipeline::default();
        let mut command = Command::default();
        command.command("command1.sh");
        command.command("command2.sh");
        pipeline.command_step(command);

        let yaml = r#"---
steps:
  - commands:
      - command1.sh
      - command2.sh
"#;
        assert_eq!(yaml, pipeline.to_yaml().unwrap());
    }

    #[test]
    fn to_yaml_docker_plugin_works() {
        let mut command = Command::default();
        let mut docker_plugin = DockerPlugin::default();
        docker_plugin
            .always_pull(true)
            .environment("CC")
            .image("gcr.io/bazel-public/ubuntu1804-java11")
            .network("host")
            .privileged(true)
            .propagate_environment(true)
            .propagate_uid_gid(true)
            .volume("/etc/group:/etc/group:ro");
        command.docker_plugin("3.8.0", docker_plugin);
        let mut pipeline = Pipeline::default();
        pipeline.command_step(command);

        let yaml = r#"---
steps:
  - plugins:
      - "docker#3.8.0":
          always-pull: true
          environment:
            - CC
          image: gcr.io/bazel-public/ubuntu1804-java11
          network: host
          privileged: true
          propagate-environment: true
          propagate-uid-gid: true
          volumes:
            - "/etc/group:/etc/group:ro"
"#;

        assert_eq!(yaml, pipeline.to_yaml().unwrap());
    }
}
