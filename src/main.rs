fn main() {
    let args = Args::parse();

    let filename = args
        .filename
        .clone()
        .unwrap_or(filename_from_name(&args.name));
    let entry_contents = make_desktop_entry_string(&args);

    let mut path = get_xdg_applications_dir(args.global);
    if !path.exists() {
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|_| panic!("could not create directory {}", path.to_str().unwrap()));
    }

    path.push(filename);

    let mut file = std::fs::File::create_new(&path)
        .or_else(|err| {
            match &err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!(
                        "Permission denied for file {}. Aborting.",
                        path.to_str().unwrap()
                    );
                    std::process::exit(1);
                }
                std::io::ErrorKind::AlreadyExists => {}
                _ => return Err(err),
            };

            eprint!(
                "The file {} already exists. Overwrite (y/n)? ",
                path.to_str().unwrap()
            );

            let mut answer = String::new();
            std::io::stdin()
                .read_line(&mut answer)
                .expect("unexpected I/O error reading stdin");

            if answer.to_ascii_lowercase().trim() != "y" {
                eprintln!("Aborting.");
                std::process::exit(1);
            }

            std::fs::File::create(&path)
        })
        .expect("failed to open file for writing");

    file.write_all(entry_contents.as_bytes())
        .expect("unexpected I/O error writing to file");

    eprintln!("Successfully created {}", path.to_str().unwrap());
}

fn filename_from_name(app_name: &str) -> String {
    let sanitized: String = app_name
        .chars()
        .filter_map(|mut c| {
            if c.is_whitespace() {
                c = '_';
            }

            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                Some(c)
            } else {
                None
            }
        })
        .collect();
    sanitized + ".desktop"
}

fn make_desktop_entry_string(args: &Args) -> String {
    let required = format!(
        r#"#!/usr/bin/env xdg-open
[Desktop Entry]
Type={}
Name={}
Exec={}
"#,
        args.entry_type, args.name, args.exec
    );

    let optional = match &args.icon {
        Some(icon_path) => format!("Icon={}\n", icon_path.to_str().unwrap()),
        None => "".into(),
    };

    required + &optional
}

fn get_xdg_applications_dir(global: bool) -> std::path::PathBuf {
    if global {
        // TODO: Is this path guaranteed?
        "/usr/share/applications".into()
    } else {
        let mut path = xdg::BaseDirectories::new()
            .expect("Could not get XDG base directories.")
            .get_data_home();
        path.push("applications");
        path
    }
}

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    after_help = "Note: The 'link' and 'directory' entry types have not been tested.",
    long_about = None
)]
struct Args {
    /// The desktop entry name (shown e.g. in the application menu)
    #[arg(long)]
    name: String,

    /// The type of desktop entry
    #[arg(long = "type")]
    entry_type: DesktopEntryType,

    /// The command line to be executed when this entry is launched
    #[arg(long)]
    exec: String,

    /// The image file path to set as the icon
    #[arg(long)]
    icon: Option<std::path::PathBuf>,

    /// If set, the entry will be created globally for all users —
    /// this may require elevated privileges (sudo)
    #[arg(short, long)]
    global: bool,

    /// The desktop entry filename
    /// (If the file exists, mkdesktop will ask for confirmation before overwriting.
    /// When unspecified, the filename is (safely) derived from the entry name.)
    filename: Option<String>,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, ValueEnum)]
enum DesktopEntryType {
    Application,
    Link,
    Directory,
}

impl std::fmt::Display for DesktopEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DesktopEntryType::Application => "Application",
                DesktopEntryType::Directory => "Directory",
                DesktopEntryType::Link => "Link",
            }
        )
    }
}

use std::io::Write;

use clap::{Parser, ValueEnum};
