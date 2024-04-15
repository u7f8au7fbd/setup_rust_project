use serde_json::{self, json};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;
use std::ptr::null_mut;
use winapi::um::winuser::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

fn main() {
    set_utf8();
    let dir_name = get_directory();
    check_string(&dir_name);
    delete_code_workspace();
    delete_launch_json();
    make_code_workspace(&dir_name);
    make_launch_json(&dir_name);
    make_cargo_toml(&dir_name);
    dialog("プロジェクトの名前を変更しました");
}

fn dialog(massage: &str) {
    let message = OsStr::new(massage)
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let title = OsStr::new("エラー")
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();

    unsafe {
        MessageBoxW(
            null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn check_string(dir_name: &str) {
    if dir_name.is_empty() {
        dialog("ファイル名が空です");
        panic!()
    }

    if dir_name.chars().any(|c| {
        !c.is_ascii()
            || c.is_ascii_control()
            || c.is_whitespace()
            || c == '\\'
            || c == '/'
            || c == ':'
            || c == '*'
            || c == '?'
            || c == '"'
            || c == '<'
            || c == '>'
            || c == '|'
    }) {
        dialog("ファイルパスに使えない文字が含まれています");
    }

    if dir_name.chars().count() > 24 {
        dialog("ファイル名が長すぎます")
    }
}

fn set_utf8() {
    Command::new("cmd")
        .args(["/C", "chcp 65001"])
        .output()
        .expect("UTF-8に設定できませんでした");
}

fn delete_code_workspace() {
    if let Ok(entries) = fs::read_dir("./") {
        for entry in entries {
            if let Some(file_name) = entry.ok().unwrap().file_name().to_str() {
                if file_name.ends_with(".code-workspace") {
                    fs::remove_file(file_name).unwrap();
                }
            }
        }
    }
}

fn make_code_workspace(project_name: &str) {
    let data = json!({
        "folders": [
            {
                "path": "."
            }
        ],
        "settings": {
            "rust-analyzer.linkedProjects": [
                ".\\Cargo.toml"
            ]
        }
    });

    let json_str = serde_json::to_string(&data).unwrap();
    let fire_path = format!("{}.code-workspace", project_name);
    let mut file = File::create(fire_path).unwrap();
    file.write_all(json_str.as_bytes()).unwrap();
}

fn make_launch_json(project_name: &str) {
    let data = json!({
        "version": "0.2.0",
        "configurations": [
            {
                "type": "lldb",
                "request": "launch",
                "name": "Debug",
                "cargo": {
                    "args": [
                        "build",
                        "--bin=${workspaceFolderBasename}",
                        "--package=${workspaceFolderBasename}",
                    ],
                    "filter": {
                        "name": project_name,
                        "kind": "bin"
                    }
                },
                "args": [],
                "cwd": "${workspaceFolder}",
                "env": {
                    "PATH": "${env:USERPROFILE}/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin;${workspaceFolder}/target/debug/deps;${env:PATH}",
                },
            }
        ]
    });

    let json_str = serde_json::to_string(&data).unwrap();
    let mut file = File::create("./.vscode/launch.json").unwrap();
    file.write_all(json_str.as_bytes()).unwrap();
}

fn delete_launch_json() {
    fs::remove_file("./.vscode/launch.json").unwrap();
}

fn make_cargo_toml(project_name: &str) {
    let cargo_toml_path = "./Cargo.toml";
    let cargo_toml_content = fs::read_to_string(cargo_toml_path).unwrap();
    let mut cargo_new_toml_content = String::new();

    for line in cargo_toml_content.lines() {
        if line.starts_with("name") {
            cargo_new_toml_content.push_str(&format!("name = \"{}\"\n", project_name));
        } else {
            cargo_new_toml_content.push_str(line);
            cargo_new_toml_content.push('\n');
        }
    }
    let cargo_new_toml_path = "./Cargo.toml";
    let mut cargo_new_toml_file = File::create(cargo_new_toml_path).unwrap();
    cargo_new_toml_file
        .write_all(cargo_new_toml_content.as_bytes())
        .unwrap();
}

fn get_directory() -> String {
    let current_dir = std::env::current_dir().expect("現在のディレクトリパスの取得に失敗しました");
    let directory = current_dir
        .file_name()
        .expect("ディレクトリ名の取得に失敗しました")
        .to_str()
        .expect("ディレクトリ名を文字列に変換できませんでした")
        .to_owned();
    directory
}
