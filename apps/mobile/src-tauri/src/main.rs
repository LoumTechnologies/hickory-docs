// Desktop entry point; mobile targets enter through `mobile_entry_point` in lib.rs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hickory_mobile_lib::run();
}
