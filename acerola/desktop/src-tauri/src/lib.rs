#![allow(dead_code)]

mod bios;
mod cmd;
mod core;
mod data;
mod infra;

#[cfg(test)]
pub mod tests;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|panic_info| {
        let panic_message = format!("Panic occurred: {}", panic_info);
        eprintln!("{panic_message}");
        let _ = std::fs::write("PANIC_LOG.txt", panic_message);
    }));

    use pdfium_render::prelude::Pdfium;

    let pdfium_executable_path = std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(|parent| parent.to_path_buf()));

    let pdfium_bindings = if let Some(ref executable_directory) = pdfium_executable_path {
        let resource_directory = executable_directory.join("_up_").join(".bin");
        if resource_directory.exists() {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                &resource_directory,
            ))
        } else {
            let local_bin_directory = executable_directory.join(".bin");
            if local_bin_directory.exists() {
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                    &local_bin_directory,
                ))
            } else {
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                    executable_directory,
                ))
            }
        }
    } else {
        Pdfium::bind_to_system_library()
    };

    let pdfium_instance = Pdfium::new(
        pdfium_bindings
            .or_else(|_| Pdfium::bind_to_system_library())
            .expect("Failed to bind to Pdfium library"),
    );
    std::mem::forget(pdfium_instance);

    let application_context = tauri::generate_context!();
    bios::build().run(application_context).expect("Erro ao executar a aplicação Tauri");
}

pub mod system_cmd {
    #[tauri::command]
    pub fn get_package_family_name() -> String {
        #[cfg(target_os = "windows")]
        {
            use windows::ApplicationModel::Package;
            match Package::Current() {
                Ok(package) => match package.Id() {
                    Ok(package_id) => match package_id.FamilyName() {
                        Ok(family_name) => family_name.to_string(),
                        Err(_) => "Error retrieving Family Name".to_string(),
                    },
                    Err(_) => "Error retrieving Package ID".to_string(),
                },
                Err(_) => "No package identity".to_string(),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            "Not running on Windows".to_string()
        }
    }

    #[tauri::command]
    pub fn open_filesystem_access_settings() {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "ms-settings:privacy-broadfilesystemaccess"])
                .spawn()
                .ok();
        }
    }
}
