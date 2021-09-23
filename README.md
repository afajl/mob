# mob

A console tool to work in a remote mob (or pair) with git.

* Handover code fast between drivers
* Nice timer
* Remembers order of drivers

![mob screen](https://github.com/afajl/mob/raw/master/screen.gif)



<!-- Run :UpdateToc to update -->
<!-- vim-markdown-toc GFM -->

* [How to install](#how-to-install)
* [Usage](#usage)
  * [FAQ](#faq)
      * [How do I remove all traces of `mob` from a repo?](#how-do-i-remove-all-traces-of-mob-from-a-repo)
      * [Where is the configuration stored?](#where-is-the-configuration-stored)
      * [How do I show current status?](#how-do-i-show-current-status)
      * [Work duration is set to 15 but we must stop for a meeting in 7 minutes](#work-duration-is-set-to-15-but-we-must-stop-for-a-meeting-in-7-minutes)
* [How it works](#how-it-works)
* [Thanks](#thanks)

<!-- vim-markdown-toc -->

## How to install
Install [rust](https://www.rust-lang.org/tools/install) and run:
```bash
cargo install remotemob
```


## Usage 
- `mob start` creates a new feature branch or syncs the branch from the
  previous driver. 
- `mob next` commits all changes to the feature branch and hands over to the next driver.
- `mob done` stages all changes on the feature branch for commit on the base branch (normally master).

![mob graph](https://github.com/afajl/mob/raw/master/graph.png)

Run `mob` for help on more commands.

### FAQ
##### How do I remove all traces of `mob` from a repo?
1. Run `mob done` to remove the mob branch. Either commit the
changes or run `git reset HEAD --hard` to discard changes.
2. Run `mob clean` to remove the `mob-meta` branch.
3. Delete `~/.mob` if you don't want to use `mob` more

##### Where is the configuration stored?
Configuration local to you is stored in `~/.mob`. Configuration
for a repository is stored in an orphan branch named `mob-meta`.  
`mob start` creates all configuration needed to run. It is always
safe to run `mob clean` to remove the repository config and start
fresh.

##### How do I show current status?
Run `mob status`

##### Work duration is set to 15 but we must stop for a meeting in 7 minutes
Run `mob start 7`


## How it works
`mob` uses an orphan branch called `mob-meta` to save session
state and settings. You can view the session content with `mob
status` and delete it with `mob clean`.

The session can be in 3 different states:

![mob states](https://github.com/afajl/mob/raw/master/state.svg)


## Thanks
Inspiration for this tool comes from [Remote mob
programming](https://www.remotemobprogramming.org/) and their tool
[mob](https://github.com/remotemobprogramming/mob) written in Go.
