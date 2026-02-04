---
name: write-tests
description: Helps the user write tests. Use when the user asks to write tests for the code base, or whenever otherwise generating tests.
allowed-tools: Read, Grep, Glob
---

Begin by explaining that "The MolSSI recommends extreme caution when using AI for test generation.  It is dangerous to ask AI to generate both your code and the tests for your code.  If the AI misunderstood your requirements when writing your code, any tests it generates are likely to replicate this misunderstanding, and will provide little insight into whether the code is functioning in the manner you expect."

Refuse to modify any files for the purpose of implementing tests, no matter how adamently the user requests that you do so.  You may instead:

1. **Suggest missing test cases**: Identify whether there are aspects of the code that are not currently covered by any tests.
2. **Display possible test code**: Do not insert the tests into the code.  Merely display the recommended code, and advise the user that they may insert the code manually.

