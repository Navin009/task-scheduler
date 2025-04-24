use std::{
    process::{Command, Output},
    time::Duration,
};
use tokio::time;
use tracing::{error, info};

use crate::error::Error;

pub struct ProcessManager {
    pub timeout: Duration,
    pub max_memory_mb: u64,
    pub max_cpu_percent: u32,
}

impl ProcessManager {
    pub fn new(timeout: Duration, max_memory_mb: u64, max_cpu_percent: u32) -> Self {
        Self {
            timeout,
            max_memory_mb,
            max_cpu_percent,
        }
    }

    pub async fn execute_command(
        &self,
        command: &str,
        args: &[String],
        env_vars: &[(String, String)],
    ) -> Result<Output, Error> {
        let mut cmd = Command::new(command);
        cmd.args(args);

        // Set environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Set resource limits
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                // Set memory limit
                if self.max_memory_mb > 0 {
                    let rlimit = libc::rlimit {
                        rlim_cur: self.max_memory_mb * 1024 * 1024,
                        rlim_max: self.max_memory_mb * 1024 * 1024,
                    };
                    unsafe {
                        libc::setrlimit(libc::RLIMIT_AS, &rlimit);
                    }
                }
                Ok(())
            });
        }

        info!("Executing command: {} {:?}", command, args);

        // Execute with timeout
        let output = time::timeout(self.timeout, async {
            cmd.output()
                .map_err(|e| Error::Process(format!("Failed to execute command: {}", e)))
        })
        .await
        .map_err(|_| Error::Timeout(format!("Command timed out after {:?}", self.timeout)))??;

        if !output.status.success() {
            error!(
                "Command failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(Error::Process(format!(
                "Command failed with status {}",
                output.status
            )));
        }

        Ok(output)
    }

    pub fn validate_resources(&self) -> Result<(), Error> {
        // Check if system has enough resources
        // This is a simplified implementation
        if self.max_memory_mb > 0 {
            let total_memory = sys_info::mem_info()
                .map_err(|e| Error::ResourceLimit(format!("Failed to get memory info: {}", e)))?;

            if total_memory.total < self.max_memory_mb * 1024 {
                return Err(Error::ResourceLimit(format!(
                    "Insufficient system memory. Required: {}MB, Available: {}MB",
                    self.max_memory_mb,
                    total_memory.total / 1024
                )));
            }
        }

        Ok(())
    }
}
