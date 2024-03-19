# site-blocker

![Unit-Test](https://github.com/lilydoar/site-blocker/actions/workflows/test.yaml/badge.svg)
[![Super-Linter](https://github.com/lilydoar/site-blocker/actions/workflows/lint.yaml/badge.svg)](https://github.com/marketplace/actions/super-linter)

A CLI to block sites through the systems hosts file.

## Installation

```shell
cargo install site-blocker
```

This CLI does not natively support Windows and has not been tested on it. However, nothing is stopping it from working since the hosts file path can be configured as an option. Try it out and create an issue if there are problems.

## Usage

```
site-blocker [OPTIONS] <COMMAND>

Commands:
  get     Get blocked sites [aliases: ls]
  add     Add blocked sites
  delete  Remove blocked sites [aliases: rm]
  edit    Edit blocked sites in an editor
```

Browsers usually require their cache to be cleared before they reflect changes to the hosts file.

### Examples


```
sudo site-blocker add www.example.com www.example2.com
INFO www.example.com added
INFO www.example2.com added

site-blocker list
www.example.com
www.example2.com

sudo site-blocker delete www.example.com
INFO www.example.com deleted

sudo site-blocker edit
```
