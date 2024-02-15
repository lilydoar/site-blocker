# site-blocker

![Unit-Test](https://github.com/lilydoar/site-blocker/actions/workflows/test.yaml/badge.svg)
[![Super-Linter](https://github.com/lilydoar/site-blocker/actions/workflows/lint.yaml/badge.svg)](https://github.com/marketplace/actions/super-linter)

A CLI to block sites through the systems hosts file.

## Usage

```shell
site-blocker [OPTIONS] <COMMAND>

Commands:
  list    List blocked sites [aliases: ls]
  add     Add a blocked site
  remove  Remove a blocked site [aliases: rm]
  help    Print this message or the help of the given subcommand(s)
```

```shell
sudo site-blocker list
www.example.com
www.example2.com

sudo site-blocker add --site www.example.com
INFO www.example.com added

sudo site-blocker remove --site www.example.com
INFO www.example.com removed
```

## Installation

```shell
cargo install --git https://github.com/lilydoar/site-blocker.git
```

Or download the latest [release](https://github.com/lilydoar/site-blocker/releases) and add the executable to your PATH.

## todo features

- Accept a file containing a list of sites as a flag
- Load hosts location from various config options. flag, env, config file
- Setup a release of this on GitHub
- Add an edit command which opens the system's default text editor for raw editing of the blocked sites
- Add windows support
- Add ability to read from stdin
- Add option to disable color
- Add autoformatting to the GitHub actions
- Publish CLI on crates.io

