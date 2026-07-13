use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;

fn main() -> Result<()> {
    serve_web()
}

fn serve_web() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("playground must be inside the workspace")
        .to_path_buf();
    let site = root.join("target").join("lab-site");
    let pkg = site.join("pkg");

    println!("Building WebAssembly playground...");
    run_command(
        Command::new("cargo")
            .args([
                "build",
                "-p",
                "picocaml-playground",
                "--release",
                "--target",
                "wasm32-unknown-unknown",
                "--lib",
            ])
            .current_dir(&root),
    )?;

    if !command_exists("wasm-bindgen") {
        anyhow::bail!(
            "wasm-bindgen is required. Install the version pinned in Cargo.lock; see the README for the setup command"
        );
    }

    fs::create_dir_all(&pkg)?;
    for file in ["index.html", "app.js", "style.css"] {
        fs::copy(root.join("playground/web").join(file), site.join(file))?;
    }
    run_command(
        Command::new("wasm-bindgen")
            .arg(root.join("target/wasm32-unknown-unknown/release/picocaml_playground.wasm"))
            .args(["--target", "web", "--out-dir"])
            .arg(&pkg),
    )?;

    let listener = TcpListener::bind("127.0.0.1:8000")?;
    println!("\n▶ picocaml lab is running at http://localhost:8000");
    println!("  Press Ctrl-C to stop.\n");
    open_browser("http://localhost:8000");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_http(stream, &site) {
                    eprintln!("HTTP error: {error}");
                }
            }
            Err(error) => eprintln!("Connection error: {error}"),
        }
    }

    Ok(())
}

fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let command = "open";
    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    #[cfg(target_os = "windows")]
    let command = "start";

    let _ = Command::new(command).arg(url).spawn();
}

fn run_command(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        anyhow::bail!("command failed: {command:?}");
    }
    Ok(())
}

fn command_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn handle_http(mut stream: TcpStream, site: &Path) -> Result<()> {
    let mut buffer = [0; 8192];
    let size = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..size]);
    let requested = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let relative = requested
        .split('?')
        .next()
        .unwrap_or("/")
        .trim_start_matches('/');
    let relative = if relative.is_empty() {
        "index.html"
    } else {
        relative
    };
    let path = site.join(relative);

    let (status, content_type, body) = if path.starts_with(site) && path.is_file() {
        let content_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "text/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("wasm") => "application/wasm",
            _ => "application/octet-stream",
        };
        ("200 OK", content_type, read_file(&path)?)
    } else {
        (
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"Not found".to_vec(),
        )
    };

    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)?;
    Ok(())
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    File::open(path)?.read_to_end(&mut body)?;
    Ok(body)
}
