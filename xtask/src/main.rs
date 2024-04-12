use anyhow::Context;
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
    WasmDeploy,
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
    let status = Command::new("git").args(["status", "--porcelain"]).output()?;
    if !status.stdout.is_empty() {
        eprintln!("xtask/wasm-deploy => Working tree is not empty!");
        std::process::exit(1);
    }
    let tag = format!("v{}", xtask::get_cargo_version()?);
    println!("xtask/wasm-deploy => Building and optimizing wasm");
    Command::new("cargo")
        .args(["xtask", "wasm-opt"])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-deploy => Creating tarball...");
    Command::new("tar")
        .args([
            "-C",
            "target/wasm32-unknown-unknown/release-wasm/opt",
            "-czvf", 
            format!("{}.tar.gz", &tag).as_str(),
            "rust-snake.d.ts",
            "rust-snake.js",
            "rust-snake_bg.wasm.d.ts",
            "rust-snake_bg.wasm",
        ])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-deploy => Creating git tag...");
    Command::new("git").args(["tag", &tag]).spawn()?.wait()?;

    println!("xtask/wasm-deploy => Push tag to github...");
    Command::new("git").args(["push", "--tags"]).spawn()?.wait()?;

    println!("xtask/wasm-deploy => Creating Github release...");
    Command::new("gh")
        .args([
            "release",
            "create",
            &tag,
            format!("./{}.tar.gz", &tag).as_str(),
            "--generate-notes",
            "--verify-tag",
        ])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-deploy => Cleaning up...");
    Command::new("rm")
        .args([format!("./{}.tar.gz", &tag).as_str()])
        .spawn()?
        .wait()?;

    println!("xtask/wasm-deploy => Copying wasm to Dropbox...");
    let home_dir = dirs::home_dir().context("couldn't get path to home dir")?;
    Command::new("cp").args([
        "target/wasm32-unknown-unknown/release-wasm/opt/rust-snake_bg.wasm",
        home_dir
            .join("Dropbox/website/website-wasm/rust-snake_bg.wasm")
            .to_str()
            .context("couldn't convert dropbox wasm path to string")?,
    ]);

    println!("xtask/wasm-deploy => Done!");
    Ok(())
}
