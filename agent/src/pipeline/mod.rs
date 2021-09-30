pub mod print;

use anyhow::Result;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pipeline {
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    tasks: HashMap<String, Task>,
}

impl Pipeline {
    pub fn task<Name: Into<String>>(&mut self, name: Name, task: Task) -> &mut Self {
        self.tasks.insert(name.into(), task);
        self
    }
}

impl Pipeline {
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let mut pipeline = serde_yaml::from_str::<Self>(yaml)?;
        pipeline.on_load();
        Ok(pipeline)
    }

    fn on_load(&mut self) {
        for (name, task) in self.tasks.iter_mut() {
            if task.platform.is_empty() {
                task.platform = name.clone();
            }
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    platform: String,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    environment: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    build_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    test_targets: Vec<String>,
}

impl Task {
    pub fn platform<T: Into<String>>(&mut self, platform: T) -> &mut Self {
        self.platform = platform.into();
        self
    }

    pub fn environment<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    pub fn build_target<T: Into<String>>(&mut self, target: T) -> &mut Self {
        self.build_targets.push(target.into());
        self
    }

    pub fn test_target<T: Into<String>>(&mut self, target: T) -> &mut Self {
        self.test_targets.push(target.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_yaml_basic_syntax_works() {
        let yaml = r#"---
tasks:
  ubuntu_build_only:
    platform: ubuntu2004
    build_targets:
    - "..."
  windows:
    platform: windows
    build_targets:
    - "..."
    test_targets:
    - "..."
"#;
        let mut ubuntu_build_only = Task::default();
        ubuntu_build_only.platform("ubuntu2004").build_target("...");

        let mut windows = Task::default();
        windows
            .platform("windows")
            .build_target("...")
            .test_target("...");

        let mut pipeline = Pipeline::default();
        pipeline
            .task("ubuntu_build_only", ubuntu_build_only)
            .task("windows", windows);

        assert_eq!(pipeline, Pipeline::from_yaml(yaml).unwrap());
    }

    #[test]
    fn from_yaml_omit_platform() {
        let yaml = r#"---
tasks:
  ubuntu2004:
    build_targets:
    - "..."
  windows:
    build_targets:
    - "..."
    test_targets:
    - "..."
"#;
        let mut ubuntu2004 = Task::default();
        ubuntu2004.platform("ubuntu2004").build_target("...");

        let mut windows = Task::default();
        windows
            .platform("windows")
            .build_target("...")
            .test_target("...");

        let mut pipeline = Pipeline::default();
        pipeline.task("ubuntu2004", ubuntu2004);
        pipeline.task("windows", windows);

        assert_eq!(pipeline, Pipeline::from_yaml(yaml).unwrap());
    }

    #[test]
    fn from_yaml_use_environment_variables() {
        let yaml = r#"---
tasks:
  ubuntu1804:
    environment:
      CC: clang
    build_targets:
    - "..."
"#;

        let mut ubuntu1804 = Task::default();
        ubuntu1804
            .platform("ubuntu1804")
            .environment("CC", "clang")
            .build_target("...");

        let mut pipeline = Pipeline::default();
        pipeline.task("ubuntu1804", ubuntu1804);

        assert_eq!(pipeline, Pipeline::from_yaml(yaml).unwrap());
    }
}
