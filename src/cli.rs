/// Drag and Drop files to and from the terminal
#[derive(clap::Parser)]
#[clap(about)]
pub struct Cli {
    /// Be verbose
    #[clap(short, long, value_parser, default_value_t = false)]
    pub verbose: bool,

    /// Act as a target instead of source
    #[clap(short, long, value_parser, default_value_t = false)]
    pub target: bool,

    /// With --target, keep files to drag out
    #[clap(
        short,
        long,
        value_parser,
        default_value_t = false,
        requires = "target"
    )]
    pub keep: bool,

    /// With --target, keep files to drag out
    #[clap(
        short,
        long,
        value_parser,
        default_value_t = false,
        requires = "target"
    )]
    pub print_path: bool,

    /// Make the window resizable
    #[clap(short, long, value_parser, default_value_t = false)]
    pub resizable: bool,

    /// Exit after first successful drag or drop
    #[clap(short = 'x', long, value_parser, default_value_t = false)]
    pub and_exit: bool,

    /// Only display icons, no labels
    #[clap(short, long, value_parser, default_value_t = false)]
    pub icons_only: bool,

    /// Don't load thumbnails from images
    #[clap(short, long, value_parser, default_value_t = false)]
    pub disable_thumbnails: bool,

    /// Size of icons and thumbnails
    #[clap(short = 's', long, value_parser, default_value_t = 32)]
    pub icon_size: i32,

    /// Min width of the main window
    #[clap(short = 'w', long, value_parser, default_value_t = 360)]
    pub content_width: i32,

    /// Default height of the main window
    #[clap(short = 'h', long, value_parser, default_value_t = 360)]
    pub content_height: i32,

    /// Accept paths from stdin
    #[clap(short = 'I', long, value_parser, default_value_t = false)]
    pub from_stdin: bool,

    /// Drag all the items together
    #[clap(short = 'a', long, value_parser, default_value_t = false)]
    pub all: bool,

    /// Show only the number of items and drag them together
    #[clap(short = 'A', long, value_parser, default_value_t = false)]
    pub all_compact: bool,

    /// Paths to the files you want to drag
    #[clap(parse(from_os_str))]
    pub paths: Vec<std::path::PathBuf>,
}
