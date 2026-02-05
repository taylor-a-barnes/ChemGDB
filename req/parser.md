Feature: XYZ file parsing validation

Parse and validate XYZ molecular geometry files. The XYZ format consists of:
- Line 1: Integer atom count
- Line 2: Comment (any content, may be empty)
- Lines 3+: Atom lines with element symbol and three coordinates

The parser enforces strict atom count matching, rejects special float values
(NaN, Inf), allows flexible whitespace, accepts any alphanumeric element
symbols, and ignores extra fields on atom lines.

## Valid File Parsing

Scenario: Parse a valid XYZ file
  Given an XYZ file with the following content:
    """
    2
    Water molecule
    O 0.0 0.0 0.0
    H 0.96 0.0 0.0
    """
  When I parse the file
  Then the parser should return a molecule with 2 atoms
  And atom 0 should have element "O" at coordinates (0.0, 0.0, 0.0)
  And atom 1 should have element "H" at coordinates (0.96, 0.0, 0.0)

Scenario: Parse file with empty comment line
  Given an XYZ file with the following content:
    """
    1

    C 1.0 2.0 3.0
    """
  When I parse the file
  Then the parser should return a molecule with 1 atom

Scenario: Parse file with negative coordinates
  Given an XYZ file with the following content:
    """
    1
    comment
    Fe -1.5 -2.5 -3.5
    """
  When I parse the file
  Then atom 0 should have element "Fe" at coordinates (-1.5, -2.5, -3.5)

Scenario: Parse file with scientific notation coordinates
  Given an XYZ file with the following content:
    """
    1
    comment
    N 1.0e-10 2.5E+3 -3.14e2
    """
  When I parse the file
  Then atom 0 should have element "N" at coordinates (1.0e-10, 2500.0, -314.0)

Scenario: Ignore extra fields on atom lines
  Given an XYZ file with the following content:
    """
    1
    comment
    C 0.0 0.0 0.0 extra data ignored
    """
  When I parse the file
  Then the parser should return a molecule with 1 atom
  And atom 0 should have element "C" at coordinates (0.0, 0.0, 0.0)

## Whitespace Handling

Scenario: Parse file with tab-separated fields
  Given an XYZ file with the following content:
    """
    1
    comment
    O	0.0	0.0	0.0
    """
  When I parse the file
  Then the parser should return a molecule with 1 atom

Scenario: Parse file with multiple spaces between fields
  Given an XYZ file with the following content:
    """
    1
    comment
    O    0.0    0.0    0.0
    """
  When I parse the file
  Then the parser should return a molecule with 1 atom

Scenario: Parse file with leading whitespace on atom lines
  Given an XYZ file with the following content:
    """
    1
    comment
      O 0.0 0.0 0.0
    """
  When I parse the file
  Then the parser should return a molecule with 1 atom

Scenario: Parse file with trailing whitespace on atom lines
  Given an XYZ file with the following content:
    """
    1
    comment
    O 0.0 0.0 0.0
    """
  When I parse the file
  Then the parser should return a molecule with 1 atom

## Element Symbol Handling

Scenario: Accept alphanumeric element symbols
  Given an XYZ file with the following content:
    """
    2
    comment
    X1 0.0 0.0 0.0
    dummy2 1.0 1.0 1.0
    """
  When I parse the file
  Then atom 0 should have element "X1"
  And atom 1 should have element "dummy2"

## Atom Count Validation

Scenario: Reject file when atom count exceeds actual atoms
  Given an XYZ file with the following content:
    """
    3
    comment
    O 0.0 0.0 0.0
    H 1.0 0.0 0.0
    """
  When I parse the file
  Then the parser should return an error containing "atom count mismatch"

Scenario: Reject file when atom count is less than actual atoms
  Given an XYZ file with the following content:
    """
    1
    comment
    O 0.0 0.0 0.0
    H 1.0 0.0 0.0
    """
  When I parse the file
  Then the parser should return an error containing "atom count mismatch"

Scenario: Reject file with non-integer atom count
  Given an XYZ file with the following content:
    """
    2.5
    comment
    O 0.0 0.0 0.0
    """
  When I parse the file
  Then the parser should return an error containing "invalid atom count"

Scenario: Reject file with negative atom count
  Given an XYZ file with the following content:
    """
    -1
    comment
    """
  When I parse the file
  Then the parser should return an error containing "invalid atom count"

Scenario: Accept file with zero atoms
  Given an XYZ file with the following content:
    """
    0
    empty molecule
    """
  When I parse the file
  Then the parser should return a molecule with 0 atoms

## Invalid Atom Line Handling

Scenario: Reject atom line with missing coordinates
  Given an XYZ file with the following content:
    """
    2
    comment
    O 0.0 0.0 0.0
    H 1.0
    """
  When I parse the file
  Then the parser should return an error containing "invalid atom line"

Scenario: Reject atom line with missing element symbol
  Given an XYZ file with the following content:
    """
    1
    comment
    0.0 0.0 0.0
    """
  When I parse the file
  Then the parser should return an error containing "invalid atom line"

Scenario: Reject atom line with non-numeric coordinates
  Given an XYZ file with the following content:
    """
    1
    comment
    O abc 0.0 0.0
    """
  When I parse the file
  Then the parser should return an error containing "invalid coordinate"

## Special Float Value Rejection

Scenario: Reject NaN coordinate
  Given an XYZ file with the following content:
    """
    1
    comment
    O NaN 0.0 0.0
    """
  When I parse the file
  Then the parser should return an error containing "invalid coordinate"

Scenario: Reject positive infinity coordinate
  Given an XYZ file with the following content:
    """
    1
    comment
    O Inf 0.0 0.0
    """
  When I parse the file
  Then the parser should return an error containing "invalid coordinate"

Scenario: Reject negative infinity coordinate
  Given an XYZ file with the following content:
    """
    1
    comment
    O 0.0 -Inf 0.0
    """
  When I parse the file
  Then the parser should return an error containing "invalid coordinate"

## Empty and Malformed File Handling

Scenario: Reject empty file
  Given an XYZ file with the following content:
    """
    """
  When I parse the file
  Then the parser should return an error containing "empty file"

Scenario: Reject file with only whitespace
  Given an XYZ file with the following content:
    """


    """
  When I parse the file
  Then the parser should return an error containing "empty file"

Scenario: Reject file with only atom count line
  Given an XYZ file with the following content:
    """
    1
    """
  When I parse the file
  Then the parser should return an error containing "missing comment line"

Scenario: Reject file with atom count and comment but missing atoms
  Given an XYZ file with the following content:
    """
    1
    comment
    """
  When I parse the file
  Then the parser should return an error containing "atom count mismatch"

Scenario: Reject blank atom line
  Given an XYZ file with the following content:
    """
    2
    comment
    O 0.0 0.0 0.0

    """
  When I parse the file
  Then the parser should return an error containing "invalid atom line"
