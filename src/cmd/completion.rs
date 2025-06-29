use crate::opt::Opt;
use std::io;
use structopt::clap::Shell;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "Generate shell completion scripts")]
#[structopt(setting(structopt::clap::AppSettings::ColoredHelp))]
pub struct CompletionOpts {
    #[structopt(
        help = "Shell to generate completions for",
        possible_values = &["bash", "zsh", "fish", "powershell", "elvish"]
    )]
    pub shell: Shell,
}

pub fn run(opts: CompletionOpts) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Opt::clap();
    let app_name = app.get_name().to_string();
    app.gen_completions_to(app_name, opts.shell, &mut io::stdout());
    Ok(())
}