use std::env;
use std::time::Duration;

//mod urad_industrial;
mod serial;

// Serial Ports Setup
const CONFIG_PORT_BAUD: u32 = 115200;
const DATA_PORT_BAUD: u32 = 921600;
const SYNC_PATTERN: u64 = 0x708050603040102;
const PORT_TIMEOUT: f32 = 0.3;

// Data Setup
const _TLV_HEADER_LEN: i32 = 8;
const _HEADER_LEN: i32 = 40;

fn main() {
    let args: Vec<String> = env::args().collect();
    let _file_path = &args[0];

    let config_port_name = "/dev/ttys000";
    let data_port_name = "/dev/ttys000";

    // if !(serial::is_port_available(config_port_name.to_string())) {
    //     serial::list_available_ports();
    //     panic!("Port '{config_port_name}' not available.");
    // }

    let mut config_port = serialport::new(config_port_name, CONFIG_PORT_BAUD)
        .timeout(Duration::from_secs_f32(PORT_TIMEOUT))
        .open()
        .expect("Failed to open config port.");

    let mut data_port = serialport::new(data_port_name, DATA_PORT_BAUD)
        .timeout(Duration::from_secs_f32(PORT_TIMEOUT))
        .open()
        .expect("Failed to open data port.");

    let mut serial_buf: Vec<u8> = vec![0; 1];
    loop {
        data_port.read(serial_buf.as_mut_slice())
            .expect("Found no data!");
        println!("{:?}", serial_buf[0]);
    }
}
