use anyhow::Result;
use clap::Parser;
use remotemob::{cmd, config, emoji_logger, git, session, session::Store};

#[derive(Parser)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!())]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Show status
    #[clap(name = "status")]
    Status(cmd::StatusOpts),

    /// Reorder drivers
    #[clap(name = "order")]
    Order,

    /// Clean up all mob related stuff from this repo
    #[clap(name = "clean")]
    Clean,

    /// Start mob session
    #[clap(name = "start")]
    Start(cmd::StartOpts),

    /// Finish turn and sync repo
    #[clap(name = "next")]
    Next,

    /// Stop session and stage all changes to commit
    #[clap(name = "done")]
    Done,
}

fn main() {
    emoji_logger::init("debug");
    if let Err(err) = run() {
        if log::log_enabled!(log::Level::Trace) {
            log::error!("{err:?}");
        } else {
            log::error!("{err}");
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let opts: Opts = Opts::parse();

    let config = config::load()?;

    let git = git::GitCommand::new(None, config.remote.clone())?;
    let store = session::SessionStore::new(&git);

    match opts.subcmd {
        SubCommand::Start(opts) => cmd::Start::new(&git, &store, opts, config).run()?,
        SubCommand::Next => cmd::Next::new(&git, &store, config).run()?,
        SubCommand::Done => cmd::Done::new(&git, &store, config).run()?,
        SubCommand::Clean => store.clean()?,
        SubCommand::Status(opts) => cmd::Status::new(opts, &store, config).run()?,
        SubCommand::Order => cmd::Order::new(&store).run()?,
    };
    Ok(())
}
