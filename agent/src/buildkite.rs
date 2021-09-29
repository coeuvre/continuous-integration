use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pipeline {
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    agents: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    env: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    steps: Vec<Step>,
}

impl Pipeline {
    pub fn command_step(&mut self, command: Command) -> &mut Self {
        self.steps.push(Step::Command(command));
        self
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Step {
    Command(Command),
}

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    commands: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    label: String,
}

impl Command {
    pub fn new() -> Self {
        Self::default()
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

    pub fn is_empty(&self) -> bool {
        self.command.is_empty() && self.commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_yaml_single_command() {
        let mut command = Command::new();
        command.command("command.sh");
        let mut pipeline = Pipeline::default();
        pipeline.command_step(command);

        let yaml = r#"---
steps:
  - command: command.sh
"#;

        assert_eq!(yaml, serde_yaml::to_string(&pipeline).unwrap(),);
    }

    #[test]
    fn from_yaml_single_command() {
        let mut command = Command::new();
        command.command("command.sh");
        let mut pipeline = Pipeline::default();
        pipeline.command_step(command);

        let yaml = r#"
steps:
  - command: "command.sh"
"#;
        assert_eq!(pipeline, serde_yaml::from_str::<Pipeline>(yaml).unwrap());
    }

    #[test]
    fn to_yaml_multiple_commands() {
        let mut pipeline = Pipeline::default();
        let mut step = Command::new();
        step.command("command1.sh");
        step.command("command2.sh");
        pipeline.steps.push(Step::Command(step));

        let yaml = r#"---
steps:
  - commands:
      - command1.sh
      - command2.sh
"#;
        assert_eq!(yaml, serde_yaml::to_string(&pipeline).unwrap());
    }

    #[test]
    fn from_yaml_multiple_commands() {
        let mut pipeline = Pipeline::default();
        let mut step = Command::new();
        step.command("command1.sh");
        step.command("command2.sh");
        pipeline.steps.push(Step::Command(step));

        let yaml = r#"---
steps:
  - commands:
    - "command1.sh"
    - "command2.sh"
"#;
        assert_eq!(pipeline, serde_yaml::from_str::<Pipeline>(yaml).unwrap());
    }
}
