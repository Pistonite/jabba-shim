use cu::pre::*;

// this is a mapping of jabba's commands, see jabba --help

#[derive(clap::Subcommand)]
pub enum Command {
    /// Resolve or update an alias
    ///
    /// SHIM: Setting the `current` alias has the same effect as `use`
    Alias {
        /// Name of the alias to create or inspect
        name: String,
        /// Target version for this alias, omit to display the version bound to an alias
        version: Option<String>,
    },

    /// Display currently 'use'ed version
    ///
    /// SHIM: Reads the `current` alias instead of PATH
    Current,

    #[clap(verbatim_doc_comment)]
    /// Download and install JDK
    ///
    /// Examples:
    /// - jabba install 1.8
    /// - jabba install ~1.8.73 # same as ">=1.8.73 <1.9.0"
    /// - jabba install 1.8.73=dmg+http://.../jdk-9-ea+110_osx-x64_bin.dmg
    Install {
        /// The version to install
        version: String,
        /// Custom destination (if outside of $JABBA_HOME/jdk then it's considered unmanaged, i.e.e
        /// not available to jabba ls, use, etc.. unless jabba linked)
        #[clap(short, long)]
        output: Option<String>,
    },

    /// Resolve or update a link
    ///
    /// SHIM: `current` cannot be used as the link name
    Link {
        /// Name of the link to create or inspect
        name: String,
        /// Target path of the JDK to link, omit to display the current link target
        path: Option<String>,
    },

    /// List installed versions
    Ls {
        /// Part of the version to trim to
        #[clap(long)]
        latest: Option<Latest>,
    },
    /// List configured aliases
    LsAlias,
    /// List remote versiosn available for install
    LsRemote {
        /// Architecture
        #[clap(long, default_value = CURR_ARCH)]
        arch: Arch,
        /// Part of the version to trim to
        #[clap(long)]
        latest: Option<Latest>,
        /// Operating System
        #[clap(long, default_value = CURR_OS)]
        os: Os,
    },

    /// Delete an alias
    ///
    /// SHIM: `current` cannot be deleted
    Unalias {
        /// Name of the alias to delete
        name: String,
    },

    /// Uninstall JDK
    Uninstall {
        /// Version to uninstall
        version: String,
    },

    /// Delete a link
    Unlink {
        /// Name of the link to delete
        name: String,
    },

    #[clap(verbatim_doc_comment)]
    /// Modify the `current` alias to use specific JDK
    ///
    /// SHIM: modifies `current` alias instead of PATH/JAVA_HOME
    ///
    /// Examples:
    /// - jabba use 1.8
    /// - jabba use ~1.8.73 # same as ">=1.8.73 <1.9.0"
    Use {
        /// Version to use
        version: String,
    },

    /// Display path to installed JDK
    Which {
        /// Version to display
        version: String,

        /// Account for platform differences so that value could be used as JAVA_HOME (e.g. append "/Contents/Home" on macOS)
        #[clap(long)]
        home: bool,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Latest {
    Major,
    Minor,
    Patch,
}
impl Latest {
    pub const fn as_str(self) -> &'static str {
        match self {
            Latest::Major => "major",
            Latest::Minor => "minor",
            Latest::Patch => "patch",
        }
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Arch {
    Amd64,
    _386,
}
impl Arch {
    pub const fn as_str(self) -> &'static str {
        match self {
            Arch::_386 => "386",
            Arch::Amd64 => "amd64",
        }
    }
}
#[cfg(target_pointer_width = "64")]
static CURR_ARCH: &str = Arch::Amd64.as_str();
#[cfg(target_pointer_width = "32")]
static CURR_ARCH: &str = Arch::_386.as_str();

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Os {
    Darwin,
    Linux,
    Windows,
}
impl Os {
    pub const fn as_str(self) -> &'static str {
        match self {
            Os::Darwin => "darwin",
            Os::Linux => "linux",
            Os::Windows => "windows",
        }
    }
}
#[cfg(target_os = "macos")]
static CURR_OS: &str = Os::Darwin.as_str();
#[cfg(windows)]
static CURR_OS: &str = Os::Windows.as_str();
#[cfg(target_os = "linux")]
static CURR_OS: &str = Os::Linux.as_str();
