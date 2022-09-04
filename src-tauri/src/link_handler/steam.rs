use sysinfo::{System, SystemExt};

pub(crate) fn process_is_running(name: &str) -> bool{
    let s = System::new_all();
    for _process in s.processes_by_exact_name(name) {
        return true
    }
    return false;
}

// open a link using the steam browser protocol: steam://openurl/<url>
// see: https://developer.valvesoftware.com/wiki/Steam_browser_protocol
pub fn handle_steam(url: &url::Url) -> bool{
    if process_is_running("steam") {
        let mut uri = String::from("steam://openurl/");
        uri.push_str(url.as_str());
        webbrowser::open(&uri).unwrap();
        return true;
    }
    return false;
}