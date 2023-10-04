use std::env;

use serde::Deserialize;
use std::process::Command;

// Token manager trait and implementations
pub trait TokenManager {
    fn get_token(&self) -> String;
    fn get_token_type(&self) -> String;
}

pub struct ReplitIdentityTokenManager;

impl TokenManager for ReplitIdentityTokenManager {
    fn get_token(&self) -> String {
        generate_replit_key()
    }

    fn get_token_type(&self) -> String {
        "Bearer".to_string()
    }
}

pub struct L402TokenManager;

impl TokenManager for L402TokenManager {
    fn get_token(&self) -> String {
        env::var("L402_TOKEN").unwrap() // Dummy implementation
    }

    fn get_token_type(&self) -> String {
        "L402".to_string()
    }
}

#[derive(Deserialize)]
pub struct ReplitTokenManagerResponse {
    pub token: String,
    pub timeout: i64,
}

pub fn generate_replit_key() -> String {
    println!("Replit Dynamic API Key ...");
    let repl_slug = env::var("REPL_SLUG").expect("REPL_SLUG not set");
    let script_path = format!("/home/runner/{}/replit/get_token.py", repl_slug);

    let proc = Command::new("python")
        .arg(script_path)
        .output()
        .expect("Failed to execute Get Replit API KEY process");
    let proc_stdout = String::from_utf8_lossy(&proc.stdout);

    if proc_stdout.is_empty() {
        return "".to_string();
    }

    let proc_stdout = proc_stdout.trim();

    // Parse the output into the ReplitTokenManagerResponse struct
    let res: ReplitTokenManagerResponse =
        serde_json::from_str(&proc_stdout).expect("Failed to parse JSON");

    println!("Generated Key!");

    res.token
}
