
use webbrowser;
use url;

pub mod steam;

type Action = fn(&url::Url) -> bool;

static FUNCTIONS: &'static [(&'static str, Action)] = &[
    ("steamcommunity.com", steam::handle_steam),
    ("store.steampowered.com", steam::handle_steam)
];

fn handle_default(url: &url::Url) -> bool{
    webbrowser::open(url.as_str()).unwrap();
    return true;
}

fn find_action(prog: &String) -> Option<Action> {
    match FUNCTIONS.binary_search_by(|&(name,_)| name.cmp(prog)) {
        Ok(idx) => Some(FUNCTIONS[idx].1),
        Err(_) => None,
    }
}

fn invoke(args: &url::Url, bypass_handlers: bool) -> bool {

    if bypass_handlers { return handle_default(args); }

    let host = String::from(args.host_str().unwrap());

    let mut result = match find_action(&host) {
        Some(action) => action(args),
        None => handle_default(args),
    };

    if !result {
        result = handle_default(args);
    }

    return result;
}

#[tauri::command]
pub(crate) fn open_link(url: String, bypass_handlers: bool){
    println!("Opening link: {}", url);

    let parsed_url = url::Url::parse(&url).unwrap();

    invoke(&parsed_url, bypass_handlers);
}