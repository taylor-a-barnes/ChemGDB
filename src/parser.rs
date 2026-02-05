use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader, Read};

/// Atom data parsed from XYZ file
#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
  pub element: String,
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

/// Molecule containing parsed atoms
#[derive(Debug, Clone, PartialEq)]
pub struct Molecule {
  pub atoms: Vec<Atom>,
  pub comment: String,
}

/// Parser error types
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
  EmptyFile,
  InvalidAtomCount(String),
  MissingCommentLine,
  InvalidAtomLine(usize, String),
  InvalidCoordinate(usize, String),
  AtomCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ParseError::EmptyFile => write!(f, "empty file"),
      ParseError::InvalidAtomCount(msg) => write!(f, "invalid atom count: {}", msg),
      ParseError::MissingCommentLine => write!(f, "missing comment line"),
      ParseError::InvalidAtomLine(line, msg) => {
        write!(f, "invalid atom line at line {}: {}", line, msg)
      }
      ParseError::InvalidCoordinate(line, msg) => {
        write!(f, "invalid coordinate at line {}: {}", line, msg)
      }
      ParseError::AtomCountMismatch { expected, actual } => {
        write!(
          f,
          "atom count mismatch: expected {} atoms, found {}",
          expected, actual
        )
      }
    }
  }
}

impl Error for ParseError {}

/// Parse an XYZ file from a reader
pub fn parse_xyz<R: Read>(reader: R) -> Result<Molecule, ParseError> {
  let buf_reader = BufReader::new(reader);
  let lines: Vec<String> = buf_reader
    .lines()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| ParseError::InvalidAtomCount(e.to_string()))?;

  // Check for empty file (no lines or only whitespace)
  if lines.is_empty() || lines.iter().all(|l| l.trim().is_empty()) {
    return Err(ParseError::EmptyFile);
  }

  // First line: atom count
  let first_line = lines.first().ok_or(ParseError::EmptyFile)?;
  let atom_count_str = first_line.trim();

  if atom_count_str.is_empty() {
    return Err(ParseError::EmptyFile);
  }

  // Check for non-integer (decimal point)
  if atom_count_str.contains('.') {
    return Err(ParseError::InvalidAtomCount(format!(
      "'{}' is not an integer",
      atom_count_str
    )));
  }

  let atom_count: i64 = atom_count_str
    .parse()
    .map_err(|_| ParseError::InvalidAtomCount(format!("'{}' is not a valid integer", atom_count_str)))?;

  // Check for negative atom count
  if atom_count < 0 {
    return Err(ParseError::InvalidAtomCount(format!(
      "'{}' is negative",
      atom_count
    )));
  }

  let atom_count = atom_count as usize;

  // Second line: comment (must exist even if empty)
  if lines.len() < 2 {
    return Err(ParseError::MissingCommentLine);
  }

  let comment = lines[1].clone();

  // Parse atom lines (starting from line 3, index 2)
  let mut atoms = Vec::with_capacity(atom_count);
  let atom_lines = &lines[2..];

  // We need exactly atom_count valid atom lines
  for i in 0..atom_count {
    let line_num = i + 3; // 1-indexed, starting from line 3

    // Check if we have enough lines
    if i >= atom_lines.len() {
      return Err(ParseError::AtomCountMismatch {
        expected: atom_count,
        actual: i,
      });
    }

    let line = &atom_lines[i];
    let trimmed = line.trim();

    // Empty lines in atom section are invalid
    if trimmed.is_empty() {
      return Err(ParseError::InvalidAtomLine(
        line_num,
        "empty line in atom section".to_string(),
      ));
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    // Need at least element + 3 coordinates
    if parts.len() < 4 {
      return Err(ParseError::InvalidAtomLine(
        line_num,
        format!("expected at least 4 fields, found {}", parts.len()),
      ));
    }

    let element = parts[0];

    // Check if element looks like a number (invalid - should be alphanumeric starting with letter)
    if element.chars().next().map_or(true, |c| c.is_ascii_digit() || c == '-' || c == '+' || c == '.') {
      return Err(ParseError::InvalidAtomLine(
        line_num,
        format!("element symbol '{}' appears to be a number", element),
      ));
    }

    // Parse coordinates
    let x = parse_coordinate(parts[1], line_num)?;
    let y = parse_coordinate(parts[2], line_num)?;
    let z = parse_coordinate(parts[3], line_num)?;

    atoms.push(Atom {
      element: element.to_string(),
      x,
      y,
      z,
    });
  }

  // Check if there are extra atom lines beyond what was declared
  let remaining_lines = &atom_lines[atom_count..];
  let extra_atom_lines = remaining_lines
    .iter()
    .filter(|l| !l.trim().is_empty())
    .count();

  if extra_atom_lines > 0 {
    return Err(ParseError::AtomCountMismatch {
      expected: atom_count,
      actual: atom_count + extra_atom_lines,
    });
  }

  Ok(Molecule { atoms, comment })
}

/// Parse a coordinate value, rejecting NaN and Inf
fn parse_coordinate(s: &str, line_num: usize) -> Result<f64, ParseError> {
  let lower = s.to_lowercase();

  // Reject special values
  if lower == "nan" || lower == "inf" || lower == "-inf" || lower == "+inf" {
    return Err(ParseError::InvalidCoordinate(
      line_num,
      format!("'{}' is not a valid coordinate (NaN/Inf not allowed)", s),
    ));
  }

  let value: f64 = s.parse().map_err(|_| {
    ParseError::InvalidCoordinate(line_num, format!("'{}' is not a valid number", s))
  })?;

  // Double-check for NaN/Inf after parsing (in case of edge cases)
  if value.is_nan() || value.is_infinite() {
    return Err(ParseError::InvalidCoordinate(
      line_num,
      format!("'{}' resulted in NaN or Infinity", s),
    ));
  }

  Ok(value)
}

/// Parse XYZ content from a string
pub fn parse_xyz_str(content: &str) -> Result<Molecule, ParseError> {
  parse_xyz(content.as_bytes())
}

#[cfg(test)]
mod tests {
  use super::*;

  // Helper to check approximate coordinate equality
  fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
  }

  // ==================== Valid File Parsing ====================

  #[test]
  fn test_parse_valid_xyz_file() {
    let content = "2\nWater molecule\nO 0.0 0.0 0.0\nH 0.96 0.0 0.0\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms.len(), 2);
    assert_eq!(result.atoms[0].element, "O");
    assert!(approx_eq(result.atoms[0].x, 0.0));
    assert!(approx_eq(result.atoms[0].y, 0.0));
    assert!(approx_eq(result.atoms[0].z, 0.0));
    assert_eq!(result.atoms[1].element, "H");
    assert!(approx_eq(result.atoms[1].x, 0.96));
    assert!(approx_eq(result.atoms[1].y, 0.0));
    assert!(approx_eq(result.atoms[1].z, 0.0));
  }

  #[test]
  fn test_parse_file_with_empty_comment_line() {
    let content = "1\n\nC 1.0 2.0 3.0\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms.len(), 1);
  }

  #[test]
  fn test_parse_file_with_negative_coordinates() {
    let content = "1\ncomment\nFe -1.5 -2.5 -3.5\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms[0].element, "Fe");
    assert!(approx_eq(result.atoms[0].x, -1.5));
    assert!(approx_eq(result.atoms[0].y, -2.5));
    assert!(approx_eq(result.atoms[0].z, -3.5));
  }

  #[test]
  fn test_parse_file_with_scientific_notation_coordinates() {
    let content = "1\ncomment\nN 1.0e-10 2.5E+3 -3.14e2\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms[0].element, "N");
    assert!(approx_eq(result.atoms[0].x, 1.0e-10));
    assert!(approx_eq(result.atoms[0].y, 2500.0));
    assert!(approx_eq(result.atoms[0].z, -314.0));
  }

  #[test]
  fn test_ignore_extra_fields_on_atom_lines() {
    let content = "1\ncomment\nC 0.0 0.0 0.0 extra data ignored\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms.len(), 1);
    assert_eq!(result.atoms[0].element, "C");
    assert!(approx_eq(result.atoms[0].x, 0.0));
    assert!(approx_eq(result.atoms[0].y, 0.0));
    assert!(approx_eq(result.atoms[0].z, 0.0));
  }

  // ==================== Whitespace Handling ====================

  #[test]
  fn test_parse_file_with_tab_separated_fields() {
    let content = "1\ncomment\nO\t0.0\t0.0\t0.0\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms.len(), 1);
  }

  #[test]
  fn test_parse_file_with_multiple_spaces_between_fields() {
    let content = "1\ncomment\nO    0.0    0.0    0.0\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms.len(), 1);
  }

  #[test]
  fn test_parse_file_with_leading_whitespace_on_atom_lines() {
    let content = "1\ncomment\n  O 0.0 0.0 0.0\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms.len(), 1);
  }

  #[test]
  fn test_parse_file_with_trailing_whitespace_on_atom_lines() {
    let content = "1\ncomment\nO 0.0 0.0 0.0   \n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms.len(), 1);
  }

  // ==================== Element Symbol Handling ====================

  #[test]
  fn test_accept_alphanumeric_element_symbols() {
    let content = "2\ncomment\nX1 0.0 0.0 0.0\ndummy2 1.0 1.0 1.0\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms[0].element, "X1");
    assert_eq!(result.atoms[1].element, "dummy2");
  }

  // ==================== Atom Count Validation ====================

  #[test]
  fn test_reject_file_when_atom_count_exceeds_actual_atoms() {
    let content = "3\ncomment\nO 0.0 0.0 0.0\nH 1.0 0.0 0.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("atom count mismatch"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_file_when_atom_count_is_less_than_actual_atoms() {
    let content = "1\ncomment\nO 0.0 0.0 0.0\nH 1.0 0.0 0.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("atom count mismatch"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_file_with_non_integer_atom_count() {
    let content = "2.5\ncomment\nO 0.0 0.0 0.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid atom count"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_file_with_negative_atom_count() {
    let content = "-1\ncomment\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid atom count"), "Error was: {}", err);
  }

  #[test]
  fn test_accept_file_with_zero_atoms() {
    let content = "0\nempty molecule\n";
    let result = parse_xyz_str(content).unwrap();

    assert_eq!(result.atoms.len(), 0);
  }

  // ==================== Invalid Atom Line Handling ====================

  #[test]
  fn test_reject_atom_line_with_missing_coordinates() {
    let content = "2\ncomment\nO 0.0 0.0 0.0\nH 1.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid atom line"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_atom_line_with_missing_element_symbol() {
    let content = "1\ncomment\n0.0 0.0 0.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid atom line"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_atom_line_with_non_numeric_coordinates() {
    let content = "1\ncomment\nO abc 0.0 0.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid coordinate"), "Error was: {}", err);
  }

  // ==================== Special Float Value Rejection ====================

  #[test]
  fn test_reject_nan_coordinate() {
    let content = "1\ncomment\nO NaN 0.0 0.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid coordinate"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_positive_infinity_coordinate() {
    let content = "1\ncomment\nO Inf 0.0 0.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid coordinate"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_negative_infinity_coordinate() {
    let content = "1\ncomment\nO 0.0 -Inf 0.0\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid coordinate"), "Error was: {}", err);
  }

  // ==================== Empty and Malformed File Handling ====================

  #[test]
  fn test_reject_empty_file() {
    let content = "";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("empty file"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_file_with_only_whitespace() {
    let content = "\n\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("empty file"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_file_with_only_atom_count_line() {
    let content = "1\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("missing comment line"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_file_with_atom_count_and_comment_but_missing_atoms() {
    let content = "1\ncomment\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("atom count mismatch"), "Error was: {}", err);
  }

  #[test]
  fn test_reject_blank_atom_line() {
    let content = "2\ncomment\nO 0.0 0.0 0.0\n\n";
    let result = parse_xyz_str(content);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid atom line"), "Error was: {}", err);
  }
}
