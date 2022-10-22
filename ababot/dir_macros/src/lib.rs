use core::panic;
use std::fs::{self, ReadDir};

use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, LitStr};

fn get_file_names(dir: ReadDir) -> Vec<String> {
    let mut names = Vec::new();
    for file in dir {
        match file {
            Ok(file) => {
                let os_name = file.file_name();
                let name = os_name.to_str();
                match name {
                    Some(name) => {
                        if name == "mod.rs" || !name.ends_with(".rs") {
                            continue;
                        }
                        let sanitized = name.split(".").next().unwrap();
                        names.push(sanitized.to_string());
                    }
                    None => panic!("Invalid file name: {:?}", os_name),
                }
            }
            Err(e) => panic!("Error reading file: {}", e),
        }
    }
    names
}

#[proc_macro]
pub fn import(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let dir_path = input.value();
    let dir = match fs::read_dir(dir_path) {
        Ok(dir) => dir,
        Err(e) => panic!("{}", e),
    };

    let imports = get_file_names(dir)
        .iter()
        .map(|name| format!("pub mod {};", name))
        .collect::<Vec<String>>()
        .join("");

    match imports.parse() {
        Ok(tok_stream) => tok_stream,
        Err(e) => panic!("Error parsing: {}", e),
    }
}

struct InvocationTarget {
    directory: LitStr,
    function_name: LitStr,
}

impl Parse for InvocationTarget {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let directory: LitStr = input.parse()?;
        let function_name: LitStr = input.parse()?;
        Ok(InvocationTarget {
            directory,
            function_name,
        })
    }
}

#[proc_macro]
pub fn invoke(input: TokenStream) -> TokenStream {
    let InvocationTarget {
        directory,
        function_name,
    } = parse_macro_input!(input as InvocationTarget);

    let dir = match fs::read_dir(directory.value()) {
        Ok(dir) => dir,
        Err(e) => panic!("{}", e),
    };

    let names = get_file_names(dir);
    let mut output = String::from(" match input {\n");
    for name in names {
        let val = directory.value();
        let mut iterator = val.split("/").map(|s| s.to_string());
        iterator.next();
        output.push_str(&format!(
            "\"{}\" => {}::{}::{}(),\n",
            name,
            iterator.collect::<Vec<String>>().join("::"),
            name,
            function_name.value()
        ))
    }
    output.push_str("_ => \"Oh no\".to_string()\n}");

    match output.parse() {
        Ok(tok_stream) => tok_stream,
        Err(e) => panic!("Error parsing: {}", e),
    }
}