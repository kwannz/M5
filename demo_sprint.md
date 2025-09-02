# Demo Sprint: Add Calculator Module

## Overview
Create a simple calculator module for the DeskAgent project with basic arithmetic operations.

## Goals
- Implement a Calculator struct with add, subtract, multiply, and divide operations
- Include comprehensive error handling for division by zero
- Add unit tests with >90% coverage
- Document the public API

## Requirements
1. **Core Calculator** (src/calculator/mod.rs)
   - Add, subtract, multiply, divide functions
   - Result<f64, CalculatorError> return type
   - Input validation

2. **Error Handling** (src/calculator/error.rs)
   - CalculatorError enum
   - Division by zero detection
   - Invalid input handling

3. **Tests** (tests/calculator_tests.rs)
   - Test all operations
   - Test error conditions
   - Integration tests

## Acceptance Criteria
- [x] All operations work correctly
- [x] Error handling prevents panics
- [x] Tests pass with >90% coverage
- [x] Code follows Rust best practices

## Timeline
Estimated: 2 hours
Priority: Medium