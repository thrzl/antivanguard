extern crate windows_service;
use is_elevated::is_elevated;
use std::ffi::OsString;
use std::{sync::mpsc, thread, time};
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};
use tray_item::{IconSource, TrayItem};
use winapi::um::wincon::FreeConsole;
use windows_service::{
    service::ServiceAccess,
    service_manager::{ServiceManager, ServiceManagerAccess},
    Result,
};

const VALORANT_PROCCESS_NAME: &str = "VALORANT-Win64-Shipping.exe";
const VGC_SERVICE_NAME: &str = "vgc";

enum Message {
    Quit,
}

fn detach_console() {
    unsafe {
        FreeConsole();
    }
}

fn main() {
    // Check if running as administrator
    if !is_elevated() {
        println!("please run the program as administrator.");
        return;
    }

    detach_console();
    loop_vgc_check();
}

fn loop_vgc_check() {
    let mut tray = TrayItem::new("antivanguard", IconSource::Resource("aa-exe-icon")).unwrap();

    tray.add_label("options").unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    tray.inner_mut().add_separator().unwrap();

    let quit_tx = tx.clone();
    tray.add_menu_item("quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new().with_exe(UpdateKind::Always)),
    );
    let service_manager =
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT).unwrap();

    let mut launcher_was_running = is_process_running(VALORANT_PROCCESS_NAME, &system);
    loop {
        match rx.try_recv() {
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }
            _ => {}
        }
        // Check if Riot Launcher is running
        system.refresh_processes();
        let riot_launcher_running = is_process_running(VALORANT_PROCCESS_NAME, &system);
        if riot_launcher_running != launcher_was_running {
            if riot_launcher_running {
                // Start the VGC service if Riot Launcher is running
                let _ = start_vgc_service(&service_manager);
            } else {
                // Stop the VGC service if Riot Launcher is not running
                let _ = stop_vgc_service(&service_manager);
            };
            launcher_was_running = riot_launcher_running;
        }

        // Wait for 1 second before next iteration
        thread::sleep(time::Duration::from_millis(1000));
    }
}

fn is_process_running(process_name: &str, s: &System) -> bool {
    let proc_name = OsString::from(process_name.to_ascii_lowercase()); // so it doesn't have to process it every iteration
    s.processes().iter().any(|(_, process)| {
        process
            .exe()
            .and_then(|path| path.file_name().map(|name| name.to_ascii_lowercase()))
            .map(|name| name == proc_name)
            .unwrap_or(false)
    })
}

fn start_vgc_service(manager: &ServiceManager) -> Result<()> {
    let service = manager.open_service(
        VGC_SERVICE_NAME,
        ServiceAccess::START | ServiceAccess::QUERY_STATUS,
    )?;
    let status = service.query_status().unwrap();
    if status.current_state == windows_service::service::ServiceState::Running {
        return Ok(());
    };
    service.start(&[""])?;
    println!("started vgc service.");
    Ok(())
}

fn stop_vgc_service(manager: &ServiceManager) -> Result<()> {
    let service = manager.open_service(
        VGC_SERVICE_NAME,
        ServiceAccess::STOP | ServiceAccess::QUERY_STATUS,
    )?;
    let status = service.query_status().unwrap();
    if status.current_state == windows_service::service::ServiceState::Stopped {
        return Ok(());
    };
    service.stop()?;
    println!("stopped vgc service.");
    Ok(())
}
