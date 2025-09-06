# Envy

A command line tool for displaying environment variables in a human friendly form.

Variables may be selected by name, glob-like `pattern` or regular expression (`-r`).

![Example displaying the PATH variable and a search for sbin](envy.png)

Variable values are split by the OS specific path separator onto separate lines. These lines can be further searched (`-s`) or checked for path existence (`-e`).

## Installation

Download one of the pre-compiled [releases](https://github.com/quornian/envy/releases) for your operating system, or install via Cargo using:

```
cargo install --locked envy-cmd
```

## Usage

```
$ envy --help

Formats and displays environment variables for human friendly reading, searching
and comparison.

Usage: envy [OPTIONS] [pattern]

Arguments:
  [pattern]
          Name(s) of environment variable(s) to show.
          May be a glob-like pattern or, used with -r, a regular expression.
          If omitted, all environment variables will be displayed.

Options:
  -i, --ignore-case
          Make regular expressions case insensitive.

      --color[=<when>]
          Control when to color the output.

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Name Options:
  -r, --regex
          Switch from glob-like to regular expressions when matching variable
          names.

Value Options:
  -s, --search <regex>
          Search the values of variables for the given regular expression.

  -f, --find <filename>
          Indicate which directories contain the given filename or glob-like
          pattern.

  -o, --only-matching
          Display only lines that match the regular expression given by -s or
          contain the file given by -f. Other lines are elided.

  -e, --exists
          Indicate any lines that appear to be paths but cannot be found on
          disk.

Environment:
  ENVY_COLORS
          Override the default colors used to display different elements of the
          output:

              var(iable)  - environment variable names
              val(ue)     - environment variable values
              mat(ched)   - highlighting for matched segments
              unm(atched) - dimming for unmatched lines
              fil(ename)  - indication for files found in directories
              mis(sing)   - indication for paths not found
              spe(cial)   - special characters
              sep(arator) - separator characters

          Color settings are colon-separated, key-value pairs in key=value form.
          Values are ANSI color codes.

          [default: var=1:val=:mat=4;97:unm=90:fil=3;96:mis=2;31:spe=35:sep=90]

  ENVY_SEP
          Override the OS specific path separators, which by default are:

              (Linux, MacOS)  ENVY_SEP=:,
              (Windows)       ENVY_SEP=;,
```
