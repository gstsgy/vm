use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use vm_core::{add_category, add_version, current_version, link_active_bin, list_categories, list_versions, load, remove_category, remove_version, save, use_version};

#[derive(Parser)]
#[command(name = "vm", version, about = "轻量级版本管理工具（自定义类目 + 版本切换）")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 类目管理
    Category {
        #[command(subcommand)]
        action: CategoryCmd,
    },
    /// 版本管理
    Version {
        #[command(subcommand)]
        action: VersionCmd,
    },
    /// 切换到某类目的指定版本
    Use {
        category: String,
        version: String,
    },
    /// 显示当前激活版本
    Current {
        category: Option<String>,
    },
    /// 打印需要加入 PATH 的环境片段
    Env,
    /// 写入 PATH 配置到 shell profile
    Init {
        #[arg(long, value_enum, default_value = "zsh")]
        shell: Shell,
    },
}

#[derive(Subcommand)]
enum CategoryCmd {
    /// 新增类目
    Add {
        name: String,
        #[arg(long)]
        desc: Option<String>,
    },
    /// 删除类目
    Remove {
        name: String,
    },
    /// 列出全部类目
    List,
}

#[derive(Subcommand)]
enum VersionCmd {
    /// 添加版本（path 为安装根目录）
    Add {
        category: String,
        version: String,
        path: String,
        /// 显式指定 bin 目录（默认自动探测 <path>/bin 或 <path>）
        #[arg(long)]
        bin: Option<String>,
    },
    /// 删除版本
    Remove {
        category: String,
        version: String,
    },
    /// 列出版本
    List {
        category: Option<String>,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum Shell {
    Zsh,
    Bash,
    PowerShell,
    Cmd,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Category { action } => match action {
            CategoryCmd::Add { name, desc } => {
                let mut cfg = load()?;
                add_category(&mut cfg, &name, &desc.unwrap_or_default())?;
                save(&cfg)?;
                println!("added category '{name}'");
            }
            CategoryCmd::Remove { name } => {
                let mut cfg = load()?;
                remove_category(&mut cfg, &name)?;
                save(&cfg)?;
                println!("removed category '{name}'");
            }
            CategoryCmd::List => {
                let cfg = load()?;
                for c in list_categories(&cfg) {
                    println!("{c}");
                }
            }
        },
        Command::Version { action } => match action {
            VersionCmd::Add {
                category,
                version,
                path,
                bin,
            } => {
                let mut cfg = load()?;
                add_version(&mut cfg, &category, &version, &path, bin)?;
                save(&cfg)?;
                println!("added version '{version}' to '{category}'");
            }
            VersionCmd::Remove { category, version } => {
                let mut cfg = load()?;
                remove_version(&mut cfg, &category, &version)?;
                save(&cfg)?;
                println!("removed version '{version}' from '{category}'");
            }
            VersionCmd::List { category } => {
                let cfg = load()?;
                match category {
                    Some(c) => {
                        let cur = current_version(&cfg, &c)?;
                        for v in list_versions(&cfg, &c)? {
                            let mark = if cur.as_deref() == Some(&v) { " *" } else { "" };
                            println!("{v}{mark}");
                        }
                    }
                    None => {
                        for c in list_categories(&cfg) {
                            println!("[{c}]");
                            let cur = current_version(&cfg, c)?;
                            for v in list_versions(&cfg, c)? {
                                let mark = if cur.as_deref() == Some(&v) { " *" } else { "" };
                                println!("  {v}{mark}");
                            }
                        }
                    }
                }
            }
        },
        Command::Use { category, version } => {
            let mut cfg = load()?;
            use_version(&mut cfg, &category, &version)?;
            link_active_bin(&cfg, &category)?;
            save(&cfg)?;
            println!("switched '{category}' -> '{version}'");
        }
        Command::Current { category } => {
            let cfg = load()?;
            match category {
                Some(c) => match current_version(&cfg, &c)? {
                    Some(v) => println!("{v}"),
                    None => println!("(no active version for '{c}')"),
                },
                None => {
                    for c in list_categories(&cfg) {
                        let v = current_version(&cfg, c)?;
                        println!("{c}: {}", v.unwrap_or_else(|| "(none)".into()));
                    }
                }
            }
        }
        Command::Env => {
            let dir = vm_core::config_dir();
            println!("# 将下面内容加入你的 shell profile，并把每个类目的 bin 加入 PATH：");
            for c in list_categories(&load()?) {
                println!("export PATH=\"{}:{}\"", dir.join(c).join("bin").display(), "$PATH");
            }
        }
        Command::Init { shell } => {
            let dir = vm_core::config_dir();
            let snippet = match shell {
                Shell::Zsh | Shell::Bash => {
                    let mut s = String::new();
                    for c in list_categories(&load()?) {
                        s.push_str(&format!(
                            "export PATH=\"{}:$PATH\"\n",
                            dir.join(c).join("bin").display()
                        ));
                    }
                    s
                }
                Shell::PowerShell => {
                    let mut s = String::new();
                    for c in list_categories(&load()?) {
                        s.push_str(&format!(
                            "$env:PATH = \"{};\" + $env:PATH\n",
                            dir.join(c).join("bin").display()
                        ));
                    }
                    s
                }
                Shell::Cmd => {
                    let mut s = String::new();
                    for c in list_categories(&load()?) {
                        s.push_str(&format!(
                            "set PATH={};%PATH%\n",
                            dir.join(c).join("bin").display()
                        ));
                    }
                    s
                }
            };
            println!("# 请将以下内容手动追加到对应 profile（自动写入需谨慎，此处仅打印）：\n{snippet}");
        }
    }
    Ok(())
}
