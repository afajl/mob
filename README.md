
![mob](https://github.com/afajl/mob/raw/main/assets/logo.png)

Mob is a console tool to work in a remote mob (or pair) with git.

* Handover code fast between drivers
* Remembers order of drivers
* [Keeps track of state](#how-it-works) (working, waiting for next, stopped) to eliminate risk of conflicts and lost work
* Use any branch as base

![mob screen](https://github.com/afajl/mob/raw/main/assets/screen.gif)



<!-- Run :UpdateToc to update -->
<!-- vim-markdown-toc GFM -->

* [Installing](#installing)
    * [Cargo](#cargo)
    * [Manually](#manually)
* [Usage](#usage)
    * [FAQ](#faq)
        * [How do I remove all traces of `mob` from a repo?](#how-do-i-remove-all-traces-of-mob-from-a-repo)
        * [Where is the configuration stored?](#where-is-the-configuration-stored)
        * [How do I show current status?](#how-do-i-show-current-status)
        * [Work duration is set to 15 but we must stop for a meeting in 7 minutes](#work-duration-is-set-to-15-but-we-must-stop-for-a-meeting-in-7-minutes)
* [Hooks](#hooks)
* [How it works](#how-it-works)
* [Inspiration and other tools](#inspiration-and-other-tools)

<!-- vim-markdown-toc -->

## Installing

### Cargo
Install [rust](https://www.rust-lang.org/tools/install) and run:
```bash
cargo install remotemob
```

### Manually
Download the [latest
release](https://github.com/afajl/mob/releases/latest) and unpack it to somewhere in your
PATH.

## Usage
> Note: It's safe to try mob! It prints all git commands it runs and if you decide that it's not for you can [remove all traces](#how-do-i-remove-all-traces-of-mob-from-a-repo).
 
- `mob start` creates a new feature branch or syncs the branch from the
  previous driver. 
- `mob next` commits all changes to the feature branch and hands over to the next driver.
- `mob done` stages all changes on the feature branch for commit on the base branch (normally main).

![mob graph](https://github.com/afajl/mob/raw/main/assets/graph.svg)

Run `mob` for help on more commands.

### FAQ
#### How do I remove all traces of `mob` from a repo?
1. Run `mob done` to remove the mob branch. Either commit the
changes or run `git reset HEAD --hard` to discard changes.
2. Run `mob clean` to remove the `mob-meta` branch.
3. Delete `~/.mob` if you don't want to use `mob` anymore

#### Where is the configuration stored?
Configuration local to you is stored in `~/.mob`. Configuration
for a repository is stored in an orphan branch named `mob-meta`.  
`mob start` creates all configuration needed to run. It is always
safe to run `mob clean` to remove the repository config and start
fresh.

#### How do I show current status?
Run `mob status`

#### Work duration is set to 15 but we must stop for a meeting in 7 minutes
Run `mob start 7`


## Hooks
You can add hooks to your configuration in `~/.mob` to notify you
when your turn is over or to take over screen sharing when you
start. 
```language: toml
...
[hooks]
after_start="take_screen.sh"
after_timer="say 'mob next NEXT_DRIVER'"
```


Hooks are executed by a `sh` and can contain two
variables:
- `CURRENT_DRIVER`: Always the name you configured in `~/.mob`
- `NEXT_DRIVER`: Next driver or `anyone` if you are the first in
  a session. It is empty on all `before_*` hooks.

The available hooks are:
- `before_start`: Run as soon as possible when you run `mob start`, before checking that it's your turn 
   or that your working directory is clean.
- `after_start`: Right after you've started a session with `mob start` but before the timer started. 
   This is a good hook for taking over the screen. 
- `after_timer`: Run when your turn ended. The first time you run
   `mob start` it tries to find commands to play a sound and show
   a desktop notification to populate this hook.
- `before_next`: Before running mob next, `NEXT_DRIVER` is not available.
- `after_next`: Before running mob next, `NEXT_DRIVER` is either
   a name or `anyone`. 
- `before_done`: Before the squashing and deleting branches.
- `after_done`: After done has been run, `NEXT_DRIVER` is not available.


## How it works
`mob` uses an orphan branch called `mob-meta` to save session
state and settings. You can view the session content with `mob
status [-r]` and delete it with `mob clean`.

```javascript
Session {
    drivers: Drivers(
        [
            "Paul",
            "Leo",
            "Ella",
        ],
    ),
    branches: Branches {
        branch: "mob-session",
        base_branch: "main",
    },
    settings: Some(
        Settings {
            commit_message: "mob sync [skip ci]",
            work_duration: 10,
        },
    ),
    state: Working {
        driver: "Ella",
    },
}
```

The session can be in 3 different states:  

![mob states](https://github.com/afajl/mob/raw/main/assets/state.svg)


## Inspiration and other tools
Inspiration for this tool comes from [Remote mob
programming](https://www.remotemobprogramming.org/) and their tool
[mob](https://github.com/remotemobprogramming/mob) written in Go.

I did this rewrite to:
- remember the order of drivers
- use any branch as base branch, not only master or main
- know about the state so we get nice
    warnings instead of git conflicts and lost refs.
- and of course... rust ;)

To find other tools look at the [mob programming timer tag on github](https://github.com/topics/mob-programming-timer)
