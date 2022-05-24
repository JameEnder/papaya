use std::{collections::HashMap, fs};
use std::env;
use regex::Regex;
use lazy_static::lazy_static;

type AnyError = Box<dyn std::error::Error>;

fn load_variables_from_files(files: &Vec<String>) -> std::io::Result<HashMap<String, String>> {  
    let mut variables: HashMap<String, String> = HashMap::new();

    for file_name in files {
        let content = fs::read_to_string(file_name)?;

        for line in content.split("\n") {
            let splitted: Vec<&str> = line.split("=").map(|s| s.trim()).collect();

            if splitted.len() < 2 {
                continue;
            }
            
            let variable_name = splitted[0];
            let variable_value = splitted[1];
    
            variables.insert(variable_name.into(), variable_value.into());
        }
    }

    Ok(variables)
}

fn html_find_includes(content: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"@include "(.*?)""#).unwrap();
    }

    let file_names: Vec<String> = RE.captures_iter(&content).map(|cap| cap[1].to_string()).collect();

    file_names
}

fn html_find_variables(content: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"@variables "(.*?)""#).unwrap();
    }

    let file_names: Vec<String> = RE.captures_iter(&content).map(|cap| cap[1].to_string()).collect();

    file_names
}

fn html_insert_includes(content: &str, includes: &Vec<String>) -> Result<String, AnyError> {
    let mut result_html = content.to_string();
    
    for file_name in includes {
        let content = fs::read_to_string(&file_name)?;
        let output_html = html_render_final_html(&content)?;
        
        let re = Regex::new(&format!(r#"\t+@include\s+"{}""#, file_name)).unwrap();
        result_html = re.replace_all(&result_html, output_html).to_string();
    }
    
    Ok(result_html)
}

fn html_cleanup(content: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"\n\s+@variables\s+"(.+?)"\s+?"#).unwrap();
    }
    
    RE.replace_all(&content, "").to_string()
}

fn html_insert_variables(content: &str, variables: &HashMap<String, String>) -> Result<String, regex::Error> {
    let mut result_html = content.to_string(); 
    
    for (variable_name, variable_value) in variables {
        let re = Regex::new(&format!(r#"\{{\{{\s+{}\s+\}}\}}"#, variable_name))?;
        result_html = re.replace_all(&result_html, variable_value).to_string();
    }

    Ok(result_html)
}

fn html_render_final_html(content: &str) -> Result<String, AnyError> {
    let includes = html_find_includes(&content);

    let output_html = html_insert_includes(&content, &includes)?;

    let variables_files = html_find_variables(&output_html);
    let variables = load_variables_from_files(&variables_files)?;

    let output_html = html_insert_variables(&output_html, &variables)?;

    Ok(html_cleanup(&output_html))
}

fn main() -> Result<(), AnyError> {
    if env::args().len() < 2 {
        eprintln!("error: missing arguments");
        eprintln!("usage: {} [FILE] [OUTPUT?]", env::args().nth(0).unwrap());
        std::process::exit(1);
    }

    let file_name = env::args().nth(1).unwrap();

    let content = fs::read_to_string(file_name)?;

    let output_html = html_render_final_html(&content)?;

    if env::args().len() == 3 {
        fs::write(env::args().nth(2).unwrap(), output_html)?;
    } else {
        println!("{}", output_html);
    }

    Ok(())
}