use iter::IntoParallelRefIterator;
use iter::ParallelIterator;
use num_traits::Num;
use rand::Rng;
use rayon::*;
use slice::ParallelSlice;
use std::env;
use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::io::{Read, Write};
use num_bigint::BigInt;
use num_traits::{Zero, ToPrimitive};
use num_integer::Integer;

const LENGTH_OF_PAGE: usize = 3239;
const PAD_CHAR: char = '.';

// Calculate powers for location multiplier
fn calculate_loc_mult(length: u32) -> BigInt {
    let thirty = BigInt::from(30u32);
    thirty.pow(length)
}

fn bytes_to_babel_text(bytes: &[u8]) -> String {
    // Process conversion in parallel for large inputs
    if bytes.len() > 1024 {  // Only parallelize for larger inputs
        bytes.par_iter()
            .map(|&byte| {
                let first = byte / 26;
                let second = byte % 26;
                format!("{}{}", 
                    char::from(b'a' + first),
                    char::from(b'a' + second))
            })
            .collect()
    } else {
        bytes.iter()
            .map(|&byte| {
                let first = byte / 26;
                let second = byte % 26;
                format!("{}{}", 
                    char::from(b'a' + first),
                    char::from(b'a' + second))
            })
            .collect()
    }
}

fn babel_text_to_bytes(text: &str) -> Vec<u8> {
    let text = text.trim_end_matches(PAD_CHAR);
    let chars: Vec<char> = text.chars().collect();
    
    // Process conversion in parallel for large inputs
    if chars.len() > 2048 {  // Only parallelize for larger inputs
        chars.par_chunks(2)
            .filter(|chunk| chunk.len() == 2)
            .map(|chunk| {
                let first = (chunk[0] as u8 - b'a') * 26;
                let second = chunk[1] as u8 - b'a';
                first + second
            })
            .collect()
    } else {
        let mut bytes = Vec::with_capacity(chars.len() / 2);
        for chunk in chars.chunks(2) {
            if chunk.len() == 2 {
                let first = (chunk[0] as u8 - b'a') * 26;
                let second = chunk[1] as u8 - b'a';
                bytes.push(first + second);
            }
        }
        bytes
    }
}

fn string_to_number(input: &str) -> BigInt {
    let digits: Vec<char> = "abcdefghijklmnopqrstuvwxyz, .".chars().collect();
    let base = BigInt::from(29u32);
    let mut result = BigInt::zero();
    
    for c in input.chars() {
        if let Some(pos) = digits.iter().position(|&x| x == c) {
            result = result * &base + BigInt::from(pos);
        }
    }
    result
}

fn int_to_base36(mut x: BigInt) -> String {
    if x.is_zero() {
        return "0".to_string();
    }

    let digits: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    let mut result = Vec::new();
    let thirty_six = BigInt::from(36u32);
    let zero = BigInt::zero();
    
    while x > zero {
        let (new_x, remainder) = x.div_rem(&thirty_six);
        result.push(digits[remainder.to_u32().unwrap_or(0) as usize]);
        x = new_x;
    }
    
    result.into_iter().rev().collect()
}

fn to_text(mut x: BigInt) -> String {
    let digits: Vec<char> = "abcdefghijklmnopqrstuvwxyz, .".chars().collect();
    let mut result = Vec::new();
    let twenty_nine = BigInt::from(29u32);
    
    if x.is_zero() {
        return "a".to_string();
    }
    
    // Convert number to base-29 digits
    while x > Zero::zero() {
        let (new_x, remainder) = x.div_rem(&twenty_nine);
        result.push(digits[remainder.to_usize().unwrap_or(0)]);
        x = new_x;
    }
    result.reverse();
    
    // Convert to string
    let mut text: String = result.into_iter().collect();
    
    // Left-pad with 'a' if we're short
    if text.len() < LENGTH_OF_PAGE {
        let padding = "a".repeat(LENGTH_OF_PAGE - text.len());
        text = format!("{}{}", padding, text);
    }
    
    text
}

// Verify page retrieval
fn verify_page(original: &str, address: &str) -> bool {
    let retrieved = get_page(address);
    let retrieved = retrieved.trim_end_matches(PAD_CHAR);
    let original_trimmed = original.trim_end_matches(PAD_CHAR);
    
    if original_trimmed.len() != retrieved.len() {
        println!("Length mismatch after trimming!");
        println!("Original length: {}", original_trimmed.len());
        println!("Retrieved length: {}", retrieved.len());
        println!("Original last 10 chars: {:?}", original_trimmed.chars().rev().take(10).collect::<Vec<_>>());
        println!("Retrieved last 10 chars: {:?}", retrieved.chars().rev().take(10).collect::<Vec<_>>());
        
        // If lengths differ, print the first differing position
        let orig_chars: Vec<char> = original_trimmed.chars().collect();
        let retr_chars: Vec<char> = retrieved.chars().collect();
        for i in 0..std::cmp::min(orig_chars.len(), retr_chars.len()) {
            if orig_chars[i] != retr_chars[i] {
                println!("First difference at position {}", i);
                println!("Original char: {:?}", orig_chars[i]);
                println!("Retrieved char: {:?}", retr_chars[i]);
                break;
            }
        }
        return false;
    }
    
    if original_trimmed != retrieved {
        println!("Content mismatch after trimming!");
        println!("Original (trimmed) [{} chars]: {}", original_trimmed.len(), original_trimmed);
        println!("Retrieved [{} chars]: {}", retrieved.len(), retrieved);
        println!("Address: {}", address);
        false
    } else {
        true
    }
}


fn search(search_str: &str) -> String {
    assert_eq!(search_str.len(), LENGTH_OF_PAGE, 
              "Search string must be exactly {} characters", LENGTH_OF_PAGE);
    
    let mut rng = rand::thread_rng();
    let wall = rng.gen_range(0..4).to_string();
    let shelf = rng.gen_range(0..5).to_string();
    let volume = format!("{:02}", rng.gen_range(0..32));
    let page = format!("{:03}", rng.gen_range(0..410));

    let loc_str = format!("{}{}{}{}", page, volume, shelf, wall);
    let loc_int = BigInt::parse_bytes(loc_str.as_bytes(), 10).unwrap();
    let loc_mult = calculate_loc_mult(LENGTH_OF_PAGE as u32);
    
    let search_num = string_to_number(search_str);
    let hex_addr = int_to_base36(search_num + (loc_int * loc_mult));
    let address = format!("{}:{}:{}:{}:{}", hex_addr, wall, shelf, volume, page);
    
    // Verify the page can be correctly retrieved
    if !verify_page(search_str, &address) {
        panic!("Page verification failed during search!");
    }
    
    address
}

fn encode_file(input_path: &str, output_path: Option<&str>) -> std::io::Result<()> {
    println!("Reading input file...");
    let mut file = fs::File::open(input_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    
    let extension = Path::new(input_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    println!("Converting to babel text...");
    let babel_text = bytes_to_babel_text(&contents);
    
    // Debug: Verify conversion is working
    println!("Verifying initial conversion...");
    let test_bytes = babel_text_to_bytes(&babel_text);
    if test_bytes != contents {
        panic!("Initial conversion verification failed!");
    }
    
    println!("Splitting into pages...");
    let chunks: Vec<String> = babel_text
        .chars()
        .collect::<Vec<char>>()
        .chunks(LENGTH_OF_PAGE)
        .map(|c| {
            let chunk_str: String = c.iter().collect();
            if chunk_str.len() < LENGTH_OF_PAGE {
                format!("{}{}", chunk_str, PAD_CHAR.to_string().repeat(LENGTH_OF_PAGE - chunk_str.len()))
            } else {
                chunk_str
            }
        })
        .collect();

    println!("Finding locations for {} pages in parallel...", chunks.len());
    let locations: Vec<(String, String)> = chunks.par_iter()
        .map(|chunk| {
            assert_eq!(chunk.len(), LENGTH_OF_PAGE, 
                      "Chunk length {} != {}", chunk.len(), LENGTH_OF_PAGE);
            let location = search(chunk);
            (chunk.clone(), location)
        })
        .collect();

    println!("Verifying all pages in parallel...");
    let verification_failed = locations.par_iter()
        .any(|(original, location)| !verify_page(original, location));

    if verification_failed {
        panic!("Page verification failed!");
    }

    let output_path = match output_path {
        Some(path) => path.to_string(),
        None => {
            let mut path = Path::new(input_path).to_path_buf();
            path.set_extension("babel");
            path.to_string_lossy().to_string()
        }
    };

    println!("Writing to {}...", output_path);
    let output_file = fs::File::create(&output_path)?;
    let mut writer = BufWriter::new(output_file);
    
    writeln!(writer, "{}", extension)?;
    writeln!(writer, "{}", contents.len())?;
    
    for (_, location) in locations {
        writeln!(writer, "{}", location)?;
    }
    
    writer.flush()?;
    println!("Encoding complete!");
    Ok(())
}

fn decode_file(input_path: &str, output_path: Option<&str>) -> std::io::Result<()> {
    println!("Reading babel file...");
    let contents = fs::read_to_string(input_path)?;
    let mut lines = contents.lines();
    
    let extension = lines.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "File is empty")
    })?;
    
    let original_size = lines.next()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid file size")
        })?;
    
    println!("Decoding {} bytes...", original_size);
    
    let locations: Vec<&str> = lines.collect();
    println!("Found {} pages to decode", locations.len());
    
    println!("Decoding pages in parallel...");
    let decoded_chunks: Vec<String> = locations.par_iter()
        .map(|&location| {
            let page_content = get_page(location);
            if let Some(last_non_period) = page_content.rfind(|c| c != PAD_CHAR) {
                page_content[..=last_non_period].to_string()
            } else {
                String::new()
            }
        })
        .collect();
    
    let decoded_text = decoded_chunks.join("");
    
    println!("Converting to bytes...");
    let mut bytes = babel_text_to_bytes(&decoded_text);
    
    println!("Original size: {}, Decoded size: {}", original_size, bytes.len());
    bytes.truncate(original_size);
    
    let output_path = match output_path {
        Some(path) => path.to_string(),
        None => Path::new(input_path)
            .with_extension(extension)
            .to_string_lossy()
            .to_string()
    };

    println!("Writing to {}", output_path);
    fs::write(output_path, bytes)?;
    
    println!("Decoding complete!");
    Ok(())
}

fn get_page(address: &str) -> String {
    let parts: Vec<&str> = address.split(':').collect();
    let hex_addr = parts[0];
    let wall = parts[1];
    let shelf = parts[2];
    let volume = format!("{:02}", parts[3]);
    let page = format!("{:03}", parts[4]);

    let loc_str = format!("{}{}{}{}", page, volume, shelf, wall);
    let loc_int = BigInt::parse_bytes(loc_str.as_bytes(), 10).unwrap();
    let loc_mult = calculate_loc_mult(LENGTH_OF_PAGE as u32);
    
    let key = BigInt::from_str_radix(hex_addr, 36).unwrap() - (loc_int * loc_mult);
    let result = to_text(key);
    
    assert_eq!(result.len(), LENGTH_OF_PAGE, 
              "Generated page must be exactly {} characters", LENGTH_OF_PAGE);
    
    result
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args.len() > 5 {
        println!("Usage:");
        println!("  Encode: {} --encode <input_file> [output_file]", args[0]);
        println!("  Decode: {} --decode <input_file> [output_file]", args[0]);
        return;
    }

    let command = &args[1];
    let input_path = &args[2];
    let output_path = args.get(3).map(|s| s.as_str());

    match command.as_str() {
        "--encode" => {
            println!("Starting encoding process...");
            match encode_file(input_path, output_path) {
                Ok(_) => println!("File encoded successfully"),
                Err(e) => eprintln!("Error encoding file: {}", e),
            }
        },
        "--decode" => {
            println!("Starting decoding process...");
            match decode_file(input_path, output_path) {
                Ok(_) => println!("File decoded successfully"),
                Err(e) => eprintln!("Error decoding file: {}", e),
            }
        },
        _ => {
            println!("Unknown command. Use --encode or --decode");
        }
    }
}