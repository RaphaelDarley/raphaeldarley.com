use std::fs;

use axum::extract::Path;

pub async fn handler_all() -> String {
    let mut acc = Vec::new();

    for kf in fs::read_dir("./content/keys").unwrap() {
        let kf = kf.unwrap();
        if kf.file_type().unwrap().is_file() {
            let key = fs::read_to_string(kf.path()).unwrap();
            acc.push(key);
        }
    }
    acc.join("\n")
}

pub async fn handler(Path(name): Path<String>) -> String {
    fs::read_to_string(format!("./content/keys/{name}")).unwrap()
}
