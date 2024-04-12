use clap::Parser;
use std::process::Command;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    cmd: Subcommand,
}

#[derive(Debug, Parser)]
enum Subcommand {
    #[clap(name = "wasm-opt")]
    WasmOpt,
    #[clap(name = "wasm-deploy")]
    WasmDeploy
}

fn main() {
    let app = Cli::parse();

    let app_result = match app.cmd {
        Subcommand::WasmOpt => wasm_opt(),
        Subcommand::WasmDeploy => wasm_deploy(),
    };

    if let Err(e) = app_result {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}

fn wasm_opt() -> anyhow::Result<()> {
    // 1) Build wasm
    // 2) wasm-bindgen
    // 3) wasm-opt

    println!("xtask/wasm-opt => Building wasm...");
    Command::new("cargo").arg("wasm-build").spawn()?.wait()?;

    println!("xtask/wasm-opt => Running wasm-bindgen...");
    Command::new("wasm-bindgen")
        .args([
            "--out-dir",
            "./target/wasm32-unknown-unknown/release-wasm/opt",
            "--target",
            "web",
            "./target/wasm32-unknown-unknown/release-wasm/rust-snake.wasm",
        ])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-opt => Running wasm-opt...");
    Command::new("wasm-opt")
        .args([
            "-Oz",
            "-o",
            "./target/wasm32-unknown-unknown/release-wasm/opt/rust-snake_bg.wasm", 
            "./target/wasm32-unknown-unknown/release-wasm/opt/rust-snake_bg.wasm"
        ])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-opt => Done!");

    Ok(())
}

fn wasm_deploy() -> anyhow::Result<()> {
    println!("*** This command uses specific paths that might not work on other machines!");
    println!("xtask/wasm-deploy => Building and optimizing wasm");
    Command::new("cargo")
        .args(["xtask", "wasm-opt"])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-deploy => Copying scripts to website...");
    Command::new("cp")
        .args([
            "target/wasm32-unknown-unknown/release-wasm/opt/rust-snake.d.ts",
            "target/wasm32-unknown-unknown/release-wasm/opt/rust-snake.js",
            "target/wasm32-unknown-unknown/release-wasm/opt/rust-snake_bg.wasm.d.ts",
            "/Users/bmoffett//projects/berintmoffett-com/main/client/static/wasm/rust-snake/",
        ])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-deploy => Copying wasm file to Dropbox...");
    Command::new("cp")
        .args([
            "target/wasm32-unknown-unknown/release-wasm/opt/rust-snake_bg.wasm",
            "/Users/bmoffett/Dropbox/website/website-wasm/rust-snake_bg.wasm",
        ])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-deploy => Automated tasks finished. Push changes on berintmoffett-com to github and restart server to complete");

    Ok(())
}
