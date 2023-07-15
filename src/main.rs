use chrono;
use serialport::ClearBuffer;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;

use bytemuck::{bytes_of_mut, Pod, Zeroable};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

//
const WRITE_TO_FILE: bool = true;

// Data Setup
const TLV_HEADER_LEN: usize = 8;
const HEADER_LEN: usize = 40;

// Serial Ports Setup
const CONFIG_PORT_BAUD: u32 = 115200;
const DATA_PORT_BAUD: u32 = 921600;
const SYNC_PATTERN: u64 = 0x708050603040102;
const PORT_TIMEOUT: f32 = 0.3;

// Setup Commands
const SETUP_COMMANDS: &'static [&'static str] = &[
    "\n",
    "sensorStop\n",
    "flushCfg\n",
    "dfeDataOutputMode 1\n",
    "channelCfg 15 7 0\n",
    "adcCfg 2 1\n",
    "adcbufCfg -1 0 1 1 1\n",
    "profileCfg 0 60 359 7 57.14 0 0 70 1 256 5209 0 0 158\n",
    "chirpCfg 0 0 0 0 0 0 0 1\n",
    "chirpCfg 1 1 0 0 0 0 0 2\n",
    "chirpCfg 2 2 0 0 0 0 0 4\n",
    "frameCfg 0 2 16 0 100 1 0\n",
    "lowPower 0 0\n",
    "guiMonitor -1 1 0 0 0 0 0\n",
    "cfarCfg -1 0 2 8 4 3 0 15 1\n",
    "cfarCfg -1 1 0 4 2 3 1 15 1\n",
    "multiObjBeamForming -1 1 0.5\n",
    "clutterRemoval -1 0\n",
    "calibDcRangeSig -1 0 -5 8 256\n",
    "extendedMaxVelocity -1 0\n",
    "lvdsStreamCfg -1 0 0 0\n",
    "compRangeBiasAndRxChanPhase 0.0 1 0 -1 0 1 0 -1 0 1 0 -1 0 1 0 -1 0 1 0 -1 0 1 0 -1 0\n",
    "measureRangeBiasAndRxChanPhase 0 1.5 0.2\n",
    "CQRxSatMonitor 0 3 5 121 0\n",
    "CQSigImgMonitor 0 127 4\n",
    "analogMonitor 0 0\n",
    "aoaFovCfg -1 -90 90 -90 90\n",
    "cfarFovCfg -1 0 0 8.92\n",
    "cfarFovCfg -1 1 -1 1.00\n",
    "calibData 0 0 0\n",
    "sensorStart\n",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Obj {
    date: String,
    obj_num: usize,
    x: f32,
    y: f32,
    z: f32,
    v: f32,
    snr: u32,
    noise: u32,
}

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct Data {
    sync: [u8; 8],
    version: [u8; 4],
    total_packet_len: [u8; 4],
    platform: [u8; 4],
    frame_number: [u8; 4],
    time_cpu_cycles: [u8; 4],
    num_detected_obj: [u8; 4],
    num_tlvs: [u8; 4],
    sub_frame_nuber: [u8; 4],
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

pub fn list_available_ports() {
    let ports = serialport::available_ports().expect("No ports found!");
    println!("Available ports:");
    for p in ports {
        println!("- {}", p.port_name);
    }
}

pub fn read_deserialize_data<R: Read>(data_port: &mut R) -> Data {
    let mut data = Data::default();
    let serial_buf = bytes_of_mut(&mut data);
    data_port.read(serial_buf).expect("Found no data!");
    data
}

pub fn write_to_file(data: &str) -> std::io::Result<()> {
    let mut file = File::options().append(true).open("data.json")?;
    writeln!(&mut file, "{}", data)?;
    Ok(())
}

fn main() {
    let _args: Vec<String> = env::args().collect();

    let config_port_name = "COM4";
    let data_port_name = "COM5";

    if !(is_port_available(config_port_name.to_string())) {
        list_available_ports();
        panic!("Port '{config_port_name}' not available.");
    }

    // Open config port
    let mut config_port = serialport::new(config_port_name, CONFIG_PORT_BAUD)
        .timeout(Duration::from_secs_f32(PORT_TIMEOUT))
        .open()
        .expect("Failed to open config port.");

    config_port
        .clear(ClearBuffer::Output)
        .expect("Failed to discard output buffer.");

    // Open data port
    let mut data_port = serialport::new(data_port_name, DATA_PORT_BAUD)
        .timeout(Duration::from_secs_f32(PORT_TIMEOUT))
        .open()
        .expect("Failed to open data port.");
    data_port
        .clear(ClearBuffer::Output)
        .expect("Failed to discard output buffer.");

    let btr = config_port
        .bytes_to_read()
        .expect("Config port read error.");
    if btr > 0 {
        let mut serial_buf: Vec<u8> = vec![0; btr as usize];
        config_port
            .read(serial_buf.as_mut_slice())
            .expect("Found no data!");
        let output = String::from_utf8_lossy(&serial_buf);
        print!("{:?}", output);
    }

    // Send setup commands
    for command in SETUP_COMMANDS.iter() {
        config_port
            .write(command.as_bytes())
            .expect("Write failed!");
        std::thread::sleep(Duration::from_millis(20));

        // Get feedback
        let mut res: Vec<u8> = Vec::new();
        while config_port.bytes_to_read().unwrap() > 0 {
            let mut serial_buf: Vec<u8> = vec![0; 1];
            config_port.read(serial_buf.as_mut_slice()).unwrap();
            res.extend_from_slice(&serial_buf);
        }
        let output = String::from_utf8_lossy(&res);
        println!("{}", output);
    }
    println!("\nSetup done.");

    // Receive data
    loop {
        let data = read_deserialize_data(&mut data_port);
        let sync = u64::from_le_bytes(data.sync);

        if sync == SYNC_PATTERN {
            let num_detected_obj = u32::from_le_bytes(data.num_detected_obj);
            let total_packet_len = u32::from_le_bytes(data.total_packet_len);

            let btr = total_packet_len - HEADER_LEN as u32;
            let mut packet_payload: Vec<u8> = vec![0; btr as usize];

            data_port
                .read(packet_payload.as_mut_slice())
                .expect("Found no data!");

            let num_tlvs = u32::from_le_bytes(data.num_tlvs);

            let mut objects: Vec<Obj> = Vec::new(); // collect all targets in Vec

            for _i in 0..num_tlvs {
                let mut tlv_type_slice = &packet_payload[0..4];
                let tlv_type = tlv_type_slice.read_u32::<LittleEndian>().unwrap();
                let mut tlv_length_slice = &packet_payload[4..8];
                let _tlv_length = tlv_length_slice.read_u32::<LittleEndian>().unwrap();

                packet_payload.drain(0..TLV_HEADER_LEN);

                if tlv_type == 1 {
                    // collect coordinates, velocity

                    for j in 0..num_detected_obj {
                        let mut x = &packet_payload[0..4];
                        let x = x.read_f32::<LittleEndian>().unwrap();

                        let mut y = &packet_payload[4..8];
                        let y = y.read_f32::<LittleEndian>().unwrap();

                        let mut z = &packet_payload[8..12];
                        let z = z.read_f32::<LittleEndian>().unwrap();

                        let mut v = &packet_payload[12..16];
                        let v = v.read_f32::<LittleEndian>().unwrap();

                        let dat = (chrono::offset::Local::now()).to_string();

                        let obj = Obj {
                            date: dat,
                            obj_num: j as usize,
                            x,
                            y,
                            z,
                            v,
                            snr: 0,
                            noise: 0,
                        };
                        objects.push(obj);

                        packet_payload.drain(0..16);
                    }
                } else if tlv_type == 7 {
                    // collect snr, noise

                    for j in 0..num_detected_obj {
                        let snr_pack = &packet_payload[0..2];
                        let snr = (snr_pack[0] as u32) | (snr_pack[1] as u32) << 8;
                        let noise_pack = &packet_payload[2..4];
                        let noise = (noise_pack[0] as u32) | (noise_pack[1] as u32) << 8;

                        objects[j as usize].snr = snr;
                        objects[j as usize].noise = noise;

                        packet_payload.drain(0..4);
                    }
                }
            }

            for i in objects {
                if WRITE_TO_FILE {
                    println!("{}", serde_json::to_string(&i).unwrap());
                    let data = serde_json::to_string(&i).unwrap();
                    let _result = write_to_file(&data).unwrap();
                } else {
                    println!("{}", serde_json::to_string(&i).unwrap());
                }
            }
        }
    }
}
