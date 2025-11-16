use std::{
    fmt::Display,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use anyhow::Result;
use capybar::{
    config::Config,
    root::{BarState, Root},
};
use clap::{Args, Parser, ValueEnum};
use std::env::var;
use thiserror::Error;
use wayland_client::{globals::registry_queue_init, Connection, EventQueue};

const SOCKET_PATH: &str = "/tmp/capybar-ipc";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    args: Arguments,

    request: Option<Request>,
}

#[derive(Debug)]
enum ControlMsg {
    Hide,
    Show,
    Toggle,
    Stop,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
#[value(rename_all = "snake_case")]
enum Request {
    Ping,
    GetTime,
    Hide,
    Show,
    Toggle,
    Stop,
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Request::*;
        match self {
            Ping => write!(f, "ping"),
            GetTime => write!(f, "get_time"),
            Hide => write!(f, "hide"),
            Show => write!(f, "show"),
            Toggle => write!(f, "toggle"),
            Stop => write!(f, "stop"),
        }
    }
}

impl std::str::FromStr for Request {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Request::*;
        match s.trim().to_ascii_lowercase().as_str() {
            "ping" => Ok(Ping),
            "get_time" => Ok(GetTime),
            "hide" => Ok(Hide),
            "show" => Ok(Show),
            "toggle" => Ok(Toggle),
            "stop" => Ok(Stop),
            other => Err(format!("unknown request: {}", other)),
        }
    }
}

#[derive(Debug, Args)]
struct Arguments {
    /// What config type to use
    #[arg(long, value_enum, default_value_t = ConfigTypes::Toml, value_name = "TYPE")]
    cfg_type: ConfigTypes,

    #[arg(long, value_name = "FILE")]
    /// Directory where the config is located
    cfg_path: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ConfigTypes {
    Toml,
}

impl Display for ConfigTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigTypes::Toml => write!(f, "toml"),
        }
    }
}

#[derive(Debug, Error)]
enum Errors {
    #[error(
        "Configuration file does not exist! 
        Make sure you are passing `--cfg_type <TYPE>` with correct type if it is not TOML.
        Make sure you provide '--cfg_path <PATH>' with your config file or \
        place it config at `~/.config/capybar/config.<TYPE>"
    )]
    ConfigNotExist,
}

fn start_bar(cli: Cli) -> Result<(Root, EventQueue<Root>)> {
    let mut cfg_path;
    match cli.args.cfg_path {
        None => {
            if let Ok(config_home) = var("XDG_CONFIG_HOME")
                .or_else(|_| var("HOME").map(|home| format!("{home}/.config")))
            {
                cfg_path = config_home.into();
            } else {
                return Err(Errors::ConfigNotExist.into());
            }
        }
        Some(value) => cfg_path = value,
    }

    if cfg_path.is_dir() {
        cfg_path.push("capybar");
        let file_name = "config.".to_string() + &cli.args.cfg_type.to_string();
        cfg_path.push(file_name);
    }

    if !cfg_path.exists() {
        return Err(Errors::ConfigNotExist.into());
    }

    let config = match cli.args.cfg_type {
        ConfigTypes::Toml => Config::parse_toml(cfg_path)?,
    };

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut capybar = Root::new(&globals, &mut event_queue, None)?;
    capybar.apply_config(config)?;

    Ok((capybar, event_queue))
}

fn run_client(request: Request) -> std::io::Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    println!("[Client] Connected to server.");

    writeln!(stream, "{request}")?;
    println!("[Client] Sent: '{}'", request);

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;
    println!("[Client] Received: '{}'", response.trim());
    Ok(())
}

fn handle_client(
    stream: UnixStream,
    shutdown: Arc<AtomicBool>,
    control_tx: mpsc::Sender<ControlMsg>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = &stream;

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let req = match request_line.parse::<Request>() {
        Ok(r) => r,
        Err(_) => {
            writer.write_all(b"ERROR: Unknown command\n")?;
            return Ok(());
        }
    };

    println!("[Server] Received request: '{}'", req);

    use Request::*;
    match req {
        Ping => {
            writer.write_all(b"PONG\n")?;
        }
        GetTime => {
            let response = format!("{}\n", chrono::Utc::now());
            writer.write_all(response.as_bytes())?;
        }
        Hide => {
            writer.write_all(b"OK: Hiding\n")?;
            let _ = control_tx.send(ControlMsg::Hide);
        }
        Show => {
            writer.write_all(b"OK: Showing\n")?;
            let _ = control_tx.send(ControlMsg::Show);
        }
        Toggle => {
            writer.write_all(b"OK: Toggling\n")?;
            let _ = control_tx.send(ControlMsg::Toggle);
        }
        Stop => {
            writer.write_all(b"OK: Shutting down\n")?;
            writer.flush()?;
            shutdown.store(true, Ordering::SeqCst);
            let _ = control_tx.send(ControlMsg::Stop);
            let _ = UnixStream::connect(SOCKET_PATH);
        }
    }

    Ok(())
}

fn run_server(shutdown: Arc<AtomicBool>, control_tx: mpsc::Sender<ControlMsg>) -> Result<()> {
    if fs::metadata(SOCKET_PATH).is_ok() {
        println!("[Server] A previous socket file was found. Removing it.");
        fs::remove_file(SOCKET_PATH)?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("[Server] Started, listening on {}", SOCKET_PATH);

    loop {
        match listener.accept() {
            Ok((stream, _addr)) => {
                if shutdown.load(Ordering::SeqCst) {
                    drop(stream);
                    break;
                }
                let shutdown_clone = Arc::clone(&shutdown);
                let tx = control_tx.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, shutdown_clone, tx) {
                        eprintln!("[Server] Error handling client: {}", e);
                    }
                });
            }
            Err(err) => {
                eprintln!("[Server] Error accepting connection: {}", err);
                break;
            }
        }
    }

    drop(listener);
    if let Err(e) = fs::remove_file(SOCKET_PATH) {
        eprintln!("[Server] Could not remove socket file: {}", e);
    }
    println!("[Server] Stopped.");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.request {
        Some(request) => {
            if let Err(e) = run_client(request) {
                eprintln!("[Client] Error: {}. Is the server running?", e);
            }
        }
        None => {
            let shutdown = Arc::new(AtomicBool::new(false));
            let (tx, rx) = mpsc::channel::<ControlMsg>();

            let server_shutdown = Arc::clone(&shutdown);
            let server_tx = tx.clone();
            let server_handle = thread::spawn(move || {
                if let Err(e) = run_server(server_shutdown.clone(), server_tx) {
                    server_shutdown.store(true, Ordering::SeqCst);
                    eprintln!("[Server] Fatal error: {}", e);
                }
            });

            println!("[Server] Server is running. Program will exit when \"stop\" is received.");
            println!("Write \"capybar stop\" to close the bar");

            let capybar_shown = Arc::new(AtomicBool::new(true));
            let capybar_shutdown = Arc::clone(&shutdown);
            let capybar_state = BarState {
                shutdown: Arc::clone(&shutdown),
                shown: Arc::clone(&capybar_shown),
            };
            let capybar_handle = thread::spawn(move || match start_bar(cli) {
                Ok((mut capybar, mut event_queue)) => {
                    if let Err(e) = capybar.run_async(&mut event_queue, capybar_state) {
                        capybar_shutdown.store(true, Ordering::SeqCst);
                        eprintln!("[Server] Fatal error: {}", e);
                    }
                }
                Err(e) => {
                    capybar_shutdown.store(true, Ordering::SeqCst);
                    eprintln!("[Server] Fatal error: {}", e);
                }
            });

            while !shutdown.load(Ordering::SeqCst) {
                match rx.recv_timeout(Duration::from_millis(1000)) {
                    Ok(ControlMsg::Hide) => {
                        capybar_shown.store(false, Ordering::SeqCst);
                    }
                    Ok(ControlMsg::Show) => {
                        capybar_shown.store(true, Ordering::SeqCst);
                    }
                    Ok(ControlMsg::Toggle) => {
                        capybar_shown
                            .store(!capybar_shown.load(Ordering::SeqCst), Ordering::SeqCst);
                    }
                    Ok(ControlMsg::Stop) => {
                        println!("[Capybar] Stop requested.");
                        shutdown.store(true, Ordering::SeqCst);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }

            println!("[Capybar] Waiting for server to exit...");
            let _ = server_handle.join();
            let _ = capybar_handle.join();
            println!("[Capybar] Goodbye.");
        }
    }
    Ok(())
}
