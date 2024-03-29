use std::error::Error;

const PUB_IMPORT: &str = "// EOF pub mods";

// fn create_file() -> R<()> {
//     unimplemented!()
// }

// fn add_mod(_file: &str, _module_name: &str) -> R<()> {
//     unimplemented!()
// }
//
// fn add_field(_file: &str, _module_name: &str) -> R<()> {
//     unimplemented!()
// }
//
// fn add_to_feld_to_new(_file: &str, _module_name: &str) -> R<()> {
//     unimplemented!()
// }

fn main() -> Result<(), Box<dyn Error>> {
    let name = match std::env::args().nth(1) {
        Some(name) if !name.is_empty() && &name != "help" && &name != "--help" => name,
        _ => {
            println!("Invalid struct name. From the git root directory run:");
            println!("cargo run --bin template Squad");
            std::process::exit(1);
        }
    };

    let name_lower = name.to_lowercase();

    // read template file, replace variables inside, put file in the project
    {
        let template = std::fs::read_to_string("mud-domain/src/game/template.rs")
            .expect("template file not found");
        let result_template = template
            .replace("Template", name.as_str())
            .replace("template", &name_lower);

        let file = format!("mud-domain/src/game/{}.rs", &name_lower);

        println!("creating {}", file);

        std::fs::write(file, result_template)?;
    }

    {
        let mut body =
            std::fs::read_to_string("mud-domain/src/game.rs").expect("game file not found");

        let new_import = format!("pub mod {};\n", &name_lower);
        if body.contains(&new_import) {
            println!("pub mod already exists in game.rs");
        } else {
            println!("adding pub mod to game.rs");

            let index = body
                .find(PUB_IMPORT)
                .expect("not found PUB_IMPORT notation in game.rs ");
            body.insert_str(index, &new_import);
            std::fs::write("mud-domain/src/game.rs", body)?;
        }
    }
    // open container file, add attribute, add initializer
    {
        let file = "mud-domain/src/game/container.rs";
        let mut changed = false;
        let mut body = std::fs::read_to_string(file)?;

        let start = body
            .find("pub struct Container {")
            .expect("could not found container struct");
        let end = body[start..].find("}").expect("could not find struct end") + start;
        let struct_body = &body[start..end];

        let reference = format!("    pub {}: {},\n", name_lower, name);
        if struct_body.contains(&reference) {
            println!("container already have reference")
        } else {
            body.insert_str(end, &reference);
            changed = true;
        }

        if changed {
            println!("update{}", file);
            std::fs::write(file, body)?;
        }
    }
    // create loader, add data, add apply_data and snapshot_obj
    Ok(())
}
