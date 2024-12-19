use std::collections::HashSet;

use anyhow::Context;
use windows_registry::*;
use clap::{Subcommand, Parser};

pub type Result<T> = anyhow::Result<T, anyhow::Error>;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    action: CliAction,
}

#[derive(Subcommand)]
pub enum CliAction {
    Prepend { dir: String },
    Append { dir: String },
    Remove { dir: String },
    List,
}

fn refresh() -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutA, HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG};
    use windows::Win32::Foundation::LPARAM;
    unsafe {
        let param = std::ffi::CString::new(r"Environment")?;
        SendMessageTimeoutA(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            None,
            LPARAM(param.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            5000,
            None,
        );
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let reg_key_str = r"Environment";
    let reg_key = CURRENT_USER.create(reg_key_str).context(format!("open register entry: {reg_key_str}"))?;

    let path = reg_key.get_string("Path")?; 
    let mut path_items = HashSet::new();
    for path in path.split(";") {
        path_items.insert(path.to_lowercase());
    }

    if reg_key.get_string("Path_Bakup").is_err() {
        reg_key.set_string("Path_Backup", &path).context("backup path environment")?;
    }

    let updated_path = match cli.action {
        CliAction::Prepend { dir } => {
            if path_items.contains(&dir.to_lowercase()) {
                return Ok(());
            }
            format!("{dir};{path}")
        },
        CliAction::Append { dir } => {
            if path_items.contains(&dir.to_lowercase()) {
                return Ok(());
            }
            format!("{path};{dir}")
        }
        CliAction::Remove { dir } => {
            if !path_items.contains(&dir.to_lowercase()) {
                return Ok(());
            }
            path.replace(&dir, "").replace(&format!("{dir};"), "").replace(r"{dir}\", "").replace(r"{dir}\;", "")
        }
        CliAction::List => {
            for path in path.split(";") {
                println!("{path}");
            }
            return Ok(())
        }
    };
    reg_key.set_string("Path", &updated_path).context("update Path environment")?;

    refresh()?;
    
    Ok(())
} 