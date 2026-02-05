Feature: XYZ file parsing validation

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
