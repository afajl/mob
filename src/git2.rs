use anyhow::Result;
use git2::{Config, Error, Oid, Repository, Signature};
use serde::Serialize;
use std::env;

pub struct Git {
    repo: Repository,
}

impl Git {
    pub fn new() -> Result<Git> {
        let cwd = env::current_dir()?;
        let repo = Repository::discover(cwd)?;
        Ok(Git { repo })
    }

    pub fn save_meta<T: ?Sized>(&self, name: &str, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let json = serde_json::to_vec(data)?;
        let oid = self.repo.blob(json.as_slice())?;
        let mut tree = self.repo.treebuilder(None)?;
        tree.insert(name, oid, 0o100644)?;
        let tree = tree.write()?;
        let tree = self.repo.find_tree(tree)?;
        println!("get signature");
        let signature = Git::get_signature()?;

        let parent = self
            .repo
            .find_reference("refs/heads/mob-meta")
            .and_then(|reference| reference.resolve())
            .and_then(|reference| {
                self.repo
                    .find_commit(reference.target().unwrap_or(Oid::zero()))
            })
            .ok();

        let parent = match parent {
            Some(ref commit) => vec![commit],
            None => vec![],
        };

        self.repo.commit(
            Some("refs/heads/mob-meta"),
            &signature,
            &signature,
            "save",
            &tree,
            parent.as_slice(),
        )?;
        Ok(())
    }

    fn get_signature() -> Result<Signature<'static>, Error> {
        let config = Config::open_default()?;
        let name = config.get_string("user.name")?;
        let email = config.get_string("user.email")?;
        Signature::now(name.as_str(), email.as_str())
    }
}

#[derive(Serialize)]
pub struct Apa {
    knas: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_new() {
        match Git::new() {
            Ok(_) => {}
            Err(_) => panic!("could not discover repo"),
        }
    }

    #[test]
    fn create() {
        let git = Git::new().expect("could not create git");
        let d = Apa {
            knas: String::from("knas"),
        };
        git.save_meta("mob", &d).expect("foora")
    }
}
