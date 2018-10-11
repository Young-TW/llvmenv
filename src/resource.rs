//! Get LLVM/Clang source

use failure::err_msg;
use reqwest;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use error::*;

#[derive(Debug)]
pub enum Resource {
    Svn {
        url: String,
        branch: Option<String>, // FIXME branch for SVN is not supported
    },
    Git {
        url: String,
        branch: Option<String>,
    },
    Tar {
        url: String,
    },
}

impl Resource {
    pub fn download(&self, dest: &Path) -> Result<()> {
        if !dest.is_dir() {
            return Err(err_msg("Download destination must be a directory"));
        }
        match self {
            Resource::Svn { url, .. } => Command::new("svn")
                .args(&["co", url.as_str()])
                .arg(dest)
                .check_run()?,
            Resource::Git { url, branch } => {
                info!("Git clone {}", url);
                let mut git = Command::new("git");
                git.args(&["clone", url.as_str()]);
                if let Some(branch) = branch {
                    git.args(&["-b", branch]);
                }
                git.current_dir(dest).check_run()?;
            }
            Resource::Tar { url } => {
                let path = download_file(url, &dest)?;
                Command::new("tar")
                    .arg("xf")
                    .arg(path.file_name().unwrap())
                    .current_dir(dest)
                    .check_run()?;
            }
        }
        Ok(())
    }
}

fn get_filename_from_url(url_str: &str) -> Result<String> {
    let url = ::url::Url::parse(url_str)?;
    let seg = url.path_segments().ok_or(err_msg("URL parse failed"))?;
    let filename = seg.last().ok_or(err_msg("URL is invalid"))?;
    Ok(filename.to_string())
}

fn download_file(url: &str, temp: &Path) -> Result<PathBuf> {
    info!("Download: {}", url);
    let mut req = reqwest::get(url)?;
    let out = if temp.is_dir() {
        let name = get_filename_from_url(url)?;
        temp.join(name)
    } else {
        temp.into()
    };
    let mut f = fs::File::create(&out)?;
    req.copy_to(&mut f)?;
    f.sync_all()?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    // Test donwloading this repo
    #[test]
    fn test_git_donwload() -> Result<()> {
        let git = Resource::Git("http://github.com/termoshtt/llvmenv".into(), None);
        let tmp_dir = TempDir::new("git_download_test")?;
        git.download(tmp_dir.path())?;
        let top = tmp_dir.path().join("llvmenv");
        assert!(top.is_dir());
        Ok(())
    }

    #[test]
    fn test_tar_download() -> Result<()> {
        let url = "https://github.com/termoshtt/llvmenv/archive/0.1.10.tar.gz".into();
        let tar = Resource::Tar(url);
        let tmp_dir = TempDir::new("tar_download_test")?;
        tar.download(tmp_dir.path())?;
        let top = tmp_dir.path().join("llvmenv-0.1.10");
        assert!(top.is_dir());
        Ok(())
    }

    #[test]
    fn test_get_filename_from_url() {
        let url = "http://releases.llvm.org/6.0.1/llvm-6.0.1.src.tar.xz".into();
        assert_eq!(
            get_filename_from_url(&url).unwrap(),
            "llvm-6.0.1.src.tar.xz"
        );
    }

}
