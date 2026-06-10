use clap::Parser;
use clap::ValueEnum;

#[derive(Parser, Debug)]
#[command(
    name = "fwaifu",
    about = "A random anime girl generator for your terminal, powered by Fastfetch and chafa",
    version
)]
pub struct Cli {
    /// Enable NSFW mode
    #[arg(short = 'n', long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub nsfw: bool,

    /// Use chafa to display the image in the terminal
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub term: bool,

    /// Width of the chafa output in characters (default: auto-detect, 1/3 of terminal width)
    #[arg(long)]
    pub term_width: Option<u32>,

    /// Enable watch/loop mode
    #[arg(short = 'w', long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub watch: bool,

    /// Watch interval in seconds
    #[arg(long)]
    pub watch_interval: Option<u64>,

    /// Proxy URL
    #[arg(short = 'p', long)]
    pub proxy: Option<String>,

    /// Disable image cropping
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub no_crop: bool,

    /// Crop width
    #[arg(long)]
    pub crop_width: Option<u32>,

    /// Crop height
    #[arg(long)]
    pub crop_height: Option<u32>,

    /// Fastfetch logo display width
    #[arg(long)]
    pub logo_width: Option<u32>,

    /// Interactive login to Nekos.moe
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub login: bool,

    /// Clear stored token
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub logout: bool,

    /// Show login status
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub status: bool,

    /// Internal: run as background stock replenisher daemon
    #[arg(long, hide = true, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub daemon: bool,

    /// Clear cache (sfw, nsfw, or all if no value given)
    #[arg(long, num_args = 0..=1, default_missing_value = "__DEFAULT__")]
    pub clean: Option<String>,

    /// Save the last displayed image (optional path, defaults to config or ~/Pictures/fwaifu)
    #[arg(short = 's', long, num_args = 0..=1, default_missing_value = "__DEFAULT__")]
    pub save: Option<String>,

    /// Check for updates and prompt to install
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub update: bool,

    /// Generate shell completion script for the given shell
    #[arg(long, value_enum)]
    pub completion: Option<Shell>,

    /// Trailing positional args passed through to fastfetch
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub fastfetch_args: Vec<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}
