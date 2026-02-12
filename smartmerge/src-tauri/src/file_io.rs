use std::fs;
use std::path::Path;

pub fn read_lines(path: &str) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(Path::new(path)).map_err(|err| err.to_string())?;
    if content.is_empty() {
        return Ok(Vec::new());
    }
    let lines = content
        .split_inclusive('\n')
        .map(|line| line.to_string())
        .collect();
    Ok(lines)
}

pub fn write_lines(path: &str, lines: &[String]) -> Result<(), String> {
    let content: String = lines.iter().cloned().collect();
    fs::write(Path::new(path), content).map_err(|err| err.to_string())
}
