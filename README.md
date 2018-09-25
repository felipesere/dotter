# Welcome to Dotter!
Think ansible, but smaller and no outside dependencies that `dotter` can't install itself.

Currently it only installs `homebrew` if its not found.

## Running in Vagrant

```sh
brew cask install virtualbox virtualbox-extension-pack vagrant vagrant-manager
```

## What it should be able to do

* Execute changes, if needed
* Rollback changes, if needed
* Explain what it is doing
* Apply only a subset of the inventory

## Inventory
Similar to Ansible, the inventory describes what `dotter` should do.

An `inventory` is made up of several named `groups`.
Currently, a group can be made up of arrays of the following things:

* `brew` is an array packages to install with hombrew.
* `symlinks` is an array of symlinks to create
* `shell` is an array of commands to execute

For homebrew, there there options:
* regular packages are just strings like `"htop"`
* cask installs with `{"cask": "intellij-idea"}`
* tap installs with `{"tap": "homeebrew/cask-fonts", "name": "font-roboto-mono"}`

Symlinks can contain environment variables which will be expanded with whatever is in the environment for the dotter process.


## Command line

Only two parameters are mandatory: whether to `run` or `rollback` and the inventory to use.

```sh
./dotter run ./sample/inventory.json
```

If you only want to run a `group` in that inventory, you can name it with `--group`:

```sh
./dotter run ./sample/inventory.json --only vim
```

Finally, if you just want to see what `run` or `rollback` would do, add the `--explain` flag.


