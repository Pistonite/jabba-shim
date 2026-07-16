use std::path::{Path, PathBuf};

use cu::pre::*;
use reqwest::Client;

/// Download jabba executable to be packaged
#[derive(Debug, AsRef, clap::Parser)]
struct Args {
    /// Specify the upstream repo to fetch the binary
    #[clap(long, default_value = "https://github.com/Jabba-Team/jabba")]
    jabba_repo: String,

    /// Select the version of jabba to download
    #[clap(long)]
    jabba_version: Option<String>,

    /// Specify the output directory to put jabba-noshim binary.
    ///
    /// If not specified then the tool must be ran by cargo and the path will be inferred
    /// based on the current executable path
    #[clap(long)]
    output_dir: Option<String>,

    #[clap(flatten)]
    #[as_ref]
    common: cu::cli::Flags,
}
#[cu::cli]
async fn main(args: Args) -> cu::Result<()> {
    let repo = args.jabba_repo.trim_start().trim_end_matches('/');
    cu::info!("using jabba repo: {repo}");

    let output_dir = match args.output_dir {
        Some(dir) => PathBuf::from(dir),
        None => infer_output_dir()?,
    };

    let client = Client::new();

    cu::info!("fetching release info...");
    let release = fetch_release(&client, repo, args.jabba_version.as_deref()).await?;
    cu::info!("resolved version: {}", release.tag_name);

    download(&client, &release, &output_dir).await?;

    cu::info!(
        "downloaded jabba {} to {}",
        release.tag_name,
        output_dir.try_to_rel().display()
    );
    Ok(())
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    /// `sha256:<hex>` digest computed by GitHub, absent on older assets
    digest: Option<String>,
    browser_download_url: String,
}

/// Fetch the release info (with assets) for the requested version, or the latest tag
#[cu::context("failed to fetch release info")]
async fn fetch_release(
    client: &Client,
    repo: &str,
    requested: Option<&str>,
) -> cu::Result<Release> {
    // Turn https://github.com/OWNER/NAME into the OWNER/NAME slug for the REST API.
    let slug = repo
        .trim_end_matches(".git")
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let path = match requested {
        None | Some("latest") => "latest".to_string(),
        Some(version) => format!("tags/{}", version.trim().trim_start_matches('v')),
    };
    let url = format!("https://api.github.com/repos/{slug}/releases/{path}");

    cu::debug!("fetching release from {url}");
    let response = client
        .get(&url)
        .header("User-Agent", "jabba-download")
        .send()
        .await?;
    let response = cu::check!(response.error_for_status(), "http error")?;
    let release: Release = cu::check!(response.json().await, "failed to parse response json")?;

    Ok(release)
}

/// Download and put the binary in the output dir
#[cu::context("failed to download jabba binary")]
async fn download(client: &Client, release: &Release, output_dir: &Path) -> cu::Result<()> {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "windows",
        other => cu::bail!("unsupported OS for jabba: {other}"),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "x86" => "386",
        "arm" => "arm",
        other => cu::bail!("unsupported architecture for jabba: {other}"),
    };
    let ext = if cfg!(windows) { ".exe" } else { "" };
    let asset_name = format!("jabba-{}-{os}-{arch}{ext}", release.tag_name);

    let Some(asset) = release.assets.iter().find(|a| a.name == asset_name) else {
        cu::bail!("no asset named {asset_name} in the release");
    };

    cu::info!("downloading {}", asset.browser_download_url);
    let response = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "jabba-download")
        .send()
        .await?;
    let response = cu::check!(response.error_for_status(), "http error")?;
    let bytes = cu::check!(response.bytes().await, "failed to read response body bytes")?;

    // Verify the downloaded bytes against the digest advertised by the API, if any
    match asset.digest.as_deref() {
        Some(digest) => verify_sha256(&bytes, digest)?,
        None => cu::warn!("asset {asset_name} has no digest; skipping hash verification"),
    }

    let out_path = output_dir.join(format!("jabba-noshim{ext}"));
    cu::fs::write(&out_path, &bytes)?;
    let out_path = output_dir.join("jabba-version");
    cu::fs::write(&out_path, &release.tag_name)?;

    Ok(())
}

/// Verify `bytes` matches the `sha256:<hex>` digest from the release asset
fn verify_sha256(bytes: &[u8], digest: &str) -> cu::Result<()> {
    use sha2::{Digest as _, Sha256};

    let Some(expected) = digest.strip_prefix("sha256:") else {
        cu::bail!("unsupported digest format: {digest}");
    };
    let actual = hex::encode(Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        cu::bail!("hash mismatch: expected {expected}, got {actual}");
    }
    cu::info!("verified sha256: {actual}");
    Ok(())
}

/// Infer the output directory (`jabba-shim/bin`) from the cargo package dir.
///
/// This relies on being built by cargo, which is the required way to run this tool
/// when no explicit output dir is given.
fn infer_output_dir() -> cu::Result<PathBuf> {
    let manifest_dir = cu::check!(
        cu::env_var("CARGO_MANIFEST_DIR"),
        "the download tool must be ran with cargo if --output-dir is not specified to infer the output path"
    )?;
    let packages_dir = Path::new(&manifest_dir).parent_abs()?;
    let output_dir = cu::path!(packages_dir / "jabba-shim" / "bin");
    Ok(output_dir)
}
