use std::fs::{self, ReadDir};

use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, LitStr, token::Token};

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
                        let sanitized = name.split('.').next().unwrap();
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
    let dir = match fs::read_dir(&dir_path) {
        Ok(dir) => dir,
        Err(e) => panic!("Error opening directory {}: {}", dir_path, e),
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
    rust_path: LitStr,
    function_name: LitStr,
}

impl Parse for InvocationTarget {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let directory: LitStr = input.parse()?;
        let rust_path: LitStr = input.parse()?;
        let function_name: LitStr = input.parse()?;
        Ok(InvocationTarget {
            directory,
            rust_path,
            function_name,
        })
    }
}

#[proc_macro]
pub fn run_commands_async(input: TokenStream) -> TokenStream {
    let InvocationTarget {
        directory,
        rust_path,
        function_name,
    } = parse_macro_input!(input as InvocationTarget);

    let dir = match fs::read_dir(directory.value()) {
        Ok(dir) => dir,
        Err(e) => panic!("{}", e),
    };

    let names = get_file_names(dir);
    let mut output = String::from(" match input {\n");
    for name in names {
        output.push_str(&format!(
            "\"{}\" => {}::{}::{}.await,\n",
            name,
            rust_path.value(),
            name,
            function_name.value()
        ))
    }
    output.push_str("_ => nop().await\n}");

    match output.parse() {
        Ok(tok_stream) => tok_stream,
        Err(e) => panic!("Error parsing: {}", e),
    }
}

#[proc_macro]
pub fn long_running(input: TokenStream) -> TokenStream {
    let InvocationTarget {
        directory,
        rust_path,
        function_name,
    } = parse_macro_input!(input as InvocationTarget);

    let dir = match fs::read_dir(directory.value()) {
        Ok(dir) => dir,
        Err(e) => panic!("{}", e),
    };

    let names = get_file_names(dir);
    let mut output = String::new();
    for name in names {
        output.push_str(&format!(
            "let ctx_cpy = Arc::clone(&ctx);\ntokio::spawn(async move {{{}::{}::{}.await}});",
            rust_path.value(),
            name,
            function_name.value()
        ))
    }

    output.parse().unwrap()
}


#[proc_macro]
pub fn run_commands(input: TokenStream) -> TokenStream {
    let InvocationTarget {
        directory,
        rust_path,
        function_name,
    } = parse_macro_input!(input as InvocationTarget);

    let dir = match fs::read_dir(directory.value()) {
        Ok(dir) => dir,
        Err(e) => panic!("{}", e),
    };

    let names = get_file_names(dir);
    let mut output = String::from(" match input {\n");
    for name in names {
        output.push_str(&format!(
            "\"{}\" => {}::{}::{},\n",
            name,
            rust_path.value(),
            name,
            function_name.value()
        ))
    }
    output.push_str("_ => \"Unrecognized command\".to_string()\n}");

    match output.parse() {
        Ok(tok_stream) => tok_stream,
        Err(e) => panic!("Error parsing: {}", e),
    }
}

#[proc_macro]
pub fn register_commands(input: TokenStream) -> TokenStream {
    let InvocationTarget {
        directory,
        rust_path,
        function_name,
    } = parse_macro_input!(input as InvocationTarget);

    let dir = match fs::read_dir(directory.value()) {
        Ok(dir) => dir,
        Err(e) => panic!("{}", e),
    };

    // commands.create_application_command(|command| commands::ping::register(command))
    let names = get_file_names(dir);
    let mut output = String::from("commands\n");
    for name in names {
        output.push_str(&format!(
            ".create_application_command(|command| {}::{}::{})\n",
            rust_path.value(),
            name,
            function_name.value()
        ))
    }

    match output.parse() {
        Ok(tok_stream) => tok_stream,
        Err(e) => panic!("Error parsing: {}", e),
    }
}
