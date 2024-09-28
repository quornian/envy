use std::{collections::HashMap, io::IsTerminal};

use clap::{Arg, ArgGroup, Command};
use regex::Regex;

#[derive(Default)]
struct Palette<'a> {
    variable: &'a str,
    value: &'a str,
    special: &'a str,
    separator: &'a str,
    reset: &'a str,
}

const DEFAULT_COLORS: Palette<'_> = Palette {
    variable: "1",
    value: "",
    special: "36",
    separator: "38;5;242",
    reset: "0",
};

const SPECIALS: &[(char, &'static str)] = &[
    ('\x1b', "\\x1b"),
    ('\r', "\\r"),
    ('\n', "\\n"),
    ('\t', "\\t"),
    ('\x07', "\\x07"),
];

fn main() {
    let cmd = Command::new("Envy")
        .version("1.0")
        .author("Ian Thompson <quornian@gmail.com>")
        .about("Prints environment variables matching a given regular expression")
        .arg(
            Arg::new("fixed_pattern")
                .value_name("NAME")
                .help("An environment variable name")
                .index(1),
        )
        .arg(
            Arg::new("regex_pattern")
                .short('r')
                // .long("regex")
                .hide_long_help(true)
                .value_name("PATTERN")
                .help("Regular expression pattern to match against environment variable names")
                .default_value(".*"),
        )
        .group(
            ArgGroup::new("pattern")
                .arg("fixed_pattern")
                .arg("regex_pattern")
                .required(true),
        )
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .value_name("WHEN")
                .help("Colorize output")
                .value_names(&["never", "always", "auto"])
                .default_missing_value("always")
                .default_value("auto"),
        );
    let after_help = format!(
        concat!(
            "{hdr}Environment:{rst}\n",
            "  {lit}ENVY_COLORS{rst}  Override colors for different elements of the output.\n",
        ),
        hdr = cmd.get_styles().get_header(),
        lit = cmd.get_styles().get_literal(),
        rst = cmd.get_styles().get_header().render_reset(),
    );
    let after_long_help = {
        let hi = if std::io::stdout().is_terminal() {
            |s: &str| {
                Regex::new("([0-9;]+)")
                    .unwrap()
                    .replace_all(&s, "\x1b[${1}m${1}\x1b[m")
                    .into_owned()
            }
        } else {
            |s: &str| s.to_owned()
        };
        format!(
            concat!(
                "{hdr}Environment:{rst}\n",
                "  {lit}ENVY_COLORS{rst}{cur}\n",
                "          Overrides the default colors used to display different elements of the output:\n",
                "            <{lit}var{rst}>iable  - environment variable names\n",
                "            <{lit}val{rst}>ue     - environment variable values\n",
                "            <{lit}spe{rst}>cial   - special characters\n",
                "            <{lit}sep{rst}>arator - separator characters\n",
                "          \n",
                "          Color settings are colon-separated, key-value pairs in key=value form.\n",
                "          Values are ANSI color codes (31 is foreground red, etc.)\n",
                "          \n",
                "          [default: {def}]",
            ),
            hdr = cmd.get_styles().get_header(),
            lit = cmd.get_styles().get_literal(),
            rst = cmd.get_styles().get_header().render_reset(),
            cur = if let Ok(cur) = std::env::var("ENVY_COLORS") {
                format!(" = {}", hi(&cur))
            } else {
                String::new()
            },
            def = hi("var=1:val=:spe=36:sep=38;5;242")
        )
    };
    let cmd = cmd.after_help(after_help).after_long_help(after_long_help);
    let matches = cmd.get_matches();
    let pattern = matches
        .get_one::<String>("fixed_pattern")
        .map(|fixed| Regex::new(&format!("^(?:{})$", regex::escape(fixed))).unwrap())
        .unwrap_or_else(|| {
            Regex::new(matches.get_one::<String>("regex_pattern").unwrap()).unwrap_or_else(|e| {
                eprintln!("Invalid pattern: {e}");
                std::process::exit(1);
            })
        });

    let color_arg = matches.get_one::<String>("color").unwrap();

    let env_color_re = Regex::new("^(var|val|spe|sep)=([0-9;]*)$").unwrap();
    let colors_from_env: HashMap<_, _> = std::env::var("ENVY_COLORS")
        .map(|value| {
            value
                .split(':')
                .filter_map(|part| {
                    env_color_re.captures(part).map(|captures| {
                        let (_, [key, value]) = captures.extract();
                        (key.to_owned(), value.to_owned())
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let palette = if match color_arg.as_str() {
        "always" => true,
        "never" => false,
        _auto => std::io::stdout().is_terminal(),
    } {
        let var_or = |var, def| {
            format!(
                "\x1b[{}m",
                colors_from_env.get(var).map(String::as_str).unwrap_or(def)
            )
        };
        Palette {
            variable: &var_or("var", DEFAULT_COLORS.variable),
            value: &var_or("val", DEFAULT_COLORS.value),
            special: &var_or("spe", DEFAULT_COLORS.special),
            separator: &var_or("sep", DEFAULT_COLORS.separator),
            reset: &format!("\x1b[{}m", DEFAULT_COLORS.reset),
        }
    } else {
        Palette::default()
    };

    let separator_chars = regex::escape(
        &std::env::var("ENVY_SEP")
            .unwrap_or_else(|_| if cfg!(windows) { ":;," } else { ":," }.to_owned()),
    );
    let separator_re = Regex::new(&format!("([^{separator_chars}]*)([{separator_chars}]*)"))
        .expect("Invalid ENVY_SEP");

    // Filter and print the environment variables that match the regex pattern
    let mut variables: Vec<_> = std::env::vars()
        .filter(|(key, _v)| pattern.is_match(&key))
        .collect();
    variables.sort();
    let variables = variables;
    let Palette {
        variable,
        value,
        special,
        separator,
        reset,
    } = palette;

    for (env_key, mut env_value) in variables.into_iter() {
        println!("{variable}{env_key}{reset}{separator}={reset}");
        for &(ch, repl) in SPECIALS.iter() {
            // Replace always allocates a new string, so check first
            if env_value.contains(ch) {
                env_value = env_value.replace(ch, &format!("{special}{repl}{reset}"));
            }
        }

        let parts: Vec<_> = separator_re.captures_iter(&env_value).collect();
        if !parts.is_empty() {
            for env_part in parts {
                let (_, [x, y]) = env_part.extract();
                println!("  {value}{x}{reset}{separator}{y}{reset}");
            }
        }
        println!();
    }
}
