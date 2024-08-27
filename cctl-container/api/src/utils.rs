use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::process::{Command, Output};
use regex::Regex;
use std::str;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Debug)]
pub struct CommandResult {
    pub status: String,
    pub stdout: String,
    pub stderr: String,
}

pub fn process_output(output: Output) -> Result<CommandResult, String> {
    if output.status.success() {
        Ok(CommandResult {
            status: "success".to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    } else {
        Err(format!(
            "Command failed with status: {}\nStdout: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn run_command(command: &str, args: Option<Vec<String>>) -> Result<CommandResult, String> {
    let json_file_path = "commands.json";
    let file_contents = fs::read_to_string(json_file_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;
    let command_map: HashMap<String, String> = serde_json::from_str(&file_contents)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    let mut final_command = command_map
        .get(command)
        .ok_or_else(|| format!("Command not found: {}", command))?
        .clone();

    if let Some(ref args_vec) = args {
        final_command = format!("{} {}", final_command, args_vec.join(" "));
    }

    println!("Running command: {}", final_command);

    let output = Command::new("bash")
        .arg("-c")
        .arg(&final_command)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    process_output(output)
}

pub fn parse_node_ports() -> HashMap<String, HashMap<String, i32>> {
    let sidecar_output = run_command("cctl-infra-sidecar-view-ports", None).unwrap();
    let node_output = run_command("cctl-infra-node-view-ports", None).unwrap();

    let sidecar_name_regex = Regex::new(r"CCTL :: SIDECAR-(\d+)").unwrap();
    let sidecar_service_port_regex = Regex::new(r"CCTL ::\s+(MAIN-RPC)\s+[-]+> *(\d+)").unwrap();

    let node_name_regex = Regex::new(r"CCTL :: NODE-(\d+)").unwrap();
    let node_sse_port_regex = Regex::new(r"CCTL ::\s+(SSE)\s+[-]+> *(\d+)").unwrap();

    let mut node_service_ports = HashMap::new();

    // Parse the SIDECAR ports output
    parse_output(&sidecar_output.stdout, &sidecar_name_regex, &sidecar_service_port_regex, &mut node_service_ports, "node-");

    // Parse the NODE ports output
    parse_output(&node_output.stdout, &node_name_regex, &node_sse_port_regex, &mut node_service_ports, "node-");

    node_service_ports
}

fn parse_output(output: &str, node_regex: &Regex, service_regex: &Regex, node_service_ports: &mut HashMap<String, HashMap<String, i32>>, node_prefix: &str) {
    let mut current_node_name = String::new();

    for line in output.lines() {
        if let Some(caps) = node_regex.captures(line) {
            current_node_name = format!("{}{}", node_prefix, &caps[1]);
            node_service_ports.entry(current_node_name.clone()).or_insert_with(HashMap::new);
        } else if let Some(caps) = service_regex.captures(line) {
            if let Some(current_services) = node_service_ports.get_mut(&current_node_name) {
                let service_name = caps[1].to_string();
                let port = caps[2].parse::<i32>().unwrap();
                current_services.insert(service_name, port);
            }
        }
    }
}

pub fn generate_nginx_config(node_service_ports: &HashMap<String, HashMap<String, i32>>) {
    let port = match std::env::var("PROXY_PORT") {
        Ok(port) => port,
        Err(_) => "80".to_string(),
    };
    
    let mut config = format!(
        "events {{
    worker_connections 1024;
}}
http {{
    server {{
        listen {};
        server_name localhost;

", port);

    for (node_name, services) in node_service_ports {
        for (service_name_unready, port) in services {
            let service_name = service_name_unready.to_lowercase();

            let location_block = if service_name == "main-rpc" {
                format!(" location /{}/", node_name)
                // localhost/node-1/ -> localhost:21101/
            } else {
                format!(" location /{}/{}/", node_name, service_name)
            };
            println!("{}", location_block);

            let proxy_pass = format!("proxy_pass http://localhost:{}/;", port);

            let full_location_block = format!("{}{{
                {}
                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                proxy_set_header X-Forwarded-Proto $scheme;
            }}
            ", location_block, proxy_pass);

            config.push_str(&full_location_block);
        }
    }

    config.push_str(" }\n}");

    let mut file = File::create("/etc/nginx/nginx.conf").unwrap();
    file.write_all(config.as_bytes()).unwrap();
}

pub fn start_nginx() {
    let start_output = Command::new("service")
            .arg("nginx")
            .arg("start")
            .output()
            .expect("Failed to start Nginx");

    if !start_output.status.success() {
        eprintln!("Failed to start Nginx: {}", String::from_utf8_lossy(&start_output.stderr));
    }
}

pub fn count_running_nodes() -> i32 {
    let command_output = run_command("cctl-infra-net-status", None).unwrap();
    let stdout = command_output.stdout;
    let mut running_nodes = 0;
    
    for line in stdout.lines() {
        if line.contains("RUNNING") {
            running_nodes += 1;
        }
    }

    running_nodes
}
