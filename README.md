# Envy

A command line tool for displaying environment variables in a human friendly form.

Variables may be selected by name, glob-like `pattern` or regular expression (`-r`).

<div style="display:flex; gap: 1em">
<pre style="flex:1"><em><b>$</b></em> envy -os sbin PATH
<b>PATH</b><span style="opacity:.3">=</span>
  /home/ian/.cargo/bin<span style="opacity:.3">:</span>
  /home/ian/.local/bin<span style="opacity:.3">:</span>
  /home/ian/bin<span style="opacity:.3">:</span>
  /usr/local/sbin<span style="opacity:.3">:</span>
  /usr/local/bin<span style="opacity:.3">:</span>
  /usr/sbin<span style="opacity:.3">:</span>
  /usr/bin<span style="opacity:.3">:</span>
  /sbin<span style="opacity:.3">:</span>
  /bin<span style="opacity:.3">:</span>
  /usr/games<span style="opacity:.3">:</span>
  /usr/local/games
</pre>
<pre style="flex:1"><em><b>$</b></em> envy -s sbin PATH
<b>PATH</b><span style="opacity:.3">=</span>
  <span style="opacity:.3">/home/ian/.cargo/bin:</span>
  <span style="opacity:.3">/home/ian/.local/bin:</span>
  <span style="opacity:.3">/home/ian/bin:</span>
  /usr/local/<u>sbin</u><span style="opacity:.3">:</span>
  <span style="opacity:.3">/usr/local/bin:</span>
  /usr/<u>sbin</u><span style="opacity:.3">:</span>
  <span style="opacity:.3">/usr/bin:</span>
  /<u>sbin</u><span style="opacity:.3">:</span>
  <span style="opacity:.3">/bin:</span>
  <span style="opacity:.3">/usr/games:</span>
  <span style="opacity:.3">/usr/local/games
</pre>
<pre style="flex:1"><em><b>$</b></em> envy -os sbin PATH
<b>PATH</b><span style="opacity:.3">=</span>
  <span style="opacity:.3">...</span>
  /usr/local/<u>sbin</u><span style="opacity:.3">:</span>
  <span style="opacity:.3">...</span>
  /usr/<u>sbin</u><span style="opacity:.3">:</span>
  <span style="opacity:.3">...</span>
  /<u>sbin</u><span style="opacity:.3">:</span>
  <span style="opacity:.3">...</span>
</pre></div>

<pre><em><b>$</b></em> envy -e XDG*DIRS
<b>XDG_CONFIG_DIRS</b><span style="opacity:.3">=</span>
  <span style="color:#833">/etc/xdg/xdg-cinnamon</span><span style="opacity:.3">:</span>
  /etc/xdg
&nbsp;
<b>XDG_DATA_DIRS</b><span style="opacity:.3">=</span>
  /usr/share/cinnamon<span style="opacity:.3">:</span>
  /usr/share/gnome<span style="opacity:.3">:</span>
  <span style="color:#833">/home/ian/.local/share/flatpak/exports/share</span><span style="opacity:.3">:</span>
  /var/lib/flatpak/exports/share<span style="opacity:.3">:</span>
  /usr/local/share<span style="opacity:.3">:</span>
  /usr/share</pre>

<pre><em><b>$</b></em> envy -of hx
<b>PATH</b><span style="opacity:.3">=</span>
  /home/ian/.cargo/bin<span style="opacity:.3">[</span><span style="opacity:.3">/</span><u style="color:#6cc">hx</u><span style="opacity:.3">]:</span>
  <span style="opacity:.3">...</span></pre>

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
          The name or glob-like pattern of the environment variable(s) to show
          (use -r to switch to regular expressions). If omitted, all environment
          variables will be displayed.

Options:
  -r, --regex
          Treat pattern as a regular expression to match against names.

  -s, --search <regex>
          Search the values of environment variables for the given pattern.

  -o, --only-matching
          After splitting values, elide unmatched lines and display only those
          that match the regular expression given by --search.

  -i, --ignore-case
          Make regular expression search and pattern match case insensitive.

  -e, --exists
          Indicate any lines that appear to be paths but cannot be found on
          disk.

      --color[=<when>]
          Control when to color the output.

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Environment:
  ENVY_COLORS
          Override the default colors used to display different elements of the
          output:

              var(iable)  - environment variable names
              val(ue)     - environment variable values
              mat(ched)   - highlighting for matched segments
              unm(atched) - dimming for unmatched lines
              mis(sing)   - indication for paths not found
              spe(cial)   - special characters
              sep(arator) - separator characters

          Color settings are colon-separated, key-value pairs in key=value form.
          Values are ANSI color codes.

          [default: var=1:val=:mat=4;97:unm=90:mis=2;31:spe=36:sep=90]

  ENVY_SEP
          Override the OS specific path separators, which by default are:

              (Linux, MacOS)  ENVY_SEP=:,
              (Windows)       ENVY_SEP=;,
```
