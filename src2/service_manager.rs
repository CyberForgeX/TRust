use std::process::{Command, Output};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Service operation '{action}' for '{service_name}' failed: {stderr}")]
    OperationFailed {
        action: String,
        service_name: String,
        stderr: String,
    },

    #[error("Unsupported operation '{action}' for the current OS")]
    UnsupportedOperation { action: String },
}

struct ServiceManager;

impl ServiceManager {
    pub fn service_action(service_name: &str, action: &str) -> Result<(), ServiceError> {
        #[cfg(target_os = "windows")]
        {
            match action {
                "start" | "stop" => Self::execute_sc_command(action, service_name),
                "enable" => Self::execute_sc_command("config", service_name, Some("start= auto")),
                "disable" => Self::execute_sc_command("config", service_name, Some("start= disabled")),
                _ => Err(ServiceError::UnsupportedOperation{ action: action.to_string() }),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self::execute_unix_service_command(action, service_name)
        }
    }

    #[cfg(target_os = "windows")]
    fn execute_sc_command(action: &str, service_name: &str, extra_arg: Option<&str> = None) -> Result<(), ServiceError> {
        let mut args = vec![action, service_name];
        if let Some(arg) = extra_arg {
            args.extend(arg.split_whitespace());
        }
        let output = Command::new("sc").args(&args).output()?;
        Self::handle_command_output(output, action, service_name)
    }

    #[cfg(not(target_os = "windows"))]
    fn execute_unix_service_command(action: &str, service_name: &str) -> Result<(), ServiceError> {
        let command = match action {
            "start" | "stop" => "systemctl",
            "enable" | "disable" => {
                if cfg!(target_os = "linux") {
                    "systemctl"
                } else {
                    // Handling for macOS and potentially other Unix-like systems could go here
                    return Err(ServiceError::UnsupportedOperation{ action: action.to_string() });
                }
            },
            _ => return Err(ServiceError::UnsupportedOperation{ action: action.to_string() }),
        };

        let args = [action, service_name];
        let output = Command::new(command).args(&args).output()?;
        Self::handle_command_output(output, action, service_name)
    }

    fn handle_command_output(output: Output, action: &str, service_name: &str) -> Result<(), ServiceError> {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ServiceError::OperationFailed {
                action: action.to_owned(),
                service_name: service_name.to_owned(),
                stderr: stderr.to_string(),
            })
        } else {
            Ok(())
        }
    }
}
