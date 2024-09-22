use std::io::IsTerminal;

use clap::{App, Arg};
use regex::Regex;

#[derive(Default)]
struct Palette<'a> {
    header: &'a str,
    value: &'a str,
    special: &'a str,
    separator: &'a str,
    reset: &'a str,
}

const DEFAULT_COLORS: Palette<'_> = Palette {
    header: "1",
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
    let matches = App::new("Envy")
        .version("1.0")
        .author("Ian Thompson <quornian@gmail.com>")
        .about("Prints environment variables matching a given regular expression")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("Regular expression pattern to match against environment variable names")
                .takes_value(true)
                .default_value("")
                .index(1),
        )
        .arg(
            Arg::with_name("color")
                .short('c')
                .long("color")
                .value_name("WHEN")
                .help("Colorize output")
                .takes_value(true)
                .possible_values(&["never", "always", "auto"])
                .default_missing_value("always")
                .default_value("auto"),
        )
        .get_matches();

    let pattern_arg = matches.value_of("pattern").unwrap();
    let color_arg = matches.value_of("color").unwrap();

    // Modify the pattern to anchor it to the start of the text
    let pattern_re = Regex::new(&format!("^(?:{pattern_arg})")).unwrap_or_else(|e| {
        // On error, parse the unmodified expression to give the user an error
        // that ties directly to what they wrote. If somehow this parses (and
        // our modified one didn't), panic!
        let e = Regex::new(pattern_arg)
            .map(|_| panic!("pattern:\n    {pattern_arg}\n{}", e))
            .unwrap_err();
        eprintln!("Invalid pattern: {e}");
        std::process::exit(1);
    });

    let palette = if match color_arg {
        "always" => true,
        "never" => false,
        _auto => std::io::stdout().is_terminal(),
    } {
        let var_or = |e, d| format!("\x1b[{}m", std::env::var(e).as_deref().unwrap_or(d));
        Palette {
            header: &var_or("ENVY_HEADER", DEFAULT_COLORS.header),
            value: &var_or("ENVY_VALUE", DEFAULT_COLORS.value),
            special: &var_or("ENVY_SPECIAL", DEFAULT_COLORS.special),
            separator: &var_or("ENVY_SEPARATOR", DEFAULT_COLORS.separator),
            reset: &format!("\x1b[{}m", DEFAULT_COLORS.reset),
        }
    } else {
        Palette::default()
    };

    let separator_chars = std::env::var("ENVY_SEP")
        .unwrap_or_else(|_| if cfg!(windows) { ":;," } else { ":," }.to_owned());
    let separator_chars = regex::escape(&separator_chars);
    let separator_re = Regex::new(&format!("([^{separator_chars}]+)([{separator_chars}]*)"))
        .expect("Invalid ENVY_SEP");

    // Filter and print the environment variables that match the regex pattern
    let mut variables: Vec<_> = std::env::vars()
        .filter(|(key, _v)| pattern_re.is_match(&key))
        .collect();
    variables.sort();
    let variables = variables;
    let Palette {
        header,
        value,
        special,
        separator,
        reset,
    } = palette;

    for (env_key, mut env_value) in variables.into_iter() {
        println!("{header}{env_key}{reset}{separator}={reset}");
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
