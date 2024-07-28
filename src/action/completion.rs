use crate::cli::Cli;
use clap::{Command, CommandFactory};
use clap_complete::{generator::generate, Generator, Shell};

#[derive(clap::ValueEnum, Clone, Debug)]
enum AvailableCompletionShell {
    Zsh,
    Bash,
    Fish,
    Elvish,
}

impl Into<Shell> for AvailableCompletionShell {
    fn into(self) -> Shell {
        match self {
            AvailableCompletionShell::Zsh => Shell::Zsh,
            AvailableCompletionShell::Bash => Shell::Bash,
            AvailableCompletionShell::Fish => Shell::Fish,
            AvailableCompletionShell::Elvish => Shell::Elvish,
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct CompletionArgs {
    #[arg(name = "shell", long = "shell", short)]
    shell: AvailableCompletionShell,
}

fn get_command() -> Command {
    Cli::command()
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}

pub fn cmd(args: CompletionArgs) {
    let mut cmd = get_command();
    let shell: Shell = args.shell.into();
    print_completions(shell, &mut cmd);
}
