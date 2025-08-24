use clap::Command;
use clap_complete::{
    generate,
    shells::{Bash, Fish, Zsh},
};

pub fn run_completions<S: AsRef<str>>(mut cmd: Command, shell: S) {
    let shell = shell.as_ref();
    match shell {
        "bash" => generate(Bash, &mut cmd, "adr-rag", &mut std::io::stdout()),
        "zsh" => generate(Zsh, &mut cmd, "adr-rag", &mut std::io::stdout()),
        "fish" => generate(Fish, &mut cmd, "adr-rag", &mut std::io::stdout()),
        _ => {
            eprintln!("Unsupported shell: {} (supported: bash|zsh|fish)", shell);
            std::process::exit(2);
        }
    }
}
