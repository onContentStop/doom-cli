# doom-cli
A command-line interface to launch Doom more ergonomically.

## Configuration
Configuration is done through a bunch of RON files stored in your $HOME/doom directory. The tool will notify you and create this directory if it doesn't already exist.
It will also generate templates for the files so you know what goes where.

### engines.ron
`engines.ron` contains the list of Doom engines you want to be able to select.

### autoloads.ron
`autoloads.ron` contains a list of Doom WAD (or .pk3, .zip, etc) files you want to autoload under certain conditions.

## Command-line
See `playdoom --help` for a description of all the options. The main ones you will probably be using are `-e` and `-p`. `-n` is useful for desktop entries on Linux.
