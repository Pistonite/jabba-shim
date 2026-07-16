use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use cu::pre::*;

static JABBA_VERSION: &str = include_str!("../bin/jabba-version");
pub fn static_version() -> &'static str {
    JABBA_VERSION
}
pub struct Noshim {
    home: PathBuf,
    executable: PathBuf,
}
impl Noshim {
    #[cu::context("failed to resolve jabba-noshim executable")]
    pub fn resolve() -> cu::Result<Self> {
        let home_env = cu::env_var("JABBA_HOME").unwrap_or_default();
        let home = if home_env.is_empty() {
            cu::check!(
                std::env::home_dir(),
                "failed to find default home, please set JABBA_HOME environment variable"
            )?
        } else {
            home_env.into()
        };
        cu::debug!("JABBA_HOME: {}", home.display());
        let Ok(executable) = home.join(Self::executable_name()).normalize_executable() else {
            cu::debug!("cannot find jabba-noshim!");
            return Self::unpack(&home);
        };
        let Ok(actual_version) = cu::fs::read_string(home.join("jabba-version")) else {
            cu::debug!("cannot find jabba-version!");
            return Self::unpack(&home);
        };
        if actual_version.trim() != JABBA_VERSION.trim() {
            cu::debug!(
                "jabba-version mismatch (blessed: {JABBA_VERSION}, actual: {actual_version})"
            );
            return Self::unpack(&home);
        }
        Ok(Self {
            home: home.normalize_exists()?,
            executable,
        })
    }

    #[cu::context("failed to unpack jabba-noshim executable")]
    fn unpack(home: &Path) -> cu::Result<Self> {
        cu::debug!("unpacking jabba-noshim...");
        #[cfg(windows)]
        let bytes = include_bytes!("../bin/jabba-noshim.exe");
        #[cfg(not(windows))]
        let bytes = include_bytes!("../bin/jabba-noshim");
        let executable = home.join(Self::executable_name());
        cu::fs::write(&executable, bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            cu::check!(
                std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)),
                "failed to set executable permission"
            )?;
        }
        cu::fs::write(home.join("jabba-version"), JABBA_VERSION)?;
        Ok(Self {
            home: home.normalize_exists()?,
            executable: executable.normalize_executable()?,
        })
    }

    pub fn version(&self) -> cu::Result<String> {
        let (child, out) = self
            .executable
            .command()
            .env("JABBA_HOME", &self.home)
            .arg("--version")
            .stdout(cu::pio::string())
            .stderr_inherit()
            .stdin_inherit()
            .spawn()?;
        child.wait_nz()?;
        Ok(out.join()??.trim().to_string())
    }

    pub fn current(&self) -> cu::Result<String> {
        let (child, out) = self
            .executable
            .command()
            .env("JABBA_HOME", &self.home)
            .args(["alias", "current"])
            .stdout(cu::pio::string())
            .stderr_inherit()
            .stdin_inherit()
            .spawn()?;
        child.wait_nz()?;
        Ok(out.join()??.trim().to_string())
    }
    pub fn installed_versions(&self) -> cu::Result<Vec<String>> {
        let (child, out) = self
            .executable
            .command()
            .env("JABBA_HOME", &self.home)
            .arg("ls")
            .stdout(cu::pio::string())
            .stderr_inherit()
            .stdin_inherit()
            .spawn()?;
        child.wait_nz()?;
        let versions = out.join()??.lines().map(|x| x.trim().to_string()).collect();
        Ok(versions)
    }

    pub fn run<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(&self, args: I) -> cu::Result<()> {
        self.executable
            .command()
            .env("JABBA_HOME", &self.home)
            .args(args)
            .stdoe(cu::lv::P)
            .stdin_inherit()
            .wait_nz()?;
        Ok(())
    }
    pub fn run_progress<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        args: I,
        progress_name: &str,
    ) -> cu::Result<()> {
        let (child, spinner) = self
            .executable
            .command()
            .env("JABBA_HOME", &self.home)
            .args(args)
            .stderr(cu::lv::P)
            .stdout(
                cu::pio::spinner(progress_name)
                    .debug()
                    .configure_spinner(|x| x.keep(false)),
            )
            .stdin_inherit()
            .spawn()?;
        child.wait_nz()?;
        spinner.done();

        Ok(())
    }
    pub fn run_raw<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(&self, args: I) -> cu::Result<()> {
        cu::co::run(async move {
            let (child, mut out, mut err) = self
                .executable
                .command()
                .env("JABBA_HOME", &self.home)
                .args(args)
                .stdout(cu::pio::co_lines())
                .stderr(cu::pio::co_lines())
                .stdin_inherit()
                .co_spawn()
                .await?;
            let out_handle = cu::co::spawn(async move {
                while let Some(Ok(line)) = out.next().await {
                    println!("{line}");
                }
            });
            let err_handle = cu::co::spawn(async move {
                while let Some(Ok(line)) = err.next().await {
                    println!("{line}");
                }
            });
            let result = child.co_wait_nz().await;
            let _ = out_handle.co_join_maybe_aborted().await;
            let _ = err_handle.co_join_maybe_aborted().await;
            result
        })
    }

    fn executable_name() -> &'static str {
        if cfg!(windows) {
            "jabba-noshim.exe"
        } else {
            "jabba-noshim"
        }
    }
}
