use anyhow::{anyhow, Result};
use clap;
use clap::Clap;
use log;
use mobr::{git, mob, Config};

#[derive(Clap)]
#[clap(version = "1.0", author = "Paul")]
struct Opts {
    #[clap(long = "dry-run")]
    dry_run: bool,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    /// Setup mob in this repo
    #[clap(name = "init")]
    Init,

    /// Start mob session
    #[clap(name = "start")]
    Start,
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();

    let git = git::Git::new(opts.dry_run);

    let project_root = git.root_dir()?;
    if std::env::current_dir()? != project_root {
        std::env::set_current_dir(project_root)?;
    }

    if let SubCommand::Init = opts.subcmd {
        return init();
    }

    let mob_config = mob::Config::load().map_err(|err| {
        if let Some(_) = err.downcast_ref::<std::io::Error>() {
            return anyhow!("You must run \"mob init\" first");
        }
        err
    })?;
    // let mob_config = mob::Config::load().or_else(|err| {
    //     let create = dialoguer::Confirmation::new()
    //         .with_text(".mob missing or bad format, create now?")
    //         .default(true)
    //         .interact()?;
    //     if create {
    //         // Commit and push
    //         return mob::Config::init_config();
    //     }
    //     Err(err)
    // })?;

    let config = Config::from(&mob_config);

    // let cfg = config::Config::from_file(&opts.config)
    //     .with_context(|| format!("could not read config from {}", opts.config))?;

    log::debug!("Running command {:?}", opts.subcmd);
    match opts.subcmd {
        SubCommand::Init => init(),
        SubCommand::Start => start(git, config),
    }
}

fn init() -> Result<()> {
    mob::Config::init_config()?;
    log::warn!("don't forget to commit .mob");
    return Ok(());
}

fn start(git: git::Git, config: Config) -> Result<()> {
    if !git.tree_is_clean()? {
        log::error!("working tree is dirty");
        return Ok(());
    }

    let remote_mob_branch = config.remote_mob_branch.as_str();
    let remote_base_branch = config.remote_base_branch.as_str();

    let has_mob_branch = git.has_branch(config.mob_branch)?;
    let has_remote_mob_branch = git.has_branch(remote_mob_branch)?;

    git.run(&["fetch", "--prune"])?;
    git.run(&["pull", "--ff-only"])?;

    match (has_mob_branch, has_remote_mob_branch) {
        (true, true) => {
            log::info!("rejoining mob session");
            if !git.on_branch(config.mob_branch)? {
                git.run(&["branch", "-D", config.mob_branch])?;
                git.run(&["checkout", config.mob_branch])?;
                git.run(&[
                    "remote",
                    "--set-upstream",
                    remote_mob_branch,
                    config.mob_branch,
                ])?;
            }
        }
        (false, false) => {
            log::info!(
                "creating mob session on {} from {}",
                config.mob_branch,
                config.base_branch
            );
            git.run(&["checkout", config.base_branch])?;
            git.run(&["merge", remote_base_branch, "--ff-only"])?;
            git.run(&["branch", config.mob_branch])?;
            git.run(&["checkout", config.mob_branch])?;
            git.run(&[
                "push",
                "--no-verify",
                "--set-upstream",
                config.remote,
                config.mob_branch,
            ])?;
        }
        (_, _) => (),
    };
    Ok(())
}
