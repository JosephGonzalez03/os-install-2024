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
    #[clap(subcommand)]
    Install(InstallOption),
    Uninstall,
}

#[derive(Debug, Subcommand)]
enum InstallOption {
    Test,
    ArchPackages,
    Dotfiles,
    Fonts,
}

fn main() {
    let config: Config = toml::from_str(include_str!("config.toml")).unwrap();
    let home_path: String = "/home/".to_string() + &config.user;

    //Add rustup component add rust-analyzer command
    match Arguments::parse().action {
        Action::Install(install_option) => {
            println!("installing...");
            match install_option {
                InstallOption::Test => test_fn(),
                InstallOption::ArchPackages => install_arch_packages(config.arch_packages),
                InstallOption::Dotfiles => install_dotfiles(config.dotfiles_repo, home_path),
                InstallOption::Fonts => install_fonts(config.font_url, home_path),
            }
        }
        Action::Uninstall => println!("uninstalling..."),
    }
}

fn test_fn() {
    let cmd = Command::new("ls").output().unwrap().stdout;
    println!("printing...{:?} end of print", cmd);
    Command::new("ls").spawn().unwrap();
}

fn install_fonts(font_url: String, home_path: String) {
    let font_name = font_url.split("/").last().unwrap().replace(".zip", "");

    println!("Installing font: {}.", font_name);
    Command::new("mkdir")
        .args(["-p", &(home_path.clone() + "/.local/share/fonts")])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    Command::new("wget")
        .current_dir(home_path.clone() + "/.local/share/fonts")
        .arg(font_url.as_str())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    Command::new("unzip")
        .current_dir(home_path.clone() + "/.local/share/fonts")
        .args([
            font_name.clone() + ".zip",
            String::from("-d"),
            font_name.clone(),
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    Command::new("rm")
        .current_dir(home_path.clone() + "/.local/share/fonts/" + font_name.as_str())
        .args([
            "LICENSE.txt",
            "README.md",
            &(String::from("../") + font_name.as_str() + ".zip"),
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn install_arch_packages(packages: Vec<String>) {
    let root_password: String = read_password("root");

    println!("Installing Arch packages.");
    Command::new("sudo")
        .stdin(Stdio::from(
            Command::new("echo")
                .arg(root_password.clone())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap()
                .stdout
                .unwrap(),
        ))
        .args(["-S", "pacman", "-S"])
        .args(packages)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn install_dotfiles(dotfiles_repo: String, home_path: String) {
    let github_ssh_password: String = read_password("github ssh key");
    println!("Applying personal settings.");

    Command::new("git")
        .current_dir(home_path.clone() + "/.config")
        .args([
            "clone",
            dotfiles_repo
                .replace("{password}", github_ssh_password.as_str())
                .as_str(),
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    let dotfiles_contents: String = String::from_utf8(
        Command::new("ls")
            .current_dir(home_path.clone() + "/.config/.dotfiles")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let dotfiles_directories: Vec<&str> = dotfiles_contents
        .split_terminator("\n")
        .filter(|directory_item| !directory_item.contains(".md"))
        .collect();

    Command::new("stow")
        .current_dir(home_path.clone() + "/.config/.dotfiles")
        .args(dotfiles_directories)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn install_software(config: Config, home_path: String) {
    let root_password: String = read_password("root");
    let mut git = Command::new("git");

    //create apps directory
    Command::new("mkdir")
        .args(["-p", &(home_path.clone() + "/apps")])
        .spawn()
        .unwrap();

    //println!("Building eww binary.");
    //git.current_dir(home_path.clone() + "/apps")
    //    .args(["clone", config.eww_repo.as_str()])
    //    .spawn()
    //    .unwrap();
    //Command::new("cargo")
    //    .current_dir(home_path.clone() + "/apps/eww")
    //    .args([
    //        "build",
    //        "--release",
    //        "--no-default-features",
    //        "--features=wayland",
    //    ])
    //    .spawn()
    //    .unwrap();
    //Command::new("chmod")
    //    .current_dir(home_path.clone() + "/apps/eww/target/release")
    //    .args(["+x", "eww"])
    //    .spawn()
    //    .unwrap();
    //println!("Installing eww binary.");
    //Command::new("sudo ")
    //    .current_dir(home_path.clone() + "/apps/eww/target/release")
    //    .stdin(Stdio::from(
    //        Command::new("echo")
    //            .arg(root_password.clone())
    //            .stdout(Stdio::piped())
    //            .spawn()
    //            .unwrap()
    //            .stdout
    //            .unwrap(),
    //    ))
    //    .args(["-S", "mv", "eww", "/usr/local/bin/"])
    //    .spawn()
    //    .unwrap();

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
}

fn read_password(account: &str) -> String {
    println!("Enter password for {}: ", account);
    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(_) => panic!("Undefined behavior. Quiting script."),
    }
}
