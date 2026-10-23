use clap::{Parser, ValueEnum};

fn print_uptime(uptime: u64) {
    let d: u64 = uptime / (24 * 60 * 60);
    let h: u64 = uptime / (60 * 60) - 24 * d;
    let m: u64 = uptime / (60) - 24 * 60 * d - 60 * h;
    let s: u64 = uptime - 24 * 60 * 60 * d - 60 * 60 * h - 60 * m;

    if d > 0 {
        print!("{d}d ");
    }
    if h > 0 {
        print!("{h}h ");
    }
    if m > 0 {
        print!("{m}m ");
    }
    if s > 0 || uptime == 0 {
        print!("{s}s");
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
#[clap(rename_all = "kebab-case")]
enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
}

impl Color {
    fn ansi(self) -> &'static str {
        match self {
            Color::Black => "\x1b[30m",
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Purple => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[39m",
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "uptime-record",
    about = "Keep track of your highest uptime!",
    after_help = "Report a bug: https://github.com/alba4k/uptime-record-rs/issues"
)]
struct Args {
    /// Don't print most output (update-only)
    #[arg(short, long)]
    silent: bool,

    /// Print some debug info
    #[arg(short, long)]
    verbose: bool,

    /// Change the output color
    #[arg(short, long, value_enum, default_value_t = Color::Green)]
    color: Color,

    /// Keep running at a set interval in seconds.
    /// Defaults to 60s if passed without value
    #[arg(
        short,
        long,
        num_args = 0..=1,
        default_missing_value = "60",
        value_name = "SECONDS"
    )]
    background: Option<u64>,
}

fn main() {
    let args = Args::parse();

    let color = args.color.ansi();
    let reset: &str = "\x1b[0m";

    if args.verbose {
        println!("{color}[DEBUG]{reset} Verbose mode enabled\x1b[0m");
    }

    let uptime: u64 = sysinfo::System::uptime();

    let mut path: std::path::PathBuf = dirs::data_dir().unwrap_or_else(|| {
        eprintln!("Data directory not found");
        std::process::exit(1);
    });
    path.push("uptime-record");

    if args.verbose {
        println!("{color}[DEBUG]{reset} Using path {}", path.display());
    }

    let max_uptime: u64 = std::fs::read_to_string(path.clone())
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0);

    if uptime > max_uptime {
        std::fs::write(path.clone(), uptime.to_string().as_bytes()).unwrap_or_else(|e| {
            eprintln!("Couldn't write to storage file: {e}");
            std::process::exit(1);
        });
    }

    if !args.silent {
        print!("Current uptime: ");
        print_uptime(uptime);
        if uptime >= max_uptime {
            print!("\n{color}This is your highest uptime!{reset}\n\nPrevious highest: ");
        } else {
            print!("\n\n{color}Highest uptime{reset}: ");
        }
        print_uptime(max_uptime);
        println!();
    }

    let interval = args.background.unwrap_or_else(|| std::process::exit(0));

    if !args.silent {
        println!("{color}Running in background every {interval}s...\x1b[0m");
    }

    loop {
        std::thread::sleep(std::time::Duration::new(interval, 0));

        let uptime: u64 = sysinfo::System::uptime();

        if args.verbose {
            println!("{color}[DEBUG]{reset} Checking for updates at uptime {uptime}");
        }

        if uptime > max_uptime {
            std::fs::write(path.clone(), uptime.to_string().as_bytes()).unwrap_or_else(|e| {
                eprintln!("Couldn't write to storage file: {e}");
                std::process::exit(1);
            });
            if args.verbose {
                println!("{color}[DEBUG]{reset} Updated highest uptime");
            }
        }
    }
}
