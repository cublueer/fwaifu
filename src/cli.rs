use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "fwaifu",
    about = "A random anime girl generator for your terminal, powered by Fastfetch",
    version
)]
pub struct Cli {
    /// Enable NSFW mode
    #[arg(short = 'n', long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    pub nsfw: bool,

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

    /// Trailing positional args passed through to fastfetch
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub fastfetch_args: Vec<String>,
}
