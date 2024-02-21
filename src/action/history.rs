use crate::{history::History, Ctx, QuartzResult};

pub struct Args {
    pub max_count: Option<usize>,
}

pub fn cmd(ctx: &Ctx, args: Args) -> QuartzResult {
    let history = History::new(ctx)?;
    let mut count = 0;
    let max_count = args.max_count.unwrap_or(usize::MAX);

    for entry in history.entries(ctx) {
        if count >= max_count {
            break;
        }

        count += 1;
        if count != 1 {
            // Separation between two entries
            println!();
        }

        println!("{entry}")
    }

    Ok(())
}
