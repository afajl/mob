# mob

A console tool to work in a remote mob (or pair) with git.

* Handover code fast between drivers
* Nice timer
* Remembers order of drivers
* Configurable interval for breaks and lunch

![mob screen](https://github.com/afajl/mob/raw/master/screen.gif)



<!-- vim-markdown-toc GFM -->

* [How to install](#how-to-install)
* [Usage](#usage)
  * [FAQ](#faq)
      * [Remove all traces of `mob` from a repo](#remove-all-traces-of-mob-from-a-repo)
      * [Where the configuration stored](#where-the-configuration-stored)
      * [Show status](#show-status)
      * [How do I change break times, lunch etc](#how-do-i-change-break-times-lunch-etc)
      * [Work duration is set to 15 but we're supposed to be in a meeting in 7 minutes](#work-duration-is-set-to-15-but-were-supposed-to-be-in-a-meeting-in-7-minutes)
* [How it works](#how-it-works)
* [Thanks](#thanks)

<!-- vim-markdown-toc -->

## How to install
```bash
cargo install remotemob
```


## Usage 
- `mob start` creates a new session or takes over from the
  previous driver. It will ask a bunch of questions about
  branches, work interval, break times if it needs.
- `mob next` hands over to the next driver.
- `mob done` squashes the feature branch to staging on the base branch
  (default master) and removes it.

Run `mob` for help on more commands.

### FAQ
##### Remove all traces of `mob` from a repo
1. Run `mob done` to remove the mob branch. Either commit the
changes or run `git reset HEAD --hard` to discard changes.
2. Run `mob clean` to remove the `mob-meta` branch.
3. Delete `~/.mob` if you don't want to use `mob` more

##### Where the configuration stored
Configuration local to you is stored in `~/.mob`. Configuration
for a repository is stored in an orphan branch named `mob-meta`.  
`mob start` creates all configuration needed to run. It is always
safe to run `mob clean` to remove the repository config and start
fresh.

##### Show status
Run `mob status`

##### How do I change break times, lunch etc
Currently you have to run `mob clean` and then `mob start`.

##### Work duration is set to 15 but we're supposed to be in a meeting in 7 minutes
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
