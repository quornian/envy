use std::{borrow::Cow, collections::HashMap, io::IsTerminal};

use clap::{builder::EnumValueParser, Arg, ArgAction, ColorChoice, Command};
use glob_match::glob_match;
use regex::{Regex, RegexBuilder};

#[derive(Default)]
struct Palette<'a> {
    variable: &'a str,
    value: &'a str,
    matched: &'a str,
    unmatched: &'a str,
    special: &'a str,
    separator: &'a str,
    reset: &'a str,
}

const DEFAULT_COLORS: Palette<'_> = Palette {
    variable: "1",
    value: "",
    matched: "4;97",
    unmatched: "90",
    special: "36",
    separator: "90",
    reset: "0",
};

const SPECIALS: &[(char, &'static str)] = &[
    ('\x1b', "\\x1b"),
    ('\r', "\\r"),
    ('\n', "\\n"),
    ('\t', "\\t"),
    ('\x07', "\\x07"),
];

const WINDOWS_DEFAULT_SEP: &str = ";,:";
const UNIX_DEFAULT_SEP: &str = ":,";

#[derive(Debug)]
enum Pattern<'a> {
    Regex(Regex),
    Glob(&'a str),
}

impl Pattern<'_> {
    fn matches(&self, haystack: &str) -> bool {
        match self {
            Pattern::Regex(re) => re.is_match(haystack),
            Pattern::Glob(glob) => glob_match(glob, haystack),
        }
    }
}

impl From<Regex> for Pattern<'_> {
    fn from(value: Regex) -> Self {
        Pattern::Regex(value)
    }
}

fn main() {
    // Set up the command line arguments
    let cmd = Command::new("Envy")
        .version("v1.0")
        .author("Ian Thompson <quornian@gmail.com>")
        .about(
            "Formats and displays environment variables for human friendly reading, \
            searching and comparison",
        )
        .arg(Arg::new("pattern").help(
            "The name or glob-like pattern of the environment variable(s) to show \
            (use -r to switch to regular expressions). If omitted, all environment \
            variables will be displayed",
        ))
        .arg(
            Arg::new("use_regex")
                .short('r')
                .long("regex")
                .action(ArgAction::SetTrue)
                .help("Treat pattern as a regular expression to match against names"),
        )
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .value_name("regex")
                .help("Search the values of environment variables for the given pattern"),
        )
        .arg(
            Arg::new("ignore_case")
                .short('i')
                .long("ignore-case")
                .action(ArgAction::SetTrue)
                .requires("use_regex")
                .help("Make regular expression search and pattern match case insensitive"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .value_name("when")
                .help("Control when to color the output")
                .value_parser(EnumValueParser::<ColorChoice>::new())
                .num_args(0..=1)
                .require_equals(true)
                .default_missing_value("always")
                .default_value("auto"),
        )
        .env_help();

    // Parse arguments
    let matches = cmd.get_matches();
    let ignore_case = matches.get_flag("ignore_case");
    let pattern = matches.get_one::<String>("pattern").map(String::as_str);
    let pattern = if matches.get_flag("use_regex") {
        RegexBuilder::new(pattern.unwrap_or(""))
            .case_insensitive(ignore_case)
            .build()
            .unwrap_or_else(|e| {
                eprintln!("Invalid pattern: {e}");
                std::process::exit(1);
            })
            .into()
    } else {
        Pattern::Glob(pattern.unwrap_or("*"))
    };
    let value_search = matches.get_one::<String>("search").map(|search| {
        RegexBuilder::new(search)
            .case_insensitive(ignore_case)
            .build()
            .unwrap_or_else(|e| {
                eprintln!("Invalid pattern: {e}");
                std::process::exit(1);
            })
    });
    let use_color = match matches.get_one::<ColorChoice>("color").unwrap() {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => std::io::stdout().is_terminal(),
    };

    // Set up color palette from command line and environment
    let colors_from_env: HashMap<_, _> = {
        std::env::var("ENVY_COLORS")
            .map(|value| {
                let env_color_re = Regex::new("^(var|val|spe|sep)=([0-9;]*)$").unwrap();
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
            .unwrap_or_default()
    };
    let palette = if use_color {
        let var_or = |var, def| {
            format!(
                "\x1b[{}m",
                colors_from_env.get(var).map(String::as_str).unwrap_or(def)
            )
        };
        Palette {
            variable: &var_or("var", DEFAULT_COLORS.variable),
            value: &var_or("val", DEFAULT_COLORS.value),
            matched: &var_or("mat", DEFAULT_COLORS.matched),
            unmatched: &var_or("unm", DEFAULT_COLORS.unmatched),
            special: &var_or("spe", DEFAULT_COLORS.special),
            separator: &var_or("sep", DEFAULT_COLORS.separator),
            reset: &format!("\x1b[{}m", DEFAULT_COLORS.reset),
        }
    } else {
        Palette::default()
    };

    let separator_re = {
        let separator_chars = regex::escape(&std::env::var("ENVY_SEP").unwrap_or_else(|_| {
            if cfg!(windows) {
                WINDOWS_DEFAULT_SEP
            } else {
                UNIX_DEFAULT_SEP
            }
            .to_owned()
        }));
        Regex::new(&format!("([^{separator_chars}]*)([{separator_chars}]*)"))
            .expect("Invalid ENVY_SEP")
    };

    // Filter and print the environment variables that match the regex pattern
    let mut variables: Vec<_> = std::env::vars()
        .filter(|(key, _value)| pattern.matches(&key))
        .collect();
    variables.sort();
    let variables = variables;
    let Palette {
        variable: p_var,
        value: p_val,
        matched: p_mat,
        unmatched: p_unm,
        special: p_spe,
        separator: p_sep,
        reset: p_res,
    } = palette;
    let found_marker = if use_color { ' ' } else { '*' };

    for (env_key, env_value) in variables.into_iter() {
        // Deal with values first so we can reject the whole entry when a value search
        // matches none of the values
        let mut any_match = value_search.is_none();
        let parts: Vec<_> = separator_re
            .captures_iter(&env_value)
            .map(|capture| {
                let (_, [part, sep]) = capture.extract();
                let part_matched = value_search.as_ref().map(|search| search.is_match(part));
                any_match = any_match || part_matched.unwrap_or_default();

                // Pre-format special character replacements here
                let mut part = Cow::from(part);
                for &(ch, repl) in SPECIALS.iter() {
                    // Replace always allocates a new string, so check first
                    if part.contains(ch) {
                        part = part
                            .as_ref()
                            .replace(ch, &format!("{p_spe}{repl}{p_res}"))
                            .into();
                    }
                }

                // Highlight matched segment
                if let Some(search) = value_search.as_ref() {
                    part = match search.replace_all(&part, &format!("{p_mat}$0{p_res}{p_val}")) {
                        Cow::Borrowed(_) => part,
                        Cow::Owned(x) => Cow::Owned(x),
                    };
                }
                (part_matched, part, sep)
            })
            .collect();
        if !any_match {
            continue;
        }

        println!("{p_var}{env_key}{p_res}{p_sep}={p_res}");
        for (part_matched, part, sep) in parts {
            let (marker, style) = match part_matched {
                Some(true) => (found_marker, ""),
                Some(false) => (' ', p_unm),
                None => (' ', ""),
            };
            println!("{marker} {style}{part}{p_res}{p_sep}{sep}{p_res}");
        }

        println!();
    }
}

trait EnvHelp {
    fn env_help(self) -> Self;
}

impl EnvHelp for Command {
    fn env_help(self) -> Self {
        // Add additional help for the environment variable ENVY_COLORS
        let hdr = self.get_styles().get_header();
        let (hdr, hdr_reset) = (hdr.render(), hdr.render_reset());
        let lit = self.get_styles().get_literal();
        let (lit, lit_reset) = (lit.render(), lit.render_reset());
        let after_help = format!(
            "{hdr}Environment:{hdr_reset}\
            \n  {lit}ENVY_COLORS{lit_reset}  \
            Override colors for different elements of the output.\n"
        );
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
        let after_long_help = format!(
            "{hdr}Environment:{hdr_reset}\
            \n  {lit}ENVY_COLORS{lit_reset}{cur}\
            \n          Overrides the default colors used to display different elements of \
                        the output:\
            \n            <{lit}var{lit_reset}>iable  - environment variable names\
            \n            <{lit}val{lit_reset}>ue     - environment variable values\
            \n            <{lit}spe{lit_reset}>cial   - special characters\
            \n            <{lit}sep{lit_reset}>arator - separator characters\
            \n          \
            \n          Color settings are colon-separated, key-value pairs in key=value form. \
                        Values are ANSI color codes.\
            \n          \
            \n          [default: {def}]\
            \n\
            \n  {lit}ENVY_SEP{lit_reset}\
            \n          Overrides the OS specific path separators, which by default are:\
            \n          Linux/MacOS:  {lit}ENVY_SEP={UNIX_DEFAULT_SEP}{lit_reset}\
            \n          Windows:      {lit}ENVY_SEP={WINDOWS_DEFAULT_SEP}{lit_reset}",
            cur = if let Ok(cur) = std::env::var("ENVY_COLORS") {
                format!(" = {}", hi(&cur))
            } else {
                String::new()
            },
            def = {
                let Palette {
                    variable,
                    value,
                    matched,
                    unmatched,
                    special,
                    separator,
                    reset: _,
                } = DEFAULT_COLORS;
                hi(&format!(
                    "var={variable}:val={value}:mat={matched}:\
                    unm={unmatched}:spe={special}:sep={separator}"
                ))
            }
        );
        self.after_help(after_help).after_long_help(after_long_help)
    }
}
