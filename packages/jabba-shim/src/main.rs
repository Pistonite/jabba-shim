use cu::pre::*;

mod noshim;
use noshim::Noshim;
mod command;
use command::Command;

/// Java Version Manager (https://github.com/Jabba-Team/jabba) - Shimmed
#[derive(clap::Parser, AsRef)]
#[clap(bin_name = "jabba")]
pub struct Args {
    #[clap(subcommand)]
    command: Option<Command>,

    /// Show version of jabba
    ///
    /// Run with -vV to show the version for jabba-shim as well
    #[clap(short = 'V', long, action(clap::ArgAction::Count))]
    version: u8,

    #[clap(flatten)]
    #[as_ref]
    flags: cu::cli::Flags,
}
impl Args {
    pub fn preprocess(&mut self) {
        if self.version > 0 {
            self.version = 1;
            if self.flags.verbose > 0 {
                self.version += 1;
            }
            self.flags.quiet = self.flags.verbose + 2;
        }
    }
}

#[cu::cli(preprocess = Args::preprocess)]
fn main(args: Args) -> cu::Result<()> {
    // with the exception of install, the commands are all short-lived
    // so printing time is not useful
    cu::lv::disable_print_time();

    if args.version > 0 {
        if args.version > 1 {
            println!("jabba-shim {}", env!("CARGO_PKG_VERSION"));
            println!("jabba-noshim {}", noshim::static_version());
        }
        let noshim = Noshim::resolve()?;
        println!("{}", noshim.version()?);
        return Ok(());
    }

    let Some(command) = args.command else {
        cu::cli::print_help::<Args>(false);
        return Ok(());
    };

    let noshim = Noshim::resolve()?;

    match command {
        Command::Alias { name, version } => match version {
            None => noshim.run_raw(["alias", &name]),
            Some(version) => {
                if name.trim().to_lowercase() == "current" {
                    cu::hint!("'use' is the shorthand for 'alias current'");
                    run_use(&noshim, version)
                } else {
                    noshim.run(["alias", &name, &version])
                }
            }
        },
        Command::Current => noshim.run_raw(["alias", "current"]),
        Command::Install { version, output } => match output {
            None => {
                noshim.run_progress(["install", &version], "Downloading")?;
                if let Ok(current) = noshim.current()
                    && current.is_empty()
                {
                    noshim.run(["alias", "current", &version])?;
                }
                Ok(())
            }
            Some(output) => {
                noshim.run_progress(["install", &version, "--output", &output], "Downloading")
            }
        },
        Command::Link { name, path } => {
            if name.trim() == "current" {
                cu::bail!("'current' is a reserved name");
            }
            match path {
                None => noshim.run_raw(["link", &name]),
                Some(path) => noshim.run(["link", &name, &path]),
            }
        }
        Command::Ls { latest } => match latest {
            None => noshim.run_raw(["ls"]),
            Some(latest) => noshim.run_raw(["ls", "--latest", latest.as_str()]),
        },
        Command::LsAlias => noshim.run_raw(["ls-alias"]),
        Command::LsRemote { arch, latest, os } => match latest {
            None => noshim.run_raw(["ls-remote", "--arch", arch.as_str(), "--os", os.as_str()]),
            Some(latest) => noshim.run_raw([
                "ls-remote",
                "--arch",
                arch.as_str(),
                "--os",
                os.as_str(),
                "--latest",
                latest.as_str(),
            ]),
        },
        Command::Unalias { name } => {
            if name.trim() == "current" {
                cu::bail!("'current' is a reserved name that cannot be unaliased");
            }
            noshim.run(["unalias", &name])
        }
        Command::Uninstall { version } => noshim.run(["uninstall", &version]),
        Command::Unlink { name } => noshim.run(["unlink", &name]),
        Command::Use { version } => run_use(&noshim, version),
        Command::Which { version, home } => {
            if home {
                noshim.run_raw(["which", &version, "--home"])
            } else {
                noshim.run_raw(["which", &version])
            }
        }
    }
}

fn run_use(noshim: &Noshim, mut version: String) -> cu::Result<()> {
    version.make_ascii_lowercase();
    let version = version.trim();
    let installed: Vec<_> = noshim
        .installed_versions()?
        .into_iter()
        .map(|mut x| {
            x.make_ascii_lowercase();
            x
        })
        .collect();
    // try prefix matching
    let mut matched = vec![];
    for v in &installed {
        if v.starts_with(version) {
            matched.push(v);
        }
    }
    if matched.len() > 1 {
        cu::bail!("'{version}' is ambiguous, matched: {matched:#?}");
    }
    if matched.len() == 1 {
        return noshim.run(["alias", "current", matched[0]]);
    }
    // try substring
    matched.clear();
    for v in &installed {
        if v.contains(version) {
            matched.push(v);
        }
    }
    if matched.len() > 1 {
        cu::bail!("'{version}' is ambiguous, matched: {matched:#?}");
    }
    if matched.len() == 1 {
        return noshim.run(["alias", "current", matched[0]]);
    }
    cu::bail!("'{version}' is not installed");
}
