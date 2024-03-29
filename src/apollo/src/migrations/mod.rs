use std::io::Read;

use apollo_utils::log;
use ic_cdk::{post_upgrade, pre_upgrade};
use ic_stable_structures::{reader::Reader, writer::Writer, Memory};
use ic_utils::{
    logger::{self, LogMessageStorage},
    monitor::{self, store::DayDataTable},
};

use crate::{memory, types::State, utils::set_custom_panic_hook, STATE};

// A pre-upgrade hook for serializing the data stored on the heap.
#[pre_upgrade]
fn pre_upgrade() {
    save_upgrade_data()
}

// A post-upgrade hook for deserializing the data back into the heap.
#[post_upgrade]
async fn post_upgrade() {
    load_upgrade_data();

    set_custom_panic_hook();
    log!("Post upgrade finished");
}

fn save_upgrade_data() {
    let mut memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut memory, 0);
    save_state_data(&mut writer);
    save_logger_data(&mut writer);
    save_monitor_data(&mut writer);
}

fn load_upgrade_data() {
    let memory = memory::get_upgrades_memory();
    let mut reader = Reader::new(&memory, 0);

    read_state_data(&mut reader);
    read_logger_data(&mut reader);
    read_monitor_data(&mut reader);
}

fn save_state_data(writer: &mut Writer<impl Memory>) {
    // Serialize the state.
    let mut state_bytes = vec![];
    STATE
        .with(|s| ciborium::ser::into_writer(&*s.borrow(), &mut state_bytes))
        .expect("failed to encode state");

    // Write the length of the serialized bytes to memory, followed by the
    // by the bytes themselves.
    let len = state_bytes.len() as u32;

    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap();
}

fn read_state_data(reader: &mut Reader<impl Memory>) {
    let mut len_bytes = [0; 4];

    // Deserialize the state data.
    // Read the length of the state bytes.

    reader.read_exact(&mut len_bytes).unwrap();
    let state_len = u32::from_le_bytes(len_bytes) as usize;

    // Read the bytes
    let mut state_bytes = vec![0; state_len];
    reader.read_exact(&mut state_bytes).unwrap();

    // Deserialize
    let state: State = ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");

    STATE.with(|s| s.replace(state));
}

fn save_logger_data(writer: &mut Writer<impl Memory>) {
    // Serialize the logger data.
    let logger_data = logger::pre_upgrade_stable_data();

    let mut logger_bytes = vec![];
    ciborium::ser::into_writer(&logger_data, &mut logger_bytes).expect("failed to encode logger");

    let len = logger_bytes.len() as u32;

    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&logger_bytes).unwrap();
}

fn read_logger_data(reader: &mut Reader<impl Memory>) {
    let mut len_bytes = [0; 4];

    // Deserialize the logger data.
    // Read the length of the state bytes.
    reader.read_exact(&mut len_bytes).unwrap();
    let logger_len = u32::from_le_bytes(len_bytes) as usize;

    // Read the bytes
    let mut logger_bytes = vec![0; logger_len];
    reader.read_exact(&mut logger_bytes).unwrap();

    // Deserialize
    let log_data: (u8, LogMessageStorage) =
        ciborium::de::from_reader(&*logger_bytes).expect("failed to decode logger_data");

    logger::post_upgrade_stable_data(log_data);
}

fn save_monitor_data(writer: &mut Writer<impl Memory>) {
    // Serialize the monitor data.
    let monitor_data = monitor::pre_upgrade_stable_data();

    let mut monitor_bytes = vec![];
    ciborium::ser::into_writer(&monitor_data, &mut monitor_bytes)
        .expect("failed to encode monitor");
    let len = monitor_bytes.len() as u32;

    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&monitor_bytes).unwrap();
}

fn read_monitor_data(reader: &mut Reader<impl Memory>) {
    let mut len_bytes = [0; 4];

    // Deserialize the monitor data.
    // Read the length of the state bytes.
    reader.read_exact(&mut len_bytes).unwrap();
    let monitor_len = u32::from_le_bytes(len_bytes) as usize;

    // Read the bytes
    let mut logger_bytes = vec![0; monitor_len];
    reader.read_exact(&mut logger_bytes).unwrap();

    // Deserialize and set the state.
    let monitor_data: (u8, DayDataTable) =
        ciborium::de::from_reader(&*logger_bytes).expect("failed to decode monitor_data");

    monitor::post_upgrade_stable_data(monitor_data);
}
