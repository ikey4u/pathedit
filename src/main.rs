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
    if reg_key.get_string("Path_Bakup").is_err() {
        reg_key.set_string("Path_Backup", &path).context("backup path environment")?;
    }
    let is_path_item_equal = |a: &str, b: &str| -> bool {
        let a = a.to_lowercase();
        let b = b.to_lowercase();
        if a == b {
            return true;
        }
        let a = a.trim_end_matches(&['/', '\\']);
        let b = b.trim_end_matches(&['/', '\\']);
        a == b
    };
    let is_path_item_exists = |dir: &str| -> bool {
        let dir = dir.to_lowercase();
        for item in path.split(";") {
            if is_path_item_equal(item, &dir) {
                return true;
            }
        }
        false
    };

    let updated_path = match cli.action {
        CliAction::Prepend { dir } => {
            if is_path_item_exists(&dir){
                return Ok(());
            }
            format!("{dir};{path}")
        },
        CliAction::Append { dir } => {
            if is_path_item_exists(&dir) {
                return Ok(());
            }
            format!("{path};{dir}")
        }
        CliAction::Remove { dir } => {
            if !is_path_item_exists(&dir) {
                return Ok(());
            }
            let mut items = vec![];
            for item in path.split(";") {
                if is_path_item_equal(item, &dir) {
                    continue;
                }
                if !item.trim().is_empty() {
                    items.push(item);
                }
            }
            items.join(";")
        }
        CliAction::List => {
            for item in path.split(";") {
                if item.trim().is_empty() {
                    continue;
                }
                println!("{item}");
            }
            return Ok(())
        }
    };
    reg_key.set_string("Path", &updated_path).context("update Path environment")?;

    refresh()?;
    
    Ok(())
} 