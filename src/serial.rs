pub fn list_available_ports() {
    let ports = serialport::available_ports().expect("No ports found!");
    println!("Available ports:");
    for p in ports {
        println!("- {}", p.port_name);
    }
}

pub fn is_port_available(port: String) -> bool {
    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports {
        if p.port_name == port {
            return true;
        }
    }
    false
}
