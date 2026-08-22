use cellsight_theme::Theme;
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("theme-ctl: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let path = args.next().ok_or_else(|| usage().to_owned())?;
    let key = args.next();
    if args.next().is_some() {
        return Err(usage().to_owned());
    }

    let theme = Theme::load(&path).map_err(|error| error.to_string())?;
    if let Some(key) = key {
        let color = theme
            .color(&key)
            .ok_or_else(|| format!("unknown theme value `{key}`"))?;
        println!("#{color:06x}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&theme).map_err(|error| error.to_string())?
        );
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: theme-ctl <theme.json> [color-name]"
}
