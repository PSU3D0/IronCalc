# Pattern Matching in IronCalc Python Bindings

IronCalc's Python bindings include comprehensive support for pattern matching, which allows you to substitute known values for complex formulas during evaluation. This is particularly useful for scenario modeling, optimization, and "what-if" analysis.

## Overview

Pattern matching works by identifying specific formula patterns (like SUMIFS, COUNTIFS, AVERAGEIFS, and VLOOKUP) and replacing their evaluation results with predetermined values. This enables rapid scenario testing without modifying the underlying spreadsheet formulas.

## Supported Functions

The pattern matching system supports the following Excel functions:

- **SUMIFS**: Sum values based on multiple criteria
- **COUNTIFS**: Count values based on multiple criteria
- **AVERAGEIFS**: Average values based on multiple criteria
- **VLOOKUP**: Lookup values in a table

## Basic Usage

### 1. Creating Patterns

Patterns are created using builder methods for a fluent interface:

```python
import ironcalc

# Create a SUMIFS pattern
sumifs_pattern = (ironcalc.PyPatternKey.sumifs()
                 .sum_column(2)  # Column B
                 .criteria_string(1, "Product A")  # Column A = "Product A"
                 .criteria_string(3, "North")  # Column C = "North"
                 .build())

# Create a COUNTIFS pattern
countifs_pattern = (ironcalc.PyPatternKey.countifs()
                   .criteria_string(1, "Active")  # Column A = "Active"
                   .criteria_number(2, 100.0)  # Column B = 100
                   .build())

# Create an AVERAGEIFS pattern
averageifs_pattern = (ironcalc.PyPatternKey.averageifs()
                     .average_column(3)  # Column C
                     .criteria_string(1, "Region")  # Column A = "Region"
                     .build())

# Create a VLOOKUP pattern
vlookup_pattern = (ironcalc.PyPatternKey.vlookup()
                  .lookup_string("Widget A")
                  .table_range(1, 1, 100, 4)  # A1:D100
                  .column_index(3)  # Return column C
                  .exact_match(True)
                  .build())
```

### 2. Adding Substitutions to Models

```python
# Create a model
model = ironcalc.create("MyModel", "en", "UTC")

# Add some formulas
model.set_user_input(0, 5, 1, '=SUMIFS(B:B,A:A,"Product A",C:C,"North")')

# Add substitution
model.add_substitution(sumifs_pattern, 50000.0)

# Evaluate with substitution
model.evaluate()
result = model.get_formatted_cell_value(0, 5, 1)
print(f"Result: {result}")  # Should output: 50000
```

### 3. Managing Substitutions

```python
# Check if a substitution exists
has_substitution = model.has_substitution(pattern)

# Get substitution value
value = model.get_substitution(pattern)

# Deactivate all substitutions
model.set_substitutions_active(False)

# Reactivate substitutions
model.set_substitutions_active(True)

# Clear all substitutions
model.clear_substitutions()

# Add multiple substitutions at once
bulk_patterns = [
    (pattern1, 1000.0),
    (pattern2, 2000.0)
]
model.add_substitutions_bulk(bulk_patterns)
```

### 4. Statistics and Monitoring

```python
# Get registry statistics
stats = model.get_substitution_stats()
print(f"Total lookups: {stats['total_lookups']}")
print(f"Success rate: {stats['success_rate']:.1f}%")
```

## Pattern Types

### SUMIFS Pattern

For formulas like `=SUMIFS(sum_range, criteria_range1, criteria1, criteria_range2, criteria2, ...)`

```python
pattern = (ironcalc.PyPatternKey.sumifs()
          .sheet("SheetName")  # Optional sheet name
          .sum_column(5)  # Column to sum (required)
          .criteria_string(1, "Value")  # String criteria
          .criteria_number(2, 100.0)  # Number criteria
          .criteria_cell(3, 0, 10, 5)  # Cell reference criteria (sheet, row, col)
          .build())
```

### COUNTIFS Pattern

For formulas like `=COUNTIFS(criteria_range1, criteria1, criteria_range2, criteria2, ...)`

```python
pattern = (ironcalc.PyPatternKey.countifs()
          .sheet("SheetName")  # Optional sheet name
          .criteria_string(1, "Active")
          .criteria_number(2, 50.0)
          .build())
```

### AVERAGEIFS Pattern

For formulas like `=AVERAGEIFS(average_range, criteria_range1, criteria1, criteria_range2, criteria2, ...)`

```python
pattern = (ironcalc.PyPatternKey.averageifs()
          .sheet("SheetName")  # Optional sheet name
          .average_column(3)  # Column to average (required)
          .criteria_string(1, "Region")
          .criteria_number(4, 2023.0)
          .build())
```

### VLOOKUP Pattern

For formulas like `=VLOOKUP(lookup_value, table_array, col_index_num, [range_lookup])`

```python
pattern = (ironcalc.PyPatternKey.vlookup()
          .sheet("SheetName")  # Optional sheet name
          .lookup_string("SearchValue")  # Or .lookup_number() or .lookup_cell()
          .table_range(1, 1, 100, 5)  # Table range (start_row, start_col, end_row, end_col)
          # Or .table_range_with_sheet("Sheet", 1, 1, 100, 5)
          .column_index(3)  # Column to return (required)
          .exact_match(True)  # True for exact match, False for approximate (required)
          .build())
```

## Advanced Features

### Standalone Substitution Registry

You can also work with a standalone substitution registry:

```python
registry = ironcalc.PySubstitutionRegistry()
registry.add_substitution(pattern, 1000.0)

# Check if pattern exists
if registry.has_substitution(pattern):
    value = registry.get_substitution(pattern)
    print(f"Pattern value: {value}")

# Get statistics
stats = registry.stats()
```

### Pattern Validation

All patterns can be validated before use:

```python
try:
    pattern.validate()
    print("Pattern is valid")
except ValueError as e:
    print(f"Pattern validation failed: {e}")
```

### Criteria Value Types

Patterns support three types of criteria values:

```python
# String criteria
string_criteria = ironcalc.PyCriteriaValue.string("Text")

# Number criteria
number_criteria = ironcalc.PyCriteriaValue.number(123.45)

# Cell reference criteria
cell_criteria = ironcalc.PyCriteriaValue.cell_ref(sheet=0, row=1, column=2)
```

## Complete Example

```python
import ironcalc

def pattern_matching_example():
    # Create model and add data
    model = ironcalc.create("PatternExample", "en", "UTC")
    
    # Set up test data
    model.set_user_input(0, 1, 1, "Product")
    model.set_user_input(0, 1, 2, "Sales")
    model.set_user_input(0, 1, 3, "Region")
    
    model.set_user_input(0, 2, 1, "Widget A")
    model.set_user_input(0, 2, 2, 1000)
    model.set_user_input(0, 2, 3, "North")
    
    model.set_user_input(0, 3, 1, "Widget A")
    model.set_user_input(0, 3, 2, 2000)
    model.set_user_input(0, 3, 3, "South")
    
    # Add formula
    model.set_user_input(0, 5, 1, '=SUMIFS(B:B,A:A,"Widget A",C:C,"North")')
    
    # Get original result
    model.evaluate()
    original = float(model.get_formatted_cell_value(0, 5, 1))
    print(f"Original result: {original}")
    
    # Create pattern and substitute
    pattern = (ironcalc.PyPatternKey.sumifs()
              .sum_column(2)
              .criteria_string(1, "Widget A")
              .criteria_string(3, "North")
              .build())
    
    model.add_substitution(pattern, 99999.0)
    
    # Get substituted result
    model.evaluate()
    substituted = float(model.get_formatted_cell_value(0, 5, 1))
    print(f"Substituted result: {substituted}")
    
    # Get statistics
    stats = model.get_substitution_stats()
    print(f"Pattern matching success rate: {stats['success_rate']:.1f}%")

if __name__ == "__main__":
    pattern_matching_example()
```

## Error Handling

Pattern creation and usage can raise `ValueError` exceptions:

```python
try:
    # This will fail - missing required sum_column
    pattern = ironcalc.PyPatternKey.sumifs().criteria_string(1, "Test").build()
except ValueError as e:
    print(f"Pattern creation failed: {e}")

try:
    # This will fail - invalid pattern structure
    pattern.validate()
except ValueError as e:
    print(f"Pattern validation failed: {e}")
```

## Performance Considerations

- Pattern matching is designed for high-performance lookup with O(1) hash-based access
- Substitutions are applied during formula evaluation, not during parsing
- Registry statistics help monitor performance and hit rates
- Deactivating substitutions allows easy comparison between normal and substituted evaluations

## Best Practices

1. **Validate patterns** after creation to catch errors early
2. **Use specific criteria** to ensure patterns match intended formulas
3. **Monitor statistics** to verify patterns are matching as expected
4. **Test deactivation** to ensure substitutions can be turned off
5. **Use bulk operations** when adding many substitutions at once
6. **Clear substitutions** when no longer needed to free memory

For more examples, see the `examples/` directory in the Python bindings.