use rusb::{
    Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext,
};
use std::{collections::HashMap, time::Duration};
use log::trace;

const LFOS_NAME: &str = env!("CARGO_PKG_NAME");
const LFOS_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

fn find_writable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (interface_number, interface) in config_desc.interfaces().enumerate() {
            for interface_desc in interface.descriptors() {
                for (endpoint_number, endpoint_desc) in
                    interface_desc.endpoint_descriptors().enumerate()
                {
                    if endpoint_desc.direction() == Direction::Out
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        trace!(
                            "Found writable endpoint {}:{} at address {} for device {}",
                            interface_number,
                            endpoint_number,
                            endpoint_desc.address(),
                            device.address()
                        );
                        return Some(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        });
                    }
                }
            }
        }
    }

    None
}

fn write_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: Endpoint,
    transfer_type: TransferType,
    data: &[u8],
) {
    trace!("Writing to endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).ok();
            true
        }
        _ => false,
    };

    trace!(" - kernel driver? {}", has_kernel_driver);

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let timeout = Duration::from_secs(1);
            trace!("Handle state {:?}", handle);

            match transfer_type {
                TransferType::Interrupt => {
                    match handle.write_interrupt(endpoint.address, data, timeout) {
                        Ok(len) => {
                            trace!(" - wrote: {} bytes", len);
                        }
                        Err(err) => {
                            println!("could not write to endpoint: {}", err);
                        }
                    }
                }
                TransferType::Bulk => match handle.write_bulk(endpoint.address, data, timeout) {
                    Ok(len) => {
                        trace!(" - wrote {:?} bytes", len);
                    }
                    Err(err) => println!("could not write to endpoint: {}", err),
                },
                _ => (),
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).ok();
    }
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    trace!(
        "Configuring for sending, and claiming the interface. {:?}",
        endpoint
    );
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}

fn decode_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

const HEADER0: &str = "04000200fcea00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const HEADER1: &str = "05003c00";
const HEADER2: &str = "05013c00";
const HEADER3: &str = "05021800";
const HEADER4: &str = "06003c00";
const HEADER5: &str = "06013c00";
const HEADER6: &str = "06021800";
const HEADER7: &str = "07003c00";
const HEADER8: &str = "07013c00";
const HEADER9: &str = "07021800";
const BODY0: &str = "ffffffffffffffffffffffffffff00ffffffffff00ffff00ffffffffff00ffffffffffffffffffffffffffffff0000ffffffffffff00ffff00ffff00";
const BODY1: &str = "ffff0000ffffffffffffffff00ffff00ffff0000ffffffffff00ffffffffff00ffff0000ffffffffff00ffffff00ff00ffff0000ffffffffffffffff";
const BODY2: &str = "ffffff00ffff0000ffffffffffffffffffff0000ffff0000000000000000000000000000000000000000000000000000000000000000000000000000";

struct Line {
    header: &'static str,
    body: &'static str,
    ofset: u8,
}

fn add_group(groups: &mut HashMap<String, Vec<String>>, name: &str, values: Vec<&str>) {
    groups.insert(
        name.to_string(),
        values.iter().map(|e| e.to_string()).collect(),
    );
}

fn get_key_groups() -> HashMap<String, Vec<String>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    add_group(&mut groups, "pkeys", vec!["p1", "p2", "p3", "p4", "p5"]);
    add_group(&mut groups, "fkeys", vec!["f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12"]);
    add_group(&mut groups, "media", vec!["play", "stop", "playlast", "playnext"]);
    add_group(&mut groups, "system", vec!["prtscrn", "sclock", "pause", "insert", "home", "insert", "pgup", "delete", "end", "pgdown"]);
    add_group(&mut groups, "arrows", vec!["leftarrow", "rightarrow", "uparrow", "downarrow"]);
    add_group(&mut groups, "numpad", vec!["numlock", "numpad/", "numpad*", "numpad-", "numpad7", "numpad8", "numpad9", "numpad+", "numpad4", "numpad5", "numpad6", "numpad1", "numpad2", "numpad3", "numpad0", "numpad.", "numpadenter"]);

    return groups;
}

fn get_keys() -> Vec<&'static str> {
    return vec![
        "esc",
        "\\",
        "tab",
        "capslock",
        "lshift",
        "lcontrol",
        "f12",
        "«",
        "f9",
        "9",
        "o",
        "l",
        ",",
        "<",
        "????",
        "leftarrow",
        "f1",
        "1",
        "q",
        "a",
        "????",
        "windows",
        "prtscrn",
        "????",
        "f10",
        "0",
        "p",
        "ç",
        ".",
        "????",
        "enter",
        "downarrow",
        "f2",
        "2",
        "w",
        "s",
        "z",
        "lalt",
        "sclock",
        "del",
        "f11",
        "'",
        "+",
        "º",
        "-",
        "????",
        "????",
        "rightarrow",
        "f3",
        "3",
        "e",
        "d",
        "x",
        "????",
        "pause",
        "delete",
        "????",
        "numpad7",
        "p1",
        "????",
        "numlock",
        "numpad6",
        "????",
        "????",
        "f4",
        "4",
        "r",
        "f",
        "c",
        "????",
        "insert",
        "end",
        "????",
        "numpad8",
        "p2",
        "????",
        "numpad/",
        "numpad1",
        "????",
        "????",
        "f5",
        "5",
        "t",
        "g",
        "v",
        "????",
        "home",
        "pgdown",
        "stop",
        "numpad9",
        "p3",
        "????",
        "numpad*",
        "numpad2",
        "????",
        "????",
        "f6",
        "6",
        "y",
        "h",
        "b",
        "????",
        "pgup",
        "rshift",
        "playlast",
        "????",
        "p4",
        "????",
        "numpad-",
        "numpad3",
        "????",
        "????",
        "f7",
        "7",
        "u",
        "j",
        "n",
        "altgr",
        "´",
        "rctrl",
        "play",
        "numpad4",
        "p5",
        "????",
        "numpad+",
        "numpad0",
        "????",
        "????",
        "f8",
        "8",
        "i",
        "k",
        "m",
        "fn",
        "~",
        "uparrow",
        "playnext",
        "numpad5",
        "????",
        "????",
        "numpadenter",
        "numpad.",
    ];
}

fn color_component(color: u32, ofset: u8) -> u8 {
    (color >> ofset & 0xff) as u8
}

#[allow(dead_code)]
fn get_color(keys: &Vec<&str>, i: usize, ofset: u8) -> u8 {
    if i < keys.len() {
        return color_component(0xff0000, ofset);
    } else if i == keys.len() {
        return color_component(0xffffff, ofset);
    }

    return color_component(0x000000, ofset);
}

struct LFOS {
    groups: HashMap<String, Vec<String>>,
    keys: Vec<&'static str>
}

fn get_lfos() -> LFOS {
    let keys = get_keys();
    let groups = get_key_groups();
    return LFOS {
        keys,
        groups
    };
}

fn show_usage(lfos: &LFOS) {
    println!("Usage: {0} [key|group] [color] ...\nexample: {0} pkeys ff0000 home 00ff00", LFOS_NAME);

    println!("Groups:\n\tall: all keys");
    for (key, value) in &lfos.groups {
        println!("\t{}: {}", key, value.join(", "));
    }

    let mut sorted_keys = lfos.keys.clone();
    sorted_keys.sort();
    println!("Keys:");
    for key in sorted_keys {
        if key != "????" {
            println!("\t{}", key);
        }
    }

    std::process::exit(0);
}

fn show_version() {
    println!("{} {}", LFOS_NAME, LFOS_VERSION);
    std::process::exit(0);
}

fn try_parse_cmd(
    lfos: &LFOS,
    args: &Vec<String>
) -> std::result::Result<HashMap<String, u32>, Box<dyn std::error::Error>> {
    let mut overrides = HashMap::<String, u32>::new();

    for arg in args {
        if arg == "--help" {
            show_usage(lfos);
        }
        if arg == "--version" {
            show_version();
        }
    }

    if args.len() % 2 != 1 {
        return Err(
            format!("Each key/group must be given a color, like so:\n\t{} key1 color1 key2 color2...", LFOS_NAME)
            .into()
        );
    }
    for i in (1..args.len()).step_by(2) {
        let key = &args[i];
        let value = u32::from_str_radix(args[i + 1].as_str(), 16)?;
        if lfos.groups.contains_key(key) {
            match lfos.groups.get(key) {
                Some(values) => {
                    for val in values {
                        overrides.insert(val.clone(), value);
                    }
                }
                None => (),
            }
        } else {
            overrides.insert(key.clone(), value);
        }
    }

    return Ok(overrides);
}

fn build_table(lfos: LFOS, overrides: HashMap<String, u32>) -> Vec<Vec<u8>> {
    let lines = vec![
        Line {
            header: HEADER1,
            body: BODY0,
            ofset: 16,
        },
        Line {
            header: HEADER2,
            body: BODY1,
            ofset: 16,
        },
        Line {
            header: HEADER3,
            body: BODY2,
            ofset: 16,
        },
        Line {
            header: HEADER4,
            body: BODY0,
            ofset: 8,
        },
        Line {
            header: HEADER5,
            body: BODY1,
            ofset: 8,
        },
        Line {
            header: HEADER6,
            body: BODY2,
            ofset: 8,
        },
        Line {
            header: HEADER7,
            body: BODY0,
            ofset: 0,
        },
        Line {
            header: HEADER8,
            body: BODY1,
            ofset: 0,
        },
        Line {
            header: HEADER9,
            body: BODY2,
            ofset: 0,
        },
    ];    
    let mut result = Vec::<Vec<u8>>::new();
    result.push(Vec::from(decode_hex(HEADER0)));

    for l in 0..lines.len() {
        let entry = &lines[l];
        let mut line = decode_hex(entry.header);
        for i in (0..entry.body.len()).step_by(2) {
            if entry.body.as_bytes()[i] == b'0' {
                line.push(0);
            } else {
                let j = (l % 3) * 60 + i / 2;
                let color = match overrides.get(lfos.keys[j]) {
                    Some(value) => value,
                    None => overrides.get("all").unwrap_or(&0xffffff),
                };
                line.push(color_component(*color, entry.ofset));
            }
        }

        result.push(line);
    }

    return result;
}

fn main() {
    let lfos = get_lfos();
    let args: Vec<String> = std::env::args().collect();
    match try_parse_cmd(&lfos, &args) {
        Ok(overrides) => {
            let table = build_table(lfos, overrides);
            let mut context = rusb::Context::new().unwrap();
            match open_device(&mut context, 0x03f0, 0x1f41) {
                Some((mut device, device_desc, mut handle)) => {
                    for line in table {
                        let ep = find_writable_endpoint(&mut device, &device_desc, TransferType::Interrupt)
                            .unwrap();
                        write_endpoint(&mut handle, ep, TransferType::Interrupt, &line);
                    }
                }
                None => (),
            };
        },
        Err(error) => {
            println!("{}", error.to_string());
        }
    }
}
