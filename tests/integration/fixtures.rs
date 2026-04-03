use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use std::{fs, io};
use tempfile::TempDir;

static INIT: Once = Once::new();

pub fn setup_repos(num_clones: usize) -> TestRepos {
    INIT.call_once(|| {
        env_logger::init();
    });

    TestRepos::new(num_clones)
}

pub struct TestClone {
    pub path: PathBuf,
}

impl TestClone {
    pub fn git(&self, args: &[&str]) -> io::Result<std::process::Output> {
        Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .output()
    }

    pub fn git_ok(&self, args: &[&str]) -> String {
        let output = self.git(args).expect("Failed to execute git");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    pub fn write_file(&self, name: &str, content: &str) {
        fs::write(self.path.join(name), content).unwrap();
    }

    pub fn read_file(&self, name: &str) -> String {
        fs::read_to_string(self.path.join(name)).unwrap()
    }

    pub fn file_exists(&self, name: &str) -> bool {
        self.path.join(name).exists()
    }
}

pub struct TestRepos {
    _temp_dir: TempDir,
    pub clones: Vec<TestClone>,
}

impl TestRepos {
    pub fn new(num_clones: usize) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create bare origin with main as default branch
        let origin = base_path.join("origin.git");
        fs::create_dir_all(&origin).unwrap();

        run_git(&origin, &["init", "--bare", "--initial-branch=main"]);

        // Create init clone and make initial commit
        let init_clone = base_path.join("init_clone");
        run_git(
            base_path,
            &["clone", origin.to_str().unwrap(), "init_clone"],
        );

        // Configure and commit
        for (key, value) in [("user.name", "Init"), ("user.email", "init@test.local")] {
            run_git(&init_clone, &["config", key, value]);
        }

        std::fs::write(init_clone.join("README.md"), "# Test\n").unwrap();
        run_git(&init_clone, &["add", "."]);
        run_git(&init_clone, &["commit", "-m", "init"]);
        run_git(&init_clone, &["push", "-u", "origin", "main"]);

        // Create user clones
        let names = ["alice", "bob", "carol"];
        let mut clones = Vec::new();

        for name in names.iter().take(num_clones) {
            let clone_path = base_path.join(format!("clone_{}", name));

            run_git(
                base_path,
                &[
                    "clone",
                    origin.to_str().unwrap(),
                    clone_path.to_str().unwrap(),
                ],
            );

            for (key, value) in [
                ("user.name", name),
                ("user.email", &format!("{}@test.local", name).as_str()),
            ] {
                run_git(&clone_path, &["config", key, value]);
            }

            // Fetch to ensure remote tracking refs are available
            run_git(&clone_path, &["fetch", "--all"]);

            clones.push(TestClone { path: clone_path });
        }

        Self {
            _temp_dir: temp_dir,
            clones,
        }
    }

    pub fn alice(&self) -> &TestClone {
        &self.clones[0]
    }

    pub fn bob(&self) -> &TestClone {
        &self.clones[1]
    }

    pub fn carol(&self) -> &TestClone {
        &self.clones[2]
    }
}

fn run_git(path: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .unwrap()
}
