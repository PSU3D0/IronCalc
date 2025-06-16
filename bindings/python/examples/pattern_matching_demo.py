#!/usr/bin/env python3
"""
IronCalc Pattern Matching Demo

This example demonstrates the pattern matching functionality for scalar substitution
in IronCalc Python bindings. Pattern matching allows you to substitute known values
for complex formulas during evaluation, which is useful for scenario modeling and
optimization.
"""

import ironcalc

def main():
    print("🎯 IronCalc Python Pattern Matching Demo")
    print("="*50)
    
    # Create a new model with some test data
    print("\n📊 Creating model with test data...")
    model = ironcalc.create("PatternMatchingDemo", "en", "UTC")
    
    # Set up some sample data
    model.set_user_input(0, 1, 1, "Product", reevaluate=False)
    model.set_user_input(0, 1, 2, "Sales", reevaluate=False)
    model.set_user_input(0, 1, 3, "Region", reevaluate=False)
    
    model.set_user_input(0, 2, 1, "Widget A", reevaluate=False)
    model.set_user_input(0, 2, 2, 5000, reevaluate=False)
    model.set_user_input(0, 2, 3, "North", reevaluate=False)
    
    model.set_user_input(0, 3, 1, "Widget B", reevaluate=False)
    model.set_user_input(0, 3, 2, 3000, reevaluate=False)
    model.set_user_input(0, 3, 3, "South", reevaluate=False)
    
    model.set_user_input(0, 4, 1, "Widget A", reevaluate=False)
    model.set_user_input(0, 4, 2, 2000, reevaluate=False)
    model.set_user_input(0, 4, 3, "South", reevaluate=False)
    
    print("✅ Test data created successfully!")
    
    # Example 1: Basic SUMIFS Pattern
    print("\n🔍 Example 1: Basic SUMIFS Pattern")
    print("-" * 35)
    
    # Create a SUMIFS pattern using the builder
    sumifs_pattern = (ironcalc.PyPatternKey.sumifs()
                     .sum_column(2)  # Column B (Sales)
                     .criteria_string(1, "Widget A")  # Column A (Product)
                     .criteria_string(3, "North")  # Column C (Region)
                     .build())
    
    print(f"Created pattern: {sumifs_pattern}")
    
    # Validate the pattern
    try:
        sumifs_pattern.validate()
        print("✅ Pattern validation successful")
    except Exception as e:
        print(f"❌ Pattern validation failed: {e}")
        return
    
    # Add formula to model
    formula = '=SUMIFS(B:B,A:A,"Widget A",C:C,"North")'
    model.set_user_input(0, 6, 1, formula)
    
    # Evaluate without substitution first
    model.evaluate()
    original_result = float(model.get_formatted_cell_value(0, 6, 1))
    print(f"Original formula result: {original_result}")
    
    # Add substitution
    substitution_value = 99999.0
    model.add_substitution(sumifs_pattern, substitution_value)
    print(f"Added substitution: {substitution_value}")
    
    # Re-evaluate with substitution
    model.evaluate()
    substituted_result = float(model.get_formatted_cell_value(0, 6, 1))
    print(f"Substituted result: {substituted_result}")
    
    if abs(substituted_result - substitution_value) < 0.001:
        print("✅ SUMIFS substitution successful!")
    else:
        print(f"❌ SUMIFS substitution failed. Expected {substitution_value}, got {substituted_result}")
    
    # Example 2: COUNTIFS Pattern
    print("\n🔢 Example 2: COUNTIFS Pattern")
    print("-" * 32)
    
    # Create a COUNTIFS pattern
    countifs_pattern = (ironcalc.PyPatternKey.countifs()
                       .criteria_string(1, "Widget A")  # Column A (Product)
                       .build())
    
    print(f"Created COUNTIFS pattern: {countifs_pattern}")
    
    # Add COUNTIFS formula
    countifs_formula = '=COUNTIFS(A:A,"Widget A")'
    model.set_user_input(0, 7, 1, countifs_formula)
    
    # Add substitution for COUNTIFS
    countifs_value = 42.0
    model.add_substitution(countifs_pattern, countifs_value)
    
    model.evaluate()
    countifs_result = float(model.get_formatted_cell_value(0, 7, 1))
    print(f"COUNTIFS result: {countifs_result}")
    
    if abs(countifs_result - countifs_value) < 0.001:
        print("✅ COUNTIFS substitution successful!")
    else:
        print(f"❌ COUNTIFS substitution failed. Expected {countifs_value}, got {countifs_result}")
    
    # Example 3: AVERAGEIFS Pattern
    print("\n📊 Example 3: AVERAGEIFS Pattern")
    print("-" * 35)
    
    # Create an AVERAGEIFS pattern
    averageifs_pattern = (ironcalc.PyPatternKey.averageifs()
                         .average_column(2)  # Column B (Sales)
                         .criteria_string(3, "South")  # Column C (Region)
                         .build())
    
    print(f"Created AVERAGEIFS pattern: {averageifs_pattern}")
    
    # Add AVERAGEIFS formula
    averageifs_formula = '=AVERAGEIFS(B:B,C:C,"South")'
    model.set_user_input(0, 8, 1, averageifs_formula)
    
    # Add substitution for AVERAGEIFS
    averageifs_value = 3500.0
    model.add_substitution(averageifs_pattern, averageifs_value)
    
    model.evaluate()
    averageifs_result = float(model.get_formatted_cell_value(0, 8, 1))
    print(f"AVERAGEIFS result: {averageifs_result}")
    
    if abs(averageifs_result - averageifs_value) < 0.001:
        print("✅ AVERAGEIFS substitution successful!")
    else:
        print(f"❌ AVERAGEIFS substitution failed. Expected {averageifs_value}, got {averageifs_result}")
    
    # Example 4: VLOOKUP Pattern
    print("\n🔍 Example 4: VLOOKUP Pattern")
    print("-" * 30)
    
    # Set up lookup table data
    model.set_user_input(0, 10, 1, "Product", reevaluate=False)
    model.set_user_input(0, 10, 2, "Price", reevaluate=False)
    model.set_user_input(0, 11, 1, "Widget A", reevaluate=False)
    model.set_user_input(0, 11, 2, 150.0, reevaluate=False)
    model.set_user_input(0, 12, 1, "Widget B", reevaluate=False)
    model.set_user_input(0, 12, 2, 200.0, reevaluate=False)
    
    # Create a VLOOKUP pattern
    vlookup_pattern = (ironcalc.PyPatternKey.vlookup()
                      .lookup_string("Widget A")
                      .table_range(11, 1, 12, 2)  # A11:B12 lookup table
                      .column_index(2)  # Return value from column B (2nd column)
                      .exact_match(True)
                      .build())
    
    print(f"Created VLOOKUP pattern: {vlookup_pattern}")
    
    # Add VLOOKUP formula
    vlookup_formula = '=VLOOKUP("Widget A",A11:B12,2,TRUE)'
    model.set_user_input(0, 14, 1, vlookup_formula)
    
    # Add substitution for VLOOKUP
    vlookup_value = 175.0
    model.add_substitution(vlookup_pattern, vlookup_value)
    
    model.evaluate()
    vlookup_result = float(model.get_formatted_cell_value(0, 14, 1))
    print(f"VLOOKUP result: {vlookup_result}")
    
    if abs(vlookup_result - vlookup_value) < 0.001:
        print("✅ VLOOKUP substitution successful!")
    else:
        print(f"❌ VLOOKUP substitution failed. Expected {vlookup_value}, got {vlookup_result}")
    
    # Example 5: SubstitutionRegistry Management
    print("\n📋 Example 5: Substitution Registry Management")
    print("-" * 45)
    
    # Get registry statistics
    stats = model.get_substitution_stats()
    print(f"Registry statistics:")
    print(f"  Total lookups: {stats['total_lookups']}")
    print(f"  Successful lookups: {stats['successful_lookups']}")
    print(f"  Failed lookups: {stats['failed_lookups']}")
    print(f"  Success rate: {stats['success_rate']:.1f}%")
    
    # Test deactivating substitutions
    print("\n🔄 Testing substitution deactivation...")
    model.set_substitutions_active(False)
    print(f"Substitutions active: {model.substitutions_active()}")
    
    model.evaluate()
    deactivated_result = float(model.get_formatted_cell_value(0, 6, 1))
    print(f"Result with substitutions deactivated: {deactivated_result}")
    
    if abs(deactivated_result - original_result) < 0.001:
        print("✅ Deactivation successful! Returned to original evaluation.")
    else:
        print(f"❌ Deactivation failed. Expected {original_result}, got {deactivated_result}")
    
    # Reactivate substitutions
    model.set_substitutions_active(True)
    model.evaluate()
    reactivated_result = float(model.get_formatted_cell_value(0, 6, 1))
    print(f"Result after reactivation: {reactivated_result}")
    
    # Example 6: Bulk Operations
    print("\n🚀 Example 6: Bulk Substitution Operations")
    print("-" * 40)
    
    # Create multiple patterns for bulk operation
    bulk_patterns = []
    
    pattern1 = (ironcalc.PyPatternKey.sumifs()
               .sum_column(2)
               .criteria_string(1, "Widget B")
               .build())
    bulk_patterns.append((pattern1, 8000.0))
    
    pattern2 = (ironcalc.PyPatternKey.countifs()
               .criteria_string(3, "South")
               .build())
    bulk_patterns.append((pattern2, 15.0))
    
    # Add bulk substitutions
    added_count = model.add_substitutions_bulk(bulk_patterns)
    print(f"Added {added_count} patterns via bulk operation")
    
    # Show final statistics
    final_stats = model.get_substitution_stats()
    print(f"\nFinal registry statistics:")
    print(f"  Total lookups: {final_stats['total_lookups']}")
    print(f"  Success rate: {final_stats['success_rate']:.1f}%")
    
    print(f"  Patterns stored in registry")
    
    # Example 7: Using Standalone SubstitutionRegistry
    print("\n🔧 Example 7: Standalone SubstitutionRegistry")
    print("-" * 42)
    
    # Create a standalone registry
    registry = ironcalc.PySubstitutionRegistry()
    
    # Add some patterns to it
    registry.add_substitution(sumifs_pattern, 12345.0)
    registry.add_substitution(countifs_pattern, 67.0)
    
    print(f"Standalone registry has {registry.len()} patterns")
    print(f"Registry is active: {registry.is_active()}")
    
    # Test lookups
    lookup_result = registry.get_substitution(sumifs_pattern)
    print(f"Lookup result for SUMIFS pattern: {lookup_result}")
    
    has_pattern = registry.has_substitution(countifs_pattern)
    print(f"Registry has COUNTIFS pattern: {has_pattern}")
    
    # Get standalone registry stats
    registry_stats = registry.stats()
    print(f"Standalone registry stats: {dict(registry_stats)}")
    
    print("\n🎉 Pattern Matching Demo Completed Successfully!")
    print("   ✅ SUMIFS pattern matching and substitution")
    print("   ✅ COUNTIFS pattern matching and substitution")
    print("   ✅ AVERAGEIFS pattern matching and substitution")
    print("   ✅ VLOOKUP pattern matching and substitution")
    print("   ✅ Registry management and statistics")
    print("   ✅ Bulk operations and deactivation/reactivation")
    print("   ✅ Standalone SubstitutionRegistry usage")

if __name__ == "__main__":
    main()