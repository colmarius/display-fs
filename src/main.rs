use clap::{Parser, Subcommand, ValueEnum};
use display_fs::{
    calculate_auto_fit_size, calculate_max_chars_per_line, create_text_image, find_display_port,
    get_now_playing, image_to_rgb565_bytes, open_connection, send_image_to_display,
    split_into_pages, DisplayConfig, DisplayModel, NowPlaying, Orientation, PortInfo,
    MIN_FONT_SIZE,
};
use serialport::SerialPort;
use std::process::{Command, ExitCode};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "display-fs")]
#[command(about = "Display text on WeAct Studio Display FS V1 (0.96 inch + 3.5 inch)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a built-in preset demo
    Preset {
        /// Preset name to run
        #[arg(value_enum)]
        name: PresetName,

        #[command(flatten)]
        display: DisplayOptions,
    },
    /// List all available presets
    Presets,
    /// Demo mode: cycle through all presets in a loop
    Demo {
        #[command(flatten)]
        display: DisplayOptions,
    },
    /// Display text on the screen
    Show(ShowArgs),
    /// Show currently playing Spotify track
    Spotify(SpotifyArgs),
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum OrientationArg {
    /// 160x80 - wider than tall (default)
    #[default]
    Landscape,
    /// 80x160 - taller than wide
    Portrait,
}

impl OrientationArg {
    fn to_orientation(self, flip: bool) -> Orientation {
        match (self, flip) {
            (OrientationArg::Landscape, true) => Orientation::LandscapeFlip,
            (OrientationArg::Landscape, false) => Orientation::Landscape,
            (OrientationArg::Portrait, true) => Orientation::PortraitFlip,
            (OrientationArg::Portrait, false) => Orientation::Portrait,
        }
    }
}

#[derive(clap::Args, Clone)]
struct DisplayOptions {
    /// Font size in pixels (must be finite and positive)
    #[arg(
        short = 's',
        long,
        default_value = "14",
        value_parser = validate_positive_f32,
        allow_hyphen_values = true
    )]
    font_size: f32,

    /// Auto-fit text to largest readable size
    #[arg(short = 'a', long)]
    auto: bool,

    /// Display orientation
    #[arg(short = 'o', long, value_enum, default_value = "landscape")]
    orientation: OrientationArg,

    /// Flip the display 180° (use if the screen is upside down)
    #[arg(long)]
    flip: bool,

    /// Delay between pages/updates in seconds (must be finite and positive)
    #[arg(
        short,
        long,
        default_value = "2.0",
        value_parser = validate_positive_f32,
        allow_hyphen_values = true
    )]
    delay: f32,

    /// Loop display continuously (until Ctrl+C)
    #[arg(short, long)]
    r#loop: bool,

    /// Speed preset (overrides --delay if provided)
    #[arg(long, value_enum)]
    speed: Option<SpeedPreset>,

    /// Force display model (small = 0.96 inch, large = 3.5 inch)
    #[arg(long, value_enum)]
    model: Option<DisplayModelArg>,

    /// Override baud rate (advanced)
    #[arg(long)]
    baud_rate: Option<u32>,

    /// Use a specific serial port instead of USB auto-detection
    /// (model defaults to small unless --model is given)
    #[arg(short = 'p', long, value_name = "PATH")]
    port: Option<String>,
}

impl DisplayOptions {
    pub fn effective_delay(&self) -> f32 {
        self.speed.map_or(self.delay, |s| s.to_delay())
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation.to_orientation(self.flip)
    }

    pub fn override_config(&self, base: DisplayConfig) -> DisplayConfig {
        let mut config = base;
        if let Some(model) = self.model {
            config = model.to_model().config();
        }
        if let Some(baud_rate) = self.baud_rate {
            config.baud_rate = baud_rate;
        }
        config
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum DisplayModelArg {
    /// 0.96-inch display
    Small,
    /// 3.5-inch display
    Large,
}

impl DisplayModelArg {
    fn to_model(self) -> DisplayModel {
        match self {
            DisplayModelArg::Small => DisplayModel::Small,
            DisplayModelArg::Large => DisplayModel::Large,
        }
    }
}

#[derive(clap::Args)]
struct ShowArgs {
    /// Text to display (default: "Hello World!")
    #[arg(default_value = "Hello World!")]
    text: String,

    /// Only check if display is connected
    #[arg(long)]
    detect: bool,

    /// Display once only (default behavior)
    #[arg(long, conflicts_with = "loop")]
    once: bool,

    #[command(flatten)]
    display: DisplayOptions,
}

#[derive(clap::Args)]
struct SpotifyArgs {
    #[command(flatten)]
    display: DisplayOptions,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum PresetName {
    /// Current time (HH:MM:SS)
    Clock,
    /// Current date and time
    #[value(name = "datetime")]
    DateTime,
    /// System uptime
    Uptime,
    /// Current git branch and status
    Git,
    /// Local IP address
    Ip,
    /// Username and hostname
    Whoami,
    /// Current working directory
    Pwd,
    /// CPU usage percentage (macOS)
    Cpu,
    /// Memory pressure (macOS)
    Memory,
    /// Docker container count
    Docker,
    /// Now playing from Spotify (macOS)
    Spotify,
    /// Random fortune cookie
    Fortune,
}

impl PresetName {
    /// Returns (description, shell command)
    pub fn info(self) -> (&'static str, &'static str) {
        match self {
            PresetName::Clock => ("Current time", "date '+%H:%M:%S'"),
            PresetName::DateTime => ("Date and time", "date '+%Y-%m-%d %H:%M'"),
            PresetName::Uptime => ("System uptime", "uptime | awk '{print $3, $4}' | sed 's/,$//'"),
            PresetName::Git => (
                "Git branch & status",
                "echo \"$(git branch --show-current 2>/dev/null || echo 'no repo'): $(git status --short 2>/dev/null | wc -l | tr -d ' ') changes\"",
            ),
            PresetName::Ip => (
                "Local IP address",
                "echo \"IP: $(ipconfig getifaddr en0 2>/dev/null || hostname -I 2>/dev/null | awk '{print $1}' || echo 'N/A')\"",
            ),
            PresetName::Whoami => ("Username @ hostname", "echo \"$(whoami)@$(hostname -s)\""),
            PresetName::Pwd => ("Current directory", "basename \"$PWD\""),
            PresetName::Cpu => (
                "CPU usage (macOS)",
                "top -l 1 -n 0 | grep 'CPU usage' | awk '{print \"CPU: \" $3}'",
            ),
            PresetName::Memory => (
                "Memory pressure (macOS)",
                "memory_pressure 2>/dev/null | grep 'System-wide' | awk '{print \"Mem: \" $NF}' || echo 'Mem: N/A'",
            ),
            PresetName::Docker => (
                "Docker containers",
                "echo \"Docker: $(docker ps -q 2>/dev/null | wc -l | tr -d ' ') running\"",
            ),
            PresetName::Spotify => (
                "Spotify now playing (macOS)",
                "osascript -e 'tell application \"Spotify\" to if player state is playing then name of current track else \"Not playing\"' 2>/dev/null || echo 'Spotify N/A'",
            ),
            PresetName::Fortune => ("Random fortune", "fortune -s 2>/dev/null || echo 'Install fortune'"),
        }
    }

    pub fn run_command(self) -> String {
        let (_, cmd) = self.info();
        match Command::new("sh").arg("-c").arg(cmd).output() {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if result.is_empty() {
                    String::from_utf8_lossy(&output.stderr).trim().to_string()
                } else {
                    result
                }
            }
            Err(e) => format!("Error: {}", e),
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum SpeedPreset {
    /// 4 seconds between pages
    Slow,
    /// 2 seconds between pages
    Normal,
    /// 1 second between pages
    Fast,
}

impl SpeedPreset {
    pub fn to_delay(self) -> f32 {
        match self {
            SpeedPreset::Slow => 4.0,
            SpeedPreset::Normal => 2.0,
            SpeedPreset::Fast => 1.0,
        }
    }
}

fn validate_positive_f32(s: &str) -> Result<f32, String> {
    let value: f32 = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;
    if !value.is_finite() || value <= 0.0 {
        Err("value must be a finite positive number".to_string())
    } else {
        Ok(value)
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Preset { name, display }) => run_preset(name, display),
        Some(Commands::Presets) => list_presets(),
        Some(Commands::Demo { display }) => run_demo(display),
        Some(Commands::Show(args)) => run_show(args),
        Some(Commands::Spotify(args)) => run_spotify(args),
        None => {
            // Default: show help
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
            ExitCode::SUCCESS
        }
    }
}

fn run_show(args: ShowArgs) -> ExitCode {
    if args.detect {
        return detect_display();
    }

    display_text(&args.text, &args.display)
}

fn list_presets() -> ExitCode {
    println!("Available presets:\n");

    for preset in ALL_PRESETS {
        let (desc, _) = preset.info();
        // Use clap's value name so the listing always matches what `preset <NAME>` accepts.
        let name = preset
            .to_possible_value()
            .expect("preset variants are never skipped")
            .get_name()
            .to_string();
        println!("  {:12} - {}", name, desc);
    }

    println!("\nUsage:");
    println!("  display-fs preset <NAME>    Run a single preset");
    println!("  display-fs demo             Cycle through all presets");
    println!("\nExamples:");
    println!("  display-fs preset clock");
    println!("  display-fs demo --delay 3");
    ExitCode::SUCCESS
}

fn run_preset(name: PresetName, display: DisplayOptions) -> ExitCode {
    let (desc, _) = name.info();
    println!("Running preset: {}", desc);

    let text = name.run_command();
    println!("Output: {}", text);

    display_text(&text, &display)
}

const ALL_PRESETS: [PresetName; 12] = [
    PresetName::Clock,
    PresetName::DateTime,
    PresetName::Uptime,
    PresetName::Git,
    PresetName::Ip,
    PresetName::Whoami,
    PresetName::Pwd,
    PresetName::Cpu,
    PresetName::Memory,
    PresetName::Docker,
    PresetName::Spotify,
    PresetName::Fortune,
];

fn run_demo(display: DisplayOptions) -> ExitCode {
    let delay = display.effective_delay();
    let orientation = display.orientation();
    println!("Demo mode: cycling through all presets (Ctrl+C to stop)");
    println!(
        "Delay: {}s between presets, orientation: {:?}\n",
        delay, orientation
    );

    let (display_config, mut connection) = match connect(&display) {
        Some(c) => c,
        None => return ExitCode::FAILURE,
    };

    let delay_duration = Duration::from_secs_f32(delay);

    loop {
        for preset in ALL_PRESETS {
            let (desc, _) = preset.info();
            let text = preset.run_command();
            println!("[{}] {}", desc, text);

            let font_size = resolve_font_size(&display, &text, orientation, display_config);
            if !render_and_send(
                &mut connection,
                display_config,
                orientation,
                &text,
                font_size,
            ) {
                return ExitCode::FAILURE;
            }

            thread::sleep(delay_duration);
        }
    }
}

/// Find the display, apply CLI overrides, and open the serial connection.
/// Prints progress and errors; returns None when no usable display is available.
fn connect(display: &DisplayOptions) -> Option<(DisplayConfig, Box<dyn SerialPort>)> {
    let mut port_info = match &display.port {
        Some(path) => {
            let model = display
                .model
                .map_or(DisplayModel::Small, DisplayModelArg::to_model);
            PortInfo {
                name: path.clone(),
                vid: 0,
                pid: 0,
                model,
                baud_rate: model.config().baud_rate,
                product: None,
                manufacturer: None,
            }
        }
        None => match find_display_port() {
            Some(p) => p,
            None => {
                println!("✗ Display FS V1 not found");
                println!("  Make sure the display is connected via USB-C");
                println!("  and the CH340/CH341 driver is installed.");
                return None;
            }
        },
    };

    let display_config = display.override_config(port_info.model.config());
    println!("✓ Using display on {}", port_info.name);
    println!(
        "Opening connection to {} at {} baud...",
        port_info.name, display_config.baud_rate
    );

    port_info.baud_rate = display_config.baud_rate;
    match open_connection(&port_info) {
        Ok(connection) => Some((display_config, connection)),
        Err(e) => {
            println!("✗ Failed to open connection: {}", e);
            None
        }
    }
}

/// Auto-fit the font size when --auto is set, else use the configured size.
fn resolve_font_size(
    display: &DisplayOptions,
    text: &str,
    orientation: Orientation,
    config: DisplayConfig,
) -> f32 {
    if !display.auto {
        return display.font_size;
    }

    let size =
        calculate_auto_fit_size(text, orientation, config.width as u32, config.height as u32);
    println!("Auto-fit font size: {:.1}", size);
    size
}

/// Render text and send the frame; prints the error and returns false on failure.
fn render_and_send(
    connection: &mut Box<dyn SerialPort>,
    config: DisplayConfig,
    orientation: Orientation,
    text: &str,
    font_size: f32,
) -> bool {
    let (width, height) = (config.width as u32, config.height as u32);
    let img = create_text_image(text, font_size, orientation, width, height);
    let image_data = image_to_rgb565_bytes(&img, orientation, width, height);

    match send_image_to_display(connection, config, &image_data, orientation) {
        Ok(()) => true,
        Err(e) => {
            println!("✗ Failed to send image: {}", e);
            false
        }
    }
}

fn run_spotify(args: SpotifyArgs) -> ExitCode {
    let orientation = args.display.orientation();

    let (display_config, mut connection) = match connect(&args.display) {
        Some(c) => c,
        None => return ExitCode::FAILURE,
    };

    // None = nothing shown yet; Some(None) = "Spotify not running" shown.
    let mut last_shown: Option<Option<NowPlaying>> = None;
    let interval = Duration::from_secs_f32(args.display.effective_delay());

    loop {
        let now_playing = get_now_playing();

        if last_shown.as_ref() != Some(&now_playing) {
            let max_line_len = spotify_max_line_len(&args.display, orientation, display_config);
            let text = match &now_playing {
                Some(np) => format_spotify_text(&np.track, &np.artist, np.is_playing, max_line_len),
                None => "Spotify not running".to_string(),
            };
            let font_size = resolve_font_size(&args.display, &text, orientation, display_config);

            if !render_and_send(
                &mut connection,
                display_config,
                orientation,
                &text,
                font_size,
            ) {
                return ExitCode::FAILURE;
            }

            println!("{}", text.replace('\n', " "));
            last_shown = Some(now_playing);
        }

        if !args.display.r#loop {
            break;
        }

        thread::sleep(interval);
    }

    ExitCode::SUCCESS
}

fn spotify_max_line_len(
    display: &DisplayOptions,
    orientation: Orientation,
    config: DisplayConfig,
) -> usize {
    let font_size = if display.auto {
        MIN_FONT_SIZE
    } else {
        display.font_size
    };

    calculate_max_chars_per_line(
        font_size,
        orientation,
        config.width as u32,
        config.height as u32,
    )
}

fn format_spotify_text(track: &str, artist: &str, is_playing: bool, max_len: usize) -> String {
    let prefix = if is_playing { "♪" } else { "||" };
    let prefix_len = prefix.chars().count();
    let track_line = format!(
        "{} {}",
        prefix,
        trim_to_width(track, max_len.saturating_sub(prefix_len + 1))
    );
    let artist_line = format!("by {}", trim_to_width(artist, max_len.saturating_sub(3)));

    format!("{}\n{}", track_line, artist_line)
}

fn trim_to_width(text: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }

    let text_len = text.chars().count();
    if text_len <= max_len {
        return text.to_string();
    }

    if max_len <= 3 {
        return text.chars().take(max_len).collect();
    }

    let truncated: String = text.chars().take(max_len - 3).collect();
    format!("{}...", truncated)
}

fn detect_display() -> ExitCode {
    println!("Looking for Display FS V1...");

    match find_display_port() {
        Some(port) => {
            println!("✓ Found display on {}", port.name);
            println!("  VID: {:04X}, PID: {:04X}", port.vid, port.pid);
            println!("  Model: {:?}", port.model);
            ExitCode::SUCCESS
        }
        None => {
            println!("✗ Display FS V1 not found");
            println!("  Make sure the display is connected via USB-C");
            println!("  and the CH340/CH341 driver is installed.");
            ExitCode::FAILURE
        }
    }
}

fn display_text(text: &str, display: &DisplayOptions) -> ExitCode {
    let delay = display.effective_delay();
    let loop_mode = display.r#loop;
    let orientation = display.orientation();

    println!("Looking for Display FS V1...");

    let (display_config, mut connection) = match connect(display) {
        Some(c) => c,
        None => return ExitCode::FAILURE,
    };

    let font_size = resolve_font_size(display, text, orientation, display_config);

    let pages = split_into_pages(
        text,
        font_size,
        orientation,
        display_config.width as u32,
        display_config.height as u32,
    );
    let pages = if pages.is_empty() {
        vec![text.to_string()]
    } else {
        pages
    };

    let page_count = pages.len();
    let needs_delay = page_count > 1 || loop_mode;

    println!(
        "Text split into {} page(s) (font size: {}, {:?})",
        page_count, font_size, orientation
    );

    let delay_duration = Duration::from_secs_f32(delay);

    loop {
        for (i, page) in pages.iter().enumerate() {
            if page_count > 1 {
                println!("Displaying page {}/{}...", i + 1, page_count);
            }

            if !render_and_send(
                &mut connection,
                display_config,
                orientation,
                page,
                font_size,
            ) {
                return ExitCode::FAILURE;
            }

            if page_count == 1 && !loop_mode {
                println!("✓ Image sent successfully!");
                println!();
                println!("'{}' should now be displayed!", text);
            }

            if needs_delay {
                let is_last_page = i == page_count - 1;
                if !is_last_page || loop_mode {
                    thread::sleep(delay_duration);
                }
            }
        }

        if !loop_mode {
            break;
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_float_parser_rejects_non_finite_and_non_positive_values() {
        for value in ["0", "-1", "NaN", "inf", "-inf"] {
            assert!(
                validate_positive_f32(value).is_err(),
                "{value} should be rejected"
            );
        }
    }

    #[test]
    fn cli_validates_font_size_and_delay_before_accessing_hardware() {
        for option in ["--font-size", "--delay"] {
            for value in ["-1", "NaN", "inf"] {
                assert!(
                    Cli::try_parse_from(["display-fs", "show", option, value]).is_err(),
                    "{option} {value} should be rejected"
                );
            }
        }
    }
}
