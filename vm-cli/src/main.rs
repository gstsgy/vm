use std::process::ExitCode;

use clap::{Parser, Subcommand};
use vm_core::{
    add_category, add_version, current_version, link_active_bin, list_categories, list_versions,
    load, remove_category, remove_version, save, use_version,
};

#[derive(Parser)]
#[command(name = "vm", version, about = "轻量级版本管理器：自定义类目 + 版本切换")]
struct Cli {
    /// 类目名。仅传类目 = 显示当前版本；传「类目 版本」= 切换版本
    category: Option<String>,
    /// 版本号（与类目一起传入即切换）
    #[arg(id = "ver", value_name = "VERSION")]
    version: Option<String>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// 列出全部类目（无参数）或某类目下的版本
    List {
        category: Option<String>,
    },
    /// 新增版本：vm add <类目> <版本> <安装路径>
    Add {
        category: String,
        version: String,
        path: String,
        /// 显式指定 bin 目录（默认自动探测 <path>/bin 或 <path>）
        #[arg(long)]
        bin: Option<String>,
    },
    /// 新增类目：vm addc <类目> [-d "说明"]
    Addc {
        category: String,
        /// 类目说明
        #[arg(short = 'd', long, default_value = "")]
        desc: String,
    },
    /// 删除类目（不指定版本）或某个版本
    Remove {
        category: String,
        version: Option<String>,
    },
    /// 打印需要加入 PATH 的环境片段
    Env,
    /// 打印可追加到 shell profile 的 PATH 配置
    Init,
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
    if let Some(cmd) = cli.command {
        return run_cmd(cmd);
    }
    match (cli.category, cli.version) {
        // `vm`：列出所有类目及当前版本
        (None, None) => {
            let cfg = load()?;
            if list_categories(&cfg).is_empty() {
                println!("尚无类目，使用 `vm addc <类目> -d \"说明\"` 创建");
            }
            for c in list_categories(&cfg) {
                let cur = current_version(&cfg, c)?;
                println!("{c}: {}", cur.unwrap_or_else(|| "(无激活版本)".into()));
            }
        }
        // `vm <类目>`：显示当前版本
        (Some(cat), None) => {
            let cfg = load()?;
            match current_version(&cfg, &cat)? {
                Some(v) => println!("{v}"),
                None => println!("(类目 '{cat}' 无激活版本)"),
            }
        }
        // 不可能出现：无类目却有版本
        (None, Some(_)) => unreachable!(),
        // `vm <类目> <版本>`：切换版本
        (Some(cat), Some(ver)) => {
            let mut cfg = load()?;
            use_version(&mut cfg, &cat, &ver)?;
            link_active_bin(&cfg, &cat)?;
            save(&cfg)?;
            println!("switched '{cat}' -> '{ver}'");
        }
    }
    Ok(())
}

fn run_cmd(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::List { category } => {
            let cfg = load()?;
            match category {
                None => {
                    for c in list_categories(&cfg) {
                        let cur = current_version(&cfg, c)?;
                        println!("{c}: {}", cur.unwrap_or_else(|| "(无激活版本)".into()));
                    }
                }
                Some(c) => {
                    let cur = current_version(&cfg, &c)?;
                    for v in list_versions(&cfg, &c)? {
                        let mark = if cur.as_deref() == Some(&v) { " *" } else { "" };
                        println!("{v}{mark}");
                    }
                }
            }
        }
        Command::Add {
            category,
            version,
            path,
            bin,
        } => {
            let mut cfg = load()?;
            if !cfg.categories.contains_key(&category) {
                anyhow::bail!("类目 '{category}' 不存在，请先执行 `vm addc {category}` 创建");
            }
            add_version(&mut cfg, &category, &version, &path, bin)?;
            save(&cfg)?;
            println!("added version '{version}' to '{category}'");
        }
        Command::Addc { category, desc } => {
            let mut cfg = load()?;
            if cfg.categories.contains_key(&category) {
                anyhow::bail!("类目 '{category}' 已存在");
            }
            add_category(&mut cfg, &category, &desc)?;
            save(&cfg)?;
            println!("added category '{category}'");
        }
        Command::Remove { category, version } => {
            let mut cfg = load()?;
            match version {
                None => {
                    remove_category(&mut cfg, &category)?;
                    save(&cfg)?;
                    println!("removed category '{category}'");
                }
                Some(v) => {
                    remove_version(&mut cfg, &category, &v)?;
                    save(&cfg)?;
                    println!("removed version '{v}' from '{category}'");
                }
            }
        }
        Command::Env => {
            let dir = vm_core::config_dir();
            for c in list_categories(&load()?) {
                println!("export PATH=\"{}:$PATH\"", dir.join(c).join("bin").display());
            }
        }
        Command::Init => {
            let dir = vm_core::config_dir();
            let mut snippet = String::new();
            for c in list_categories(&load()?) {
                snippet.push_str(&format!(
                    "export PATH=\"{}:$PATH\"\n",
                    dir.join(c).join("bin").display()
                ));
            }
            println!("# 请将以下内容追加到你的 shell profile（如 ~/.zshrc）：\n{snippet}");
        }
    }
    Ok(())
}
