# jabba-shim

A shim for [jabba](https://github.com/Jabba-Team/jabba) that works-around issues
with shell integration.

the main difference is that instead of trying to edit `PATH` and `JAVA_HOME` at runtime,
the shim tool keeps an alias `current` that points to the current active installation globally.

The downside is that each shell session cannot have their own instance.
If you need the tool to automatically switch version based on the current directory,
then use the unshimmed version (upstream jabba) directly.

## Installation
Method 1: With `cargo-binstall` (recommended if you already have a Rust toolchain setup)
```
cargo binstall jabba-shim --git https://github.com/Pistonite/jabba-shim
```
This downloads the pre-built binaries from GitHub release

Method 2: Download manually from GitHub release and put the executable somewhere in your `PATH`

Method 3: Build from source (requires Rust toolchain)
```
git clone https://github.com/Pistonite/jabba-shim
cd jabba-shim
cargo run --bin jabba-download [-- --jabba-version 0.15.0]
cargo build --bin jabba --release
```
(use the `--jabba-version` flag if you need to specify a version other than the latest)
The output `jabba` binary is at `target/release/jabba[.exe]`. Put it somewhere in your `PATH`


## Configuration

**To customize the install location**: The shim uses the same environment variable `JABBA_HOME` to control where things get installed.
The underlying un-shimmed `jabba` binary is at `$JABBA_HOME/jabba-noshim.exe`.

If `JABBA_HOME` is not set then the tool assumes `~/.jabba` - the same behavior as the underlying program.

Additionally you need to set the following environment variables:
- Windows: use the control panel to set them so they work regardless of the shell/terminal.
  If you use the default `JABBA_HOME` then replace `%JABBA_HOME%` below with `%USERPROFILE%\.jabba`
    - Set `JAVA_HOME` to `%JABBA_HOME%\jdk\current`
    - Add to `PATH`: `%JAVA_HOME%\bin`
- Shells with `export`: replace `$JABBA_HOME` with `~/.jabba` if you use the default
  ```
  export JAVA_HOME="$JABBA_HOME/jdk/current"
  export PATH="$JAVA_HOME/bin:$PATH"
  ```

On Windows, you need to turn on Developer Mode in order to create symlinks without administrator privilege.


## Command reference/differences to `jabba`

- `alias`: same behavior (setting the `current` alias will change the active version like `use`)
- `check`: not supported - not needed
- `completion`: not supported
- `current`: instead of using `PATH` the shim reads the `current` alias.
- `deactivate`: not supported
- `help`: `-h` is short help and `--help` is long help, including differences between the shimmed and unshimmed (original) versions
- `init`: not supported.
- `install`: same behavior, plus if current alias is not set it will set to the installed version
- `link`: same behavior except `current` cannot be used as a link name
- `ls`: same behavior
- `ls-alias`: same behavior (`current` will be visible as an alias)
- `ls-remote`: same behavior
- `unalias`: same behavior except `current` cannot be unaliased
- `uninstall`: same behavior (note `current` will not automatically be unaliased)
- `unlink`: same behavior
- `use`: calls `alias current <version>` instead
  - the version is prefix-matched then substring matched by installed versions
- `which`: same behavior


