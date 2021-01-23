#[cfg(test)]
extern crate quickcheck;
use crate::error::Error;
use anyhow::Result;
use std::env;
#[cfg(test)]
use std::path::PathBuf;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub enum Shell {
    Fish,
    Bash,
}

impl Default for Shell {
    fn default() -> Self {
        Shell::Bash
    }
}

impl FromStr for Shell {
    type Err = ();

    fn from_str(shell: &str) -> Result<Self, Self::Err> {
        match shell {
            "fish" => Ok(Shell::Fish),
            _ => Ok(Shell::default()),
        }
    }
}

const SHELL_ENV: &str = "SHELL";

pub fn get_shell(preselect: &Option<String>) -> Shell {
    match preselect {
        Some(shell) => Shell::from_str(shell.as_str()).unwrap(),
        _ => match env::var(SHELL_ENV) {
            Ok(shell) => Shell::from_str(shell.split("/").last().unwrap()).unwrap(),
            _ => Shell::default(),
        },
    }
}

pub fn export_string(shell: &Shell, var: &str, val: &String) -> Result<String, Error> {
    if var.is_empty() || val.is_empty() {
        Err(Error::InvalidSetEnv)
    } else {
        let string = match shell {
            Shell::Fish => format!("set -gx {} {};", var, val),
            Shell::Bash => format!("export {}={}", var, val),
        };
        Ok(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_from_string() {
        assert_eq!(Shell::from_str("bash").unwrap(), Shell::Bash);
        assert_eq!(Shell::from_str("fish").unwrap(), Shell::Fish);
    }

    #[quickcheck]
    fn string_default(s: String) -> bool {
        Shell::from_str(&s).unwrap() == Shell::default()
    }

    #[test]
    fn reading_from_env() {
        env::set_var(SHELL_ENV, "/usr/bin/fish");
        assert_eq!(get_shell(&None), Shell::Fish);
        env::set_var(SHELL_ENV, "/usr/bin/bash");
        assert_eq!(get_shell(&None), Shell::Bash);
        env::set_var(SHELL_ENV, "/usr/bin/zsh");
        assert_eq!(get_shell(&None), Shell::default());
    }

    #[quickcheck]
    fn env_default(path: PathBuf) -> bool {
        let path = path.into_os_string().into_string().unwrap();
        // env::set_var does not allow nul bytes
        let path = path.replace(char::from(0), "");
        env::set_var(SHELL_ENV, &path);
        get_shell(&None) == Shell::default()
    }

    #[test]
    fn prefer_preselect_over_env() {
        env::set_var(SHELL_ENV, "/usr/bin/fish");
        assert_eq!(get_shell(&Some(String::from("bash"))), Shell::Bash);
    }

    #[test]
    fn string_formatting() {
        assert_eq!(
            export_string(&Shell::Fish, "FOO", &String::from("bar")).unwrap(),
            "set -gx FOO bar;"
        );
        assert_eq!(
            export_string(&Shell::Bash, "FOO", &String::from("bar")).unwrap(),
            "export FOO=bar"
        );
    }
    #[test]
    fn prevent_invalid_setenv() {
        assert!(export_string(&Shell::default(), "", &String::from("")).is_err());
        assert!(export_string(&Shell::default(), "FOO", &String::from("")).is_err());
        assert!(export_string(&Shell::default(), "", &String::from("bar")).is_err());
    }
}
