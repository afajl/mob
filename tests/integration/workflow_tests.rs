use crate::fixtures::setup_repos;
use remotemob::prompt::MockPrompter;
use remotemob::session::Store;
use remotemob::{cmd, config, git, session};

fn create_test_config(name: &str) -> config::Config {
    config::Config {
        name: name.to_string(),
        remote: "origin".to_string(),
        hooks: config::Hooks::new(None),
    }
}

#[test]
fn test_mob_start_creates_session() {
    let repos = setup_repos(1);
    let alice = repos.alice();

    let config = create_test_config("alice");

    // Setup mock prompter - using defaults for all prompts
    let prompter = MockPrompter::new()
        .with_string("mob-session") // feature branch
        .with_string("main") // base branch
        .with_number(10) // work duration
        .with_string("mob sync"); // commit message

    // Create git and store
    let git = git::GitCommand::new(Some(alice.path.clone()), "origin".to_string()).unwrap();
    let store = session::SessionStore::new(&git);

    // Run start command
    let opts = cmd::StartOpts { minutes: Some(0) };
    let start = cmd::Start::new(&git, &store, &prompter, opts, config);
    start.run().unwrap();

    // Verify session was created
    let session = store.load().unwrap();
    assert!(matches!(
        session.state,
        session::State::Working { driver } if driver == "alice"
    ));
}

#[test]
fn test_start_with_dirty_tree_stash() {
    let repos = setup_repos(1);
    let alice = repos.alice();

    // Create dirty file
    alice.write_file("dirty.txt", "uncommitted");

    let config = create_test_config("alice");

    // Mock prompter - select "Stash changes" (index 1)
    let prompter = MockPrompter::new()
        .with_selection(1) // Stash changes
        .with_string("mob-session") // feature branch
        .with_string("main") // base branch
        .with_number(10) // work duration
        .with_string("mob sync"); // commit message

    let git = git::GitCommand::new(Some(alice.path.clone()), "origin".to_string()).unwrap();
    let store = session::SessionStore::new(&git);

    let opts = cmd::StartOpts { minutes: Some(0) };
    let start = cmd::Start::new(&git, &store, &prompter, opts, config);
    start.run().unwrap();

    // Verify file was stashed (no longer in working tree)
    assert!(!alice.file_exists("dirty.txt"));

    // Verify stash exists
    let stash = alice.git_ok(&["stash", "list"]);
    assert!(!stash.is_empty());
}

#[test]
fn test_start_with_dirty_tree_discard() {
    let repos = setup_repos(1);
    let alice = repos.alice();

    alice.write_file("dirty.txt", "uncommitted");

    let config = create_test_config("alice");

    // Mock prompter - select "Discard changes" (index 2)
    let prompter = MockPrompter::new()
        .with_selection(2) // Discard changes
        .with_string("mob-session") // feature branch
        .with_string("main") // base branch
        .with_number(10) // work duration
        .with_string("mob sync"); // commit message

    let git = git::GitCommand::new(Some(alice.path.clone()), "origin".to_string()).unwrap();
    let store = session::SessionStore::new(&git);

    let opts = cmd::StartOpts { minutes: Some(0) };
    let start = cmd::Start::new(&git, &store, &prompter, opts, config);
    start.run().unwrap();

    // Verify file was discarded
    assert!(!alice.file_exists("dirty.txt"));
}

#[test]
fn test_full_workflow_alice_to_bob() {
    let repos = setup_repos(2);
    let alice = repos.alice();
    let bob = repos.bob();

    // Alice starts
    let alice_prompter = MockPrompter::new()
        .with_string("mob-session")
        .with_string("main")
        .with_number(10)
        .with_string("mob sync");

    let alice_git = git::GitCommand::new(Some(alice.path.clone()), "origin".to_string()).unwrap();
    let alice_store = session::SessionStore::new(&alice_git);

    cmd::Start::new(
        &alice_git,
        &alice_store,
        &alice_prompter,
        cmd::StartOpts { minutes: Some(0) },
        create_test_config("alice"),
    )
    .run()
    .unwrap();

    // Alice writes a file
    alice.write_file("alice.txt", "Alice's work");

    // Alice runs next
    cmd::Next::new(&alice_git, &alice_store, create_test_config("alice"))
        .run()
        .unwrap();

    // Verify state is WaitingForNext
    let session = alice_store.load().unwrap();
    assert!(matches!(
        session.state,
        session::State::WaitingForNext { .. }
    ));

    // Bob starts (takes over)
    let bob_prompter = MockPrompter::new(); // Use defaults

    let bob_git = git::GitCommand::new(Some(bob.path.clone()), "origin".to_string()).unwrap();
    let bob_store = session::SessionStore::new(&bob_git);

    cmd::Start::new(
        &bob_git,
        &bob_store,
        &bob_prompter,
        cmd::StartOpts { minutes: Some(0) },
        create_test_config("bob"),
    )
    .run()
    .unwrap();

    // Verify bob is now driver
    let session = bob_store.load().unwrap();
    assert!(matches!(
        session.state,
        session::State::Working { driver } if driver == "bob"
    ));

    // Verify alice's file is present
    assert!(bob.file_exists("alice.txt"));
    assert_eq!(bob.read_file("alice.txt"), "Alice's work");

    // Bob runs done
    let bob_done_prompter = MockPrompter::new();
    cmd::Done::new(
        &bob_git,
        &bob_store,
        &bob_done_prompter,
        create_test_config("bob"),
    )
    .run()
    .unwrap();

    // Verify back on main
    let branch = bob.git_ok(&["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(branch, "main");
}

#[test]
fn test_driver_rotation() {
    let repos = setup_repos(3);
    let alice = repos.alice();
    let bob = repos.bob();
    let carol = repos.carol();

    // Alice starts
    let alice_prompter = MockPrompter::new()
        .with_string("mob-session")
        .with_string("main")
        .with_number(10)
        .with_string("mob sync");

    let alice_git = git::GitCommand::new(Some(alice.path.clone()), "origin".to_string()).unwrap();
    let alice_store = session::SessionStore::new(&alice_git);

    cmd::Start::new(
        &alice_git,
        &alice_store,
        &alice_prompter,
        cmd::StartOpts { minutes: Some(0) },
        create_test_config("alice"),
    )
    .run()
    .unwrap();

    // Alice runs next
    cmd::Next::new(&alice_git, &alice_store, create_test_config("alice"))
        .run()
        .unwrap();

    // Bob joins
    let bob_prompter = MockPrompter::new();
    let bob_git = git::GitCommand::new(Some(bob.path.clone()), "origin".to_string()).unwrap();
    let bob_store = session::SessionStore::new(&bob_git);

    cmd::Start::new(
        &bob_git,
        &bob_store,
        &bob_prompter,
        cmd::StartOpts { minutes: Some(0) },
        create_test_config("bob"),
    )
    .run()
    .unwrap();

    // Verify bob is driver
    let session = bob_store.load().unwrap();
    assert!(matches!(
        session.state,
        session::State::Working { driver } if driver == "bob"
    ));

    // Bob runs next
    cmd::Next::new(&bob_git, &bob_store, create_test_config("bob"))
        .run()
        .unwrap();

    // Carol joins
    let carol_prompter = MockPrompter::new();
    let carol_git = git::GitCommand::new(Some(carol.path.clone()), "origin".to_string()).unwrap();
    let carol_store = session::SessionStore::new(&carol_git);

    cmd::Start::new(
        &carol_git,
        &carol_store,
        &carol_prompter,
        cmd::StartOpts { minutes: Some(0) },
        create_test_config("carol"),
    )
    .run()
    .unwrap();

    // Verify carol is driver
    let session = carol_store.load().unwrap();
    assert!(matches!(
        session.state,
        session::State::Working { driver } if driver == "carol"
    ));

    // Verify all three are in the drivers list
    let drivers = session.drivers.all();
    assert_eq!(drivers.len(), 3);
    assert!(drivers.contains(&"alice".to_string()));
    assert!(drivers.contains(&"bob".to_string()));
    assert!(drivers.contains(&"carol".to_string()));
}
