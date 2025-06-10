use std::process::Command;

pub fn start_frontend() {
    let frontend = Command::new("npm")
        .arg("start")
        .current_dir("frontend")
        .spawn();

    match frontend {
        Ok(_) => println!("✅ Frontend server started on http://localhost:3000"),
        Err(e) => eprintln!("❌ Failed to start frontend server: {}", e),
    }
}