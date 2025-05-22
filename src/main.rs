use std::env;
use std::fs;
use std::path::Path;
use std::process;

// Add this to your Cargo.toml:
// [dependencies]
// pdf-extract = "0.7"
// clap = { version = "4.0", features = ["derive"] }

use clap::{Arg, Command};
use pdf_extract::extract_text;

fn main() {
    let matches = Command::new("PDF to Text Converter")
        .version("1.0")
        .about("Converts PDF files to text format for AI processing")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input PDF file path")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output text file path (optional, defaults to input name with .txt extension)"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let input_path = matches.get_one::<String>("input").unwrap();
    let verbose = matches.get_flag("verbose");

    // Check if input file exists
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path);
        process::exit(1);
    }

    // Determine output path
    let output_path = match matches.get_one::<String>("output") {
        Some(path) => path.clone(),
        None => {
            let input_path_obj = Path::new(input_path);
            let stem = input_path_obj.file_stem().unwrap().to_str().unwrap();
            format!("{}.txt", stem)
        }
    };

    if verbose {
        println!("Input file: {}", input_path);
        println!("Output file: {}", output_path);
        println!("Starting PDF text extraction...");
    }

    // Extract text from PDF
    match extract_text_from_pdf(input_path) {
        Ok(text) => {
            if verbose {
                println!("Successfully extracted {} characters", text.len());
            }

            // Process and clean the text
            let processed_text = process_extracted_text(&text);

            // Write to output file
            match fs::write(&output_path, processed_text) {
                Ok(_) => {
                    println!("✅ Successfully converted '{}' to '{}'", input_path, output_path);
                    if verbose {
                        println!("Text extraction complete!");
                    }
                }
                Err(e) => {
                    eprintln!("Error writing to output file: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error extracting text from PDF: {}", e);
            process::exit(1);
        }
    }
}

fn extract_text_from_pdf(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Extract text using pdf-extract directly with the file path
    let text = extract_text(file_path)?;
    
    Ok(text)
}

fn process_extracted_text(raw_text: &str) -> String {
    let mut processed = String::new();
    let mut prev_char = ' ';
    
    for line in raw_text.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines but preserve paragraph breaks
        if trimmed.is_empty() {
            if !processed.ends_with("\n\n") && !processed.is_empty() {
                processed.push('\n');
            }
            continue;
        }
        
        // Add line with proper spacing
        if !processed.is_empty() && !processed.ends_with('\n') {
            // Check if we need a space between words that got split across lines
            let last_char = processed.chars().last().unwrap_or(' ');
            let first_char = trimmed.chars().next().unwrap_or(' ');
            
            if last_char.is_alphanumeric() && first_char.is_alphanumeric() {
                processed.push(' ');
            } else if !last_char.is_whitespace() {
                processed.push(' ');
            }
        }
        
        processed.push_str(trimmed);
        prev_char = trimmed.chars().last().unwrap_or(' ');
    }
    
    // Clean up multiple consecutive spaces and normalize whitespace
    let cleaned = processed
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    
    // Add metadata header for AI context
    let mut result = String::new();
    result.push_str("=== PDF TEXT EXTRACTION ===\n");
    result.push_str("This text was extracted from a PDF file for AI processing.\n");
    result.push_str("Some formatting and layout information may be lost.\n");
    result.push_str("=== CONTENT BEGINS ===\n\n");
    result.push_str(&cleaned);
    result.push_str("\n\n=== CONTENT ENDS ===\n");
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_processing() {
        let raw_text = "This is a test\n   \n\nwith multiple    spaces\nand line breaks";
        let processed = process_extracted_text(raw_text);
        
        assert!(processed.contains("This is a test with multiple spaces and line breaks"));
        assert!(processed.contains("=== PDF TEXT EXTRACTION ==="));
    }
}