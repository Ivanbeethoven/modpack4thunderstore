use std::{cell::RefCell, collections::HashMap, fs, io::Read, rc::Rc};
use serde;

#[derive(Clone, PartialEq)]
struct Plugins{
    name:String,
    author:String,
    version:String,
    dependency: Vec<Rc<RefCell<Plugins>>>, // Use RefCell to enable mutation
}
#[derive(Debug, Clone, serde::Deserialize)]
struct Manifest {
    name: String,
    author: Option<String>, // Change to Option<String> to allow for empty author
    version_number: String,
    website_url: String,
    description: String,
    dependencies: Vec<String>,
}
impl From<Manifest> for Plugins {
    fn from(manifest: Manifest) -> Self {
        Plugins {
            name: manifest.name,
            author: manifest.author.unwrap_or_default(),
            version: manifest.version_number,
            dependency: Vec::new(), // Initialize as an empty vector
        }
    }
}

fn read_utf8_with_bom(file_path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(file_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // Use encoding_rs to handle BOM
    let (cow ,encoding, had_errors) = encoding_rs::UTF_8.decode(&buf);

    // Get the valid data after removing BOM
    let content = String::from(cow).to_string();
    Ok(content)
}

fn main() {
     
    let mod_path = String::from("C:\\Users\\HXY\\AppData\\Roaming\\r2modmanPlus-local\\LethalCompany\\profiles\\Default\\BepInEx\\plugins");
    // let user_name = std::env::var("USERNAME").unwrap_or_else(|_| "User".to_string());
    // let default_param = "Default"; // Replace with your parameter as needed
    // let mod_path = format!("C:\\Users\\{}\\AppData\\Roaming\\r2modmanPlus-local\\LethalCompany\\profiles\\{}\\BepInEx\\plugins", user_name, default_param);
    //std::io::stdin().read_line(&mut mod_path).expect("Failed to read line");
    let paths = fs::read_dir(mod_path.trim()).expect("Failed to read directory");
    let mut modlist: Vec<Manifest> = vec![];
    for path in paths {
        let dir = path.expect("Failed to get path").path();
        if !dir.is_dir() {
            continue;
        }

        let manifest_path = dir.join("manifest.json");
        if manifest_path.exists() {
            let manifest_content = read_utf8_with_bom(manifest_path.to_str().expect("Failed to convert path to string")).expect("Failed to read manifest.json");
            let mut manifest: Manifest = serde_json::from_str(&manifest_content).unwrap_or_else(|e| {
                eprintln!("parse- error: {}", e);
                eprintln!("context: {}", manifest_content);
                panic!("Failed to parse manifest.json");
            });
            let author_modname = dir.file_name().unwrap();
            let author_modname_str = author_modname.to_string_lossy(); // Convert file name to string
            let parts: Vec<&str> = author_modname_str.split('-').collect(); // Split by "-" into two strings
            if parts.len() < 2 {
                continue;
            }
            manifest.author = Some(parts[0].to_string());
            manifest.name = parts[1].to_string();
            //println!("{:?}", manifest);
            modlist.push(manifest);
        }   
       
    }

    let mut modmap: HashMap<String, bool> = HashMap::new();
    for modifest in &modlist {
        if let Some(a) = modmap.get_mut(&modifest.name) {
            // Already depended on.
        } else {
            modmap.insert(modifest.name.clone(), false);
        }

        for modifest_dep in &modifest.dependencies {
            let parts: Vec<&str> = modifest_dep.split('-').collect();
            modmap.insert(parts[1].to_owned(), true);
        }
        // modmap.insert(modifest., v)
    }
    let mut dep_list = vec![];
    for m in &modlist {
        if !*modmap.get(&m.name).unwrap() {
            dep_list.push(format!("{}-{}-{}", m.author.clone().unwrap(), m.name, m.version_number));
        }
    }
    let modpack = serde_json::json!({
        "name": "Modpackhxy",
        "version_number": "1.2.0",
        "website_url": "https://gitee.com/hanxiaoyang1/lc_modpack",
        "description": "sksb",
        "dependencies": dep_list,
    });

    // Print the generated modpack JSON
    //println!("{}", serde_json::to_string_pretty(&modpack).unwrap());
    let output_path = "./test.json"; // Specify your desired output path
    let json_content = serde_json::to_string_pretty(&modpack).expect("Failed to serialize modpack to JSON");
    fs::write(output_path, json_content).expect("Failed to write JSON to file");
}
