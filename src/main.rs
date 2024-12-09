use std::{cell::RefCell, fs, io::Read, rc::Rc};
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

    // 使用 encoding_rs 处理 BOM
    let (cow ,encoding, had_errors) =encoding_rs::UTF_8.decode(&buf);

    // 获取去除 BOM 后的有效数据
    let content = String::from(cow).to_string();
    Ok(content)
}

fn main() {
     
    let mod_path = String::from("C:\\Users\\HXY\\AppData\\Roaming\\r2modmanPlus-local\\LethalCompany\\profiles\\Default\\BepInEx\\plugins");
    //std::io::stdin().read_line(&mut mod_path).expect("Failed to read line");
    let paths = fs::read_dir(mod_path.trim()).expect("Failed to read directory");
    let mut modlist: Vec<Manifest> = vec![];
    for path in paths {
        println!("{}",1);
        let dir = path.expect("Failed to get path").path();
        if !dir.is_dir() {
            continue;
        }

        let manifest_path = dir.join("manifest.json");
        if manifest_path.exists() {
            let manifest_content = read_utf8_with_bom(manifest_path.to_str().expect("Failed to convert path to string")).expect("Failed to read manifest.json");
            let manifest: Manifest = serde_json::from_str(&manifest_content).unwrap_or_else(|e| {
                eprintln!("parse- error: {}", e);
                eprintln!("context: {}", manifest_content);
                panic!("Failed to parse manifest.json");
            });
            modlist.push(manifest);
        }
    }
    // 我现在有一个modlist: Vec<Manifest> ，其中Manifest中的dependencies 的依赖格式是 “{author}-{name}-{version}”
    // 通过这种格式指向其他若干个(0或n个)其他Manifest。
    // 你需要构建一个依赖图。同时找出一个最小化的集合（可以是plugins列表形式），其中每个plugins都不被其他依赖。
    let mut dependency_map= std::collections::HashMap::new();
    let mut plugins_list = Vec::new();

    for manifest in modlist {
        let plugin = Rc::new(RefCell::new(Plugins::from(manifest.clone()))); // Wrap in RefCell
        plugins_list.push(plugin.clone());
        let pb =plugin.borrow();
        let key_format = format!("{}-{}-{}", 
                    if pb.author.is_empty() { "" } else { &pb.author },
                    if pb.name.is_empty() { "" } else { &pb.name },
                    if pb.version.is_empty() { "" } else { &pb.version }
                ); 
        drop(pb);
        let key = key_format.trim_matches('-'); // Remove trailing underscores if any
        dependency_map.insert(key.to_string(), plugin);
    }



    let independent_plugins = plugins_list.iter()
        .filter(|p| {
            let plugin_borrowed = p.borrow();
            !plugins_list.iter().any(|other| {
                let other_borrowed = other.borrow();
                other_borrowed.dependency.iter().any(|dep| Rc::ptr_eq(dep, p))
            })
        }).collect::<Vec<_>>();

    // Output the independent plugins
    for plugin in independent_plugins {
        let bp = plugin.borrow();
        println!("Independent Plugin: {} by {}", bp.name, bp.author);
    }

}

