#!/usr/bin/env python3
"""
Simple IronCalc Pattern Matching Example

This is a quick example showing basic pattern matching functionality.
"""

import ironcalc

def main():
    print("Simple Pattern Matching Example")
    print("="*35)
    
    # Create a new spreadsheet
    model = ironcalc.create("SimpleExample", "en", "UTC")
    
    # Add some test data
    model.set_user_input(0, 1, 1, "Product")
    model.set_user_input(0, 1, 2, "Sales") 
    model.set_user_input(0, 2, 1, "Widget A")
    model.set_user_input(0, 2, 2, 1000)
    model.set_user_input(0, 3, 1, "Widget A")
    model.set_user_input(0, 3, 2, 2000)
    
    # Add a SUMIFS formula
    model.set_user_input(0, 5, 1, '=SUMIFS(B:B,A:A,"Widget A")')
    
    # Evaluate normally first
    model.evaluate()
    normal_result = model.get_formatted_cell_value(0, 5, 1)
    print(f"Normal SUMIFS result: {normal_result}")
    
    # Create a pattern to match this formula
    pattern = (ironcalc.PyPatternKey.sumifs()
              .sum_column(2)  # Column B
              .criteria_string(1, "Widget A")  # Column A = "Widget A"
              .build())
    
    # Add a substitution - replace the formula result with 99999
    model.add_substitution(pattern, 99999.0)
    
    # Evaluate with pattern matching
    model.evaluate()
    substituted_result = model.get_formatted_cell_value(0, 5, 1)
    print(f"Substituted result: {substituted_result}")
    
    if substituted_result == "99999":
        print("✅ Pattern matching worked!")
    else:
        print("❌ Pattern matching failed")

if __name__ == "__main__":
    main()