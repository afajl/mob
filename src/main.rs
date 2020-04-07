use anyhow::Result;
use clap;
use clap::Clap;
use log;
use mobr::{cmd, config, git, session, settings};

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
    // Start mob session
    #[clap(name = "start")]
    Start(cmd::StartOpts),
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opts: Opts = Opts::parse();

    let git = git::GitCommand::new(None)?;
    let settings_service = settings::StoreService::new(&git);
    let mut session_service = session::StoreService::new(&git);
    let config = config::load()?;

    // let config = Config::from(&settings);
    // let tools = cmd::Tools::new(&git, settings_service, session_service);

    log::debug!("Running command {:?}", opts.subcmd);
    match opts.subcmd {
        SubCommand::Init => Ok(()),
        SubCommand::Clean => Ok(()),
        SubCommand::Start(_start) => {
            cmd::Start::new(config, &git, &settings_service, &mut session_service).run()
        }
    }
}

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
