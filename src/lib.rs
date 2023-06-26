use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::fs::File;
use std::fs;
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct ListItem {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub cookie: Option<String>,
    pub list: Option<Vec<ListItem>>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            cookie: None,
            list: None,
        }
    }
}

pub fn write_to_see(key: &str, value: impl Serialize) -> Result<(), Box<dyn Error>> {
    let home_dir = env::var("HOME")?;
    let see_file_path = format!("{}/.see", home_dir);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(see_file_path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Data = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("deserialize error: {}", e);
            Default::default()
        }
    };

    match key {
        "cookie" => {
            let cookie: Option<String> = match serde_json::from_value(serde_json::to_value(value)?) {
                Ok(cookie) => cookie,
                Err(e) => {
                    eprintln!("deserialize error: {}", e);
                    None
                }
            };
            data.cookie = cookie;
        }
        "list" => {
            let list: Option<Vec<ListItem>> = match serde_json::from_value(serde_json::to_value(value)?) {
                Ok(list) => list,
                Err(e) => {
                    eprintln!("deserialize error: {}", e);
                    None
                }
            };
            if let Some(new_list) = list {
                if let Some(existing_list) = &mut data.list {
                    for new_item in new_list {
                        if let Some(existing_item) = existing_list.iter_mut().find(|item| item.name == new_item.name) {
                            existing_item.value = new_item.value.clone();
                        } else {
                            existing_list.push(new_item);
                        }
                    }
                } else {
                    data.list = Some(new_list);
                }
            }
        }
        _ => return Err("Invalid key".into()),
    }

    file.set_len(0)?;
    file.seek(std::io::SeekFrom::Start(0))?;
    serde_json::to_writer(file, &data).unwrap();

    Ok(())
}

pub fn read_from_see() -> Result<Data, Box<dyn Error>> {
    let home_dir = env::var("HOME")?;
    let see_file_path = format!("{}/.see", home_dir);
    println!("see_file_path: {}", see_file_path);
    let mut file = match File::open(see_file_path) {
        Ok(file) => file,
        Err(_) => return Ok(serde_json::from_str("{}")?),
    };
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: Data = serde_json::from_str(&contents)?;

    Ok(data)
}

fn is_dir_not_empty(path: &str) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            // 如果有任何一个文件，就说明目录非空
            if entry.is_ok() {
                return true;
            }
        }
    }
    false
}

fn is_dir_exist(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.exists() && path.is_dir()
}

pub fn has_content_in_dir(path: &str) -> bool {
    is_dir_exist(path) && is_dir_not_empty(path)
}