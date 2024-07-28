use crate::cli::Cli;
use clap::{Command, CommandFactory};
use clap_complete::{generator::generate, Generator, Shell};

#[derive(Debug, clap::Args)]
pub struct CompletionArgs {
    #[arg(name = "shell", long = "shell", short)]
    shell: String,
}

fn get_command() -> Command {
    Cli::command()
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

fn get_shell_by_name(shell_name: &str) -> Option<Shell> {
    match shell_name {
        "zsh" => Some(Shell::Zsh),
        "bash" => Some(Shell::Bash),
        "fish" => Some(Shell::Fish),
        "elvish" => Some(Shell::Elvish),
        _ => None,
    }
}

pub fn cmd(args: CompletionArgs) {
    if let Some(shell) = get_shell_by_name(&args.shell) {
        let mut cmd = get_command();
        print_completions(shell, &mut cmd);
    } else {
        eprintln!("no such shell: {}", &args.shell);
    }
}
