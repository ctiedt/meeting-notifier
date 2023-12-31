use clap::Parser;
use windows::Win32::{
    Foundation::{CloseHandle, HMODULE},
    System::ProcessStatus::GetModuleBaseNameA,
};

struct ProcInfo {
    pid: u32,
    name: String,
}

#[derive(Parser)]
struct Args {
    pattern: String,
    cmd: String,
    closing_cmd: Option<String>,
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();
    let mut processes = [0u32; 8192];
    let mut cb_needed = 0;

    let program_args = args
        .cmd
        .split_whitespace()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    let mut closing_program_args = None;

    if let Some(closing_cmd) = args.closing_cmd {
        closing_program_args.replace(
            closing_cmd
                .split_whitespace()
                .map(|it| it.to_string())
                .collect::<Vec<_>>(),
        );
    }

    let mut target_detected = false;

    loop {
        unsafe {
            windows::Win32::System::ProcessStatus::EnumProcesses(
                processes.as_mut_ptr(),
                processes.len() as u32,
                &mut cb_needed,
            )
        };

        let c_processes = cb_needed as usize / std::mem::size_of::<u32>();

        let proc_infos = processes
            .iter()
            .take(c_processes)
            .filter_map(|pid| get_process_info(*pid).ok())
            .collect::<Vec<_>>();

        if proc_infos
            .iter()
            .any(|proc| proc.name.to_ascii_lowercase().contains(&args.pattern))
        {
            if !target_detected {
                std::process::Command::new(&program_args[0])
                    .args(&program_args[1..])
                    .spawn()?
                    .wait()?;
                target_detected = true;
            }
        } else {
            if target_detected {
                if let Some(closing_program_args) = &closing_program_args {
                    std::process::Command::new(&closing_program_args[0])
                        .args(&closing_program_args[1..])
                        .spawn()?
                        .wait()?;
                }
            }
            target_detected = false;
        }
    }
}

fn get_process_info(pid: u32) -> color_eyre::Result<ProcInfo> {
    let mut sz_processname = [0u8; 1024];

    let handle = unsafe {
        windows::Win32::System::Threading::OpenProcess(
            windows::Win32::System::Threading::PROCESS_QUERY_INFORMATION
                | windows::Win32::System::Threading::PROCESS_VM_READ,
            false,
            pid,
        )
    }?;
    let mut hmod = windows::Win32::Foundation::HMODULE(0);
    let mut cb_needed = 0;

    if unsafe {
        windows::Win32::System::ProcessStatus::EnumProcessModules(
            handle,
            &mut hmod,
            std::mem::size_of::<HMODULE>() as u32,
            &mut cb_needed,
        )
        .into()
    } {
        unsafe { GetModuleBaseNameA(handle, hmod, &mut sz_processname) };
    }

    unsafe { CloseHandle(handle) };

    let name = String::from_utf8_lossy(&sz_processname).to_string();
    Ok(ProcInfo { pid, name })
}
