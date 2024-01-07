use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::io::stdin;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
struct Config {
    user: String,
    arch_packages: Vec<String>,
    eww_repo: String,
    swww_repo: String,
    dotfiles_repo: String,
    font_url: String,
}

#[derive(Debug, Parser)]
struct Arguments {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    TEST,
    INSTALL,
    UNINSTALL,
}

fn main() {
    match Arguments::parse().action {
        Action::TEST => {
            println!("installing...");
            test();
        }
        Action::INSTALL => {
            println!("installing...");
            install_software()
        }
        Action::UNINSTALL => println!("uninstalling..."),
    }
}

fn test() {
    let config: Config = toml::from_str(include_str!("config.toml")).unwrap();
    let home_path: String = "/home/".to_string() + &config.user;
    let root_password = String::new();
    let github_ssh_password: &str = "";
}

fn install_software() {
    let config: Config = toml::from_str(include_str!("config.toml")).unwrap();
    let home_path: String = "/home/".to_string() + &config.user;
    let github_ssh_password: String = read_password("github ssh key");
    let root_password: String = read_password("root");
    let mut git = Command::new("git");

    let font_name = config
        .font_url
        .split("/")
        .last()
        .unwrap()
        .replace(".zip", "");
    println!("Installing font: {}.", font_name);
    Command::new("touch")
        .args(["-p", &(home_path.clone() + "/.local/share/fonts")])
        .spawn()
        .unwrap();
    Command::new("wget")
        .current_dir(home_path.clone() + "/.local/share/fonts")
        .arg(config.font_url.as_str())
        .spawn()
        .unwrap();
    Command::new("unzip")
        .current_dir(home_path.clone() + "/.local/share/fonts")
        .args(["-d", font_name.as_str()])
        .spawn()
        .unwrap();
    Command::new("rm")
        .current_dir(home_path.clone() + "/.local/share/fonts" + font_name.as_str())
        .args([
            "LICENSE README.md",
            &(String::from("../") + font_name.as_str() + ".zip"),
        ])
        .spawn()
        .unwrap();

    println!("Installing Arch packages.");
    Command::new("pacman")
        .args(["-S", config.arch_packages.join(" ").as_str()])
        .spawn()
        .unwrap();

    //create apps directory
    Command::new("mkdir")
        .args(["-p", &(home_path.clone() + "/apps")])
        .spawn()
        .unwrap();

    println!("Building eww binary.");
    git.current_dir(home_path.clone() + "/apps")
        .args(["clone", config.eww_repo.as_str()])
        .spawn()
        .unwrap();
    Command::new("cargo")
        .current_dir(home_path.clone() + "/apps/eww")
        .args([
            "build",
            "--release",
            "--no-default-features",
            "--features=wayland",
        ])
        .spawn()
        .unwrap();
    Command::new("chmod")
        .current_dir(home_path.clone() + "/apps/eww/target/release")
        .args(["+x", "eww"])
        .spawn()
        .unwrap();
    println!("Installing eww binary.");
    Command::new("sudo ")
        .current_dir(home_path.clone() + "/apps/eww/target/release")
        .stdin(Stdio::from(
            Command::new("echo")
                .arg(root_password.clone())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap()
                .stdout
                .unwrap(),
        ))
        .args(["-S", "mv", "eww", "/usr/local/bin/"])
        .spawn()
        .unwrap();

    println!("Building swww and swww-daemon binaries.");
    git.current_dir(home_path.clone() + "/apps")
        .args(["clone", config.swww_repo.as_str()])
        .spawn()
        .unwrap();
    Command::new("cargo")
        .current_dir(home_path.clone() + "/apps/swww")
        .args([
            "build",
            "--release",
            "--no-default-features",
            "--features=wayland",
        ])
        .spawn()
        .unwrap();
    Command::new("chmod")
        .current_dir(home_path.clone() + "/apps/swww/target/release")
        .args(["+x", "swww swww-daemon"])
        .spawn()
        .unwrap();
    println!("Installing swww and swww-daemon binaries.");
    Command::new("sudo")
        .stdin(Stdio::from(
            Command::new("echo")
                .arg(root_password)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap()
                .stdout
                .unwrap(),
        ))
        .args(["-S", "mv", "swww swww-daemon", "/usr/local/bin/"])
        .current_dir(home_path.clone() + "/apps/swww/target/release")
        .spawn()
        .unwrap();

    println!("Applying personal settings.");
    git.current_dir(home_path.clone() + "/.config")
        .args([
            "clone",
            config
                .dotfiles_repo
                .replace("{password}", github_ssh_password.as_str())
                .as_str(),
        ])
        .spawn()
        .unwrap();
    let mut stow = Command::new("stow");
    String::from_utf8(
        Command::new("ls")
            .current_dir(home_path.clone() + "/.config/.dotfiles")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .split_terminator("\n")
    .collect::<Vec<&str>>()
    .into_iter()
    .filter(|directory_item| !directory_item.contains(".md"))
    .inspect(|directory| println!("Creating symlink for {}.", directory))
    .for_each(|directory| {
        stow.current_dir(home_path.clone() + "/.config/.dotfiles")
            .arg(directory)
            .spawn()
            .unwrap();
    })
}

fn read_password(account: &str) -> String {
    println!("Enter password for {}: ", account);
    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(_) => panic!("Undefined behavior. Quiting script."),
    }
}
