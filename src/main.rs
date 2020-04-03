use anyhow::Result;
use clap;
use clap::Clap;
use log;
use mobr::{git, settings, Config};

#[derive(Clap)]
#[clap(version = "1.0", author = "Paul")]
struct Opts {
    // #[clap(long = "dry-run")]
    // dry_run: bool,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    /// Setup mob in this repo
    #[clap(name = "init")]
    Init,

    /// Clean up mob related stuff from this repo
    #[clap(name = "clean")]
    Clean,

    /// Start mob session
    #[clap(name = "start")]
    Start,
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();

    let git = git::Git::new(None)?;

    if let SubCommand::Init = opts.subcmd {
        return init();
    }

    let config_store = settings::Service::new(&git);

    let mob_config = config_store.load()?;

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
        SubCommand::Clean => clean(git, config),
        SubCommand::Start => start(git, config),
    }
}

fn init() -> Result<()> {
    settings::Settings::init_config()?;
    log::warn!("don't forget to commit .mob");
    return Ok(());
}

fn clean(git: git::Git, _config: Config) -> Result<()> {
    // TODO clean up session branch
    // TODO this should probably live in the store
    git.run(&["branch", "-D", git::MOB_META_BRANCH])
}

fn start(git: git::Git, _config: Config) -> Result<()> {
    if !git.tree_is_clean()? {
        log::error!("working tree is dirty");
        return Ok(());
    }
    return Ok(());

    // let remote_mob_branch = config.remote_mob_branch.as_str();
    // let remote_base_branch = config.remote_base_branch.as_str();

    // let has_mob_branch = git.has_branch(config.mob_branch)?;
    // let has_remote_mob_branch = git.has_branch(remote_mob_branch)?;

    // git.run(&["fetch", "--prune"])?;
    // git.run(&["pull", "--ff-only"])?;

    // match (has_mob_branch, has_remote_mob_branch) {
    //     (true, true) => {
    //         log::info!("rejoining mob session");
    //         if !git.on_branch(config.mob_branch)? {
    //             git.run(&["branch", "-D", config.mob_branch])?;
    //             git.run(&["checkout", config.mob_branch])?;
    //             git.run(&[
    //                 "remote",
    //                 "--set-upstream",
    //                 remote_mob_branch,
    //                 config.mob_branch,
    //             ])?;
    //         }
    //     }
    //     (false, false) => {
    //         log::info!(
    //             "creating mob session on {} from {}",
    //             config.mob_branch,
    //             config.base_branch
    //         );
    //         git.run(&["checkout", config.base_branch])?;
    //         git.run(&["merge", remote_base_branch, "--ff-only"])?;
    //         git.run(&["branch", config.mob_branch])?;
    //         git.run(&["checkout", config.mob_branch])?;
    //         git.run(&[
    //             "push",
    //             "--no-verify",
    //             "--set-upstream",
    //             config.remote,
    //             config.mob_branch,
    //         ])?;
    //     }
    //     (_, _) => (),
    // };
    // Ok(())
}
