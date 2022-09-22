pub mod steam;

type Action = fn(&url::Url) -> bool;

static FUNCTIONS: &'static [(&'static str, Action)] = &[
    ("steamcommunity.com", steam::handle_steam),
    ("store.steampowered.com", steam::handle_steam)
];

fn handle_default(url: &url::Url) -> bool{
    let result = open::that(url.as_str());
    match result {
        Ok(_) => true,
        Err(_) => false
    }
}

fn find_action(prog: &str) -> Option<Action> {
    match FUNCTIONS.binary_search_by(|&(name,_)| name.cmp(prog)) {
        Ok(idx) => Some(FUNCTIONS[idx].1),
        Err(_) => None,
    }
}

fn invoke(url: &url::Url, bypass_handlers: bool) -> bool {

    if bypass_handlers { return handle_default(url); }

    let mut search_string: Option<&str>;

    search_string = url.host_str();

    search_string = match search_string {
        Some(str) => Some(str),
        None => Some(url.scheme())
    };

    let action: Action = match search_string {
        Some(host_string) => {
            match find_action(host_string) {
                Some(found_action) => found_action,
                None => handle_default
            }
        },
        None => handle_default
    };

    let mut result = action(url);

    // If the custom link handler returned false, it failed and we should open in default
    if !result {
        result = handle_default(url)
    }

    return result;
}

#[tauri::command]
pub(crate) fn open_link(url: String, bypass_handlers: bool) -> bool{
    println!("Opening link: {}", url);

    match url::Url::parse(&url) {
        Ok(url) => invoke(&url, bypass_handlers),
        Err(_) => false
    }
}