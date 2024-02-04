use crate::{
    history::{self, History},
    QuartzResult,
};
use colored::Colorize;

pub fn cmd(
    max_count: Option<usize>,
    date: Option<String>,
    show_fields: Vec<String>,
) -> QuartzResult {
    let history = History::new()?;
    let mut count = 0;
    let max_count = max_count.unwrap_or(usize::MAX);
    let format = date.unwrap_or(history::DEFAULT_DATE_FORMAT.into());

    for mut entry in history {
        entry.date_format(format.clone());

        if count >= max_count {
            break;
        }

        count += 1;
        if count != 1 {
            println!();
        }

        if show_fields.is_empty() {
            println!("{entry}");
            continue;
        }

        let mut outputs: Vec<String> = Vec::new();
        for key in &show_fields {
            let value = entry
                .field_as_string(key)
                .unwrap_or_else(|_| panic!("invalid field: {}", key.red()));

            outputs.push(value);
        }

        for value in outputs {
            println!("{}", value);
        }
    }

    Ok(())
}
