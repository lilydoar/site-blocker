# site-blocker

![Unit-Test](https://github.com/lilydoar/site-blocker/actions/workflows/test.yaml/badge.svg)
[![Super-Linter](https://github.com/lilydoar/site-blocker/actions/workflows/lint.yaml/badge.svg)](https://github.com/marketplace/actions/super-linter)

A CLI to block sites through the systems hosts file.

## Usage

```shell
site-blocker [OPTIONS] <COMMAND>

Commands:
  get     Get blocked sites
  add     Add blocked sites
  delete  Remove blocked sites
  edit    Edit blocked sites through system editor
```

```shell
sudo site-blocker list
www.example.com
www.example2.com

sudo site-blocker add www.example.com
INFO www.example.com added

sudo site-blocker delete www.example.com
INFO www.example.com deleted
```

Warning: Browsers require their cache to be cleared before they reflect changes to the hosts file.

## Installation

```shell
cargo install --git https://github.com/lilydoar/site-blocker.git
```

Or download the latest [release](https://github.com/lilydoar/site-blocker/releases) and add the executable to your PATH.

## todo features

- Check for write permission before trying to write. This will make output less confusing
- Setup a release of this on GitHub
- Publish CLI on crates.io
- Add autoformatting to the GitHub actions
