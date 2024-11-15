# Library of Babel File Encoder

This tool allows you to encode any file into a series of Library of Babel page references, and decode them back into the original file.

## How it Works

1. Files are converted into a series of a-z characters using a custom encoding
2. The text is split into chunks that fit on Library of Babel pages (3239 characters each)
3. For each chunk, a page is found in the Library that contains that exact text
4. The page references are stored in a .babel file
5. When decoding, the tool retrieves each page and reconstructs the original file

## Installation

### From Source
```bash
# Clone the repository
git clone https://github.com/Gaijutsu/babel-encoding
cd babel-encoder

# Build
cargo build --release

# The executable will be in target/release/
```

## Usage

### Encoding a File
```bash
# Default output (adds .babel extension)
./babel-encoder --encode input.txt

# Custom output path
./babel-encoder --encode input.txt output.babel
```

### Decoding a File
```bash
# Default output (uses original extension)
./babel-encoder --decode input.babel

# Custom output path
./babel-encoder --decode input.babel output.txt
```

## File Format
The .babel file format is as follows:
- Line 1: Original file extension
- Line 2: Original file size in bytes
- Remaining lines: Library of Babel page references, one per line

## Technical Details

### Page Structure
- Each page contains exactly 3239 characters
- Characters allowed: a-z, space, comma, period
- Pages are identified by wall:shelf:volume:page coordinates

### Encoding Process
1. File bytes are converted to a-z pairs
2. Text is split into 3239-character chunks
3. Each chunk is padded with periods if needed
4. A mathematical transformation finds the exact page containing each chunk

## Building from Source
```bash
cargo build --release
```