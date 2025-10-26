/// Macro to print colored help text
#[macro_export]
macro_rules! print_help {
    // Print header with bold text
    (header: $text:expr) => {{
        use colored::Colorize;
        println!("{}\n", $text.bold());
    }};
    
    // Print usage line
    (usage: $cmd:expr) => {{
        use colored::Colorize;
        println!("{} {}\n", "Usage:".bold(), $cmd);
    }};
    
    // Print section heading in green bold
    (section: $heading:expr) => {{
        use colored::Colorize;
        println!("{}", $heading.green().bold());
    }};
    
    // Print command with cyan bold name
    (command: $name:expr, $desc:expr) => {{
        use colored::Colorize;
        println!("  {}  {}", $name.cyan().bold(), $desc);
    }};
    
    // Print regular option line
    (option: $flags:expr, $desc:expr) => {
        println!("  {}  {}", $flags, $desc);
    };
    
    // Print footer text
    (footer: $text:expr) => {
        println!("{}", $text);
    };
}

// Colored heading constants for option groups
pub const PATH_OPTIONS_HEADING: &str = "\u{001b}[1;32mPath options\u{001b}[0m";
pub const ENCRYPTION_OPTIONS_HEADING: &str = "\u{001b}[1;32mEncryption options\u{001b}[0m";
pub const DECRYPTION_OPTIONS_HEADING: &str = "\u{001b}[1;32mDecryption options\u{001b}[0m";
pub const PERFORMANCE_OPTIONS_HEADING: &str = "\u{001b}[1;32mPerformance options\u{001b}[0m";
pub const OUTPUT_OPTIONS_HEADING: &str = "\u{001b}[1;32mOutput options\u{001b}[0m";
pub const SECURITY_OPTIONS_HEADING: &str = "\u{001b}[1;32mSecurity options\u{001b}[0m";
pub const FILE_OPTIONS_HEADING: &str = "\u{001b}[1;32mFile options\u{001b}[0m";
pub const KEY_OPTIONS_HEADING: &str = "\u{001b}[1;32mKey options\u{001b}[0m";
pub const GITHUB_OPTIONS_HEADING: &str = "\u{001b}[1;32mGitHub options\u{001b}[0m";
pub const PROVIDER_OPTIONS_HEADING: &str = "\u{001b}[1;32mProvider options\u{001b}[0m";
pub const INPUT_HEADING: &str = "\u{001b}[1;32mInput\u{001b}[0m";