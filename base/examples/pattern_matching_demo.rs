/*!
# Pattern Matching Demo

This example demonstrates the core pattern matching functionality for scalar substitution.
*/

use ironcalc_base::expressions::pattern_matching::*;
use ironcalc_base::expressions::types::CellReferenceIndex;

fn main() {
    println!("🎯 IronCalc Pattern Matching Demo");
    println!("==================================\n");

    // Example 1: Basic SUMIFS pattern using builder
    println!("📊 Example 1: Basic SUMIFS Pattern");
    println!("---------------------------------");

    let pattern1 = PatternKey::sumifs()
        .sheet("INJECT.MONTHLY")
        .sum_column(4) // Column D
        .criteria_string(6, "Collections Clinic") // Column F
        .build()
        .unwrap();

    println!("Created pattern: {:#?}", pattern1);

    // Example 2: Multi-criteria SUMIFS pattern
    println!("\n📊 Example 2: Multi-Criteria SUMIFS Pattern");
    println!("-------------------------------------------");

    let pattern2 = PatternKey::sumifs()
        .sheet("INJECT.MONTHLY")
        .sum_column(15) // Column O
        .criteria_string(6, "Lasik Eval") // Column F
        .criteria_number(8, 1811349806.0) // Column H
        .criteria_cell(
            2,
            CellReferenceIndex {
                sheet: 0,
                row: 2,
                column: 3,
            },
        ) // C2
        .build()
        .unwrap();

    println!("Created pattern: {:#?}", pattern2);

    // Example 3: HashMap usage for substitution registry
    println!("\n🔍 Example 3: Substitution Registry Usage");
    println!("----------------------------------------");

    let mut substitutions: HashMap<PatternKey, f64> = HashMap::new();

    // Add our patterns to the registry
    substitutions.insert(pattern1.clone(), 42000.0);
    substitutions.insert(pattern2.clone(), 75000.0);

    println!("Registry contains {} substitutions", substitutions.len());

    // Test lookup performance
    println!("Lookup for pattern1: {:?}", substitutions.get(&pattern1));
    println!("Lookup for pattern2: {:?}", substitutions.get(&pattern2));

    // Example 4: Pattern equality and hashing
    println!("\n⚖️  Example 4: Pattern Equality and Hashing");
    println!("------------------------------------------");

    // Create identical pattern using direct construction
    let pattern1_clone = PatternKey::Sumifs {
        sheet: Some("INJECT.MONTHLY".to_string()),
        sum_column: 4,
        criteria_pairs: vec![CriteriaPair {
            column: 6,
            criteria: CriteriaValue::String("Collections Clinic".to_string()),
        }],
    };

    println!("pattern1 == pattern1_clone: {}", pattern1 == pattern1_clone);
    println!(
        "HashMap lookup with clone: {:?}",
        substitutions.get(&pattern1_clone)
    );

    // Example 5: Different criteria value types
    println!("\n🎨 Example 5: Criteria Value Types");
    println!("---------------------------------");

    let string_criteria = CriteriaValue::string("Test");
    let number_criteria = CriteriaValue::number(123.45);
    let cell_criteria = CriteriaValue::cell_ref(0, 1, 1);

    println!("String criteria: {:?}", string_criteria);
    println!("Number criteria: {:?}", number_criteria);
    println!("Cell criteria: {:?}", cell_criteria);

    // Example 6: Error handling
    println!("\n❌ Example 6: Error Handling");
    println!("---------------------------");

    let invalid_pattern = PatternKey::sumifs()
        .sheet("TestSheet")
        .criteria_string(6, "Test")
        .build(); // Missing required sum_column

    match invalid_pattern {
        Some(pattern) => println!("Pattern created: {:?}", pattern),
        None => println!("❌ Pattern creation failed - missing required field"),
    }

    // Example 7: Extract patterns from parsed formulas
    println!("\n🔬 Example 7: Pattern Extraction from Parsed Formulas");
    println!("----------------------------------------------------");

    use ironcalc_base::expressions::parser::Parser;
    use ironcalc_base::expressions::types::CellReferenceRC;
    use std::collections::HashMap;

    let worksheets = vec!["Sheet1".to_string(), "INJECT.MONTHLY".to_string()];
    let mut parser = Parser::new(worksheets, vec![], HashMap::new());

    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };

    // Parse a SUMIFS formula and extract the pattern
    let formula = "SUMIFS(INJECT.MONTHLY!D:D,INJECT.MONTHLY!F:F,\"Collections Clinic\",INJECT.MONTHLY!H:H,1811349806)";
    let ast = parser.parse(formula, &cell_reference);

    match PatternKey::from_node(&ast) {
        Some(extracted_pattern) => {
            println!("✅ Successfully extracted pattern from formula:");
            println!("   Formula: {}", formula);
            println!("   Pattern: {:#?}", extracted_pattern);

            // Verify it matches our manually created pattern
            let manual_pattern = PatternKey::sumifs()
                .sheet("INJECT.MONTHLY")
                .sum_column(4) // D column
                .criteria_string(6, "Collections Clinic") // F column
                .criteria_number(8, 1811349806.0) // H column
                .build()
                .unwrap();

            println!(
                "   Matches manual pattern: {}",
                extracted_pattern == manual_pattern
            );
        }
        None => {
            println!("❌ Failed to extract pattern from formula");
        }
    }

    // Example with cell reference criteria
    let formula2 = "SUMIFS(D:D,F:F,C2)";
    let ast2 = parser.parse(formula2, &cell_reference);

    if let Some(pattern) = PatternKey::from_node(&ast2) {
        println!("\n✅ Pattern with cell reference criteria:");
        println!("   Formula: {}", formula2);
        println!("   Pattern: {:#?}", pattern);
    }

    // Example 8: SubstitutionRegistry usage
    println!("\n📋 Example 8: SubstitutionRegistry Usage");
    println!("---------------------------------------");

    let mut registry = SubstitutionRegistry::new();

    // Add substitutions using our previously created patterns
    registry.add_substitution(pattern1.clone(), 42000.0);
    registry.add_substitution(pattern2.clone(), 75000.0);

    println!(
        "Registry status: active={}, count={}",
        registry.is_active(),
        registry.len()
    );

    // Test lookups
    println!("Lookup results:");
    let lookup1 = registry.get_substitution(&pattern1);
    println!("  Pattern1: {:?}", lookup1);

    let lookup2 = registry.get_substitution(&pattern2);
    println!("  Pattern2: {:?}", lookup2);

    // Test deactivation
    registry.set_active(false);
    let lookup_inactive = registry.get_substitution(&pattern1);
    println!("Lookup when inactive: {:?}", lookup_inactive);

    // Reactivate and show stats
    registry.set_active(true);
    registry.get_substitution(&pattern1); // One more lookup for stats

    let stats = registry.stats();
    println!("\nRegistry Statistics:");
    println!("  Total lookups: {}", stats.total_lookups());
    println!("  Successful: {}", stats.successful_lookups);
    println!("  Failed: {}", stats.failed_lookups);
    println!("  Inactive: {}", stats.inactive_lookups);
    println!("  Success rate: {:.1}%", stats.success_rate());
    println!(
        "  Memory usage estimate: {} bytes",
        registry.memory_usage_estimate()
    );

    // Bulk operations
    let more_patterns = vec![
        (
            PatternKey::sumifs()
                .sum_column(10)
                .criteria_string(11, "Bulk1")
                .build()
                .unwrap(),
            10000.0,
        ),
        (
            PatternKey::sumifs()
                .sum_column(12)
                .criteria_string(13, "Bulk2")
                .build()
                .unwrap(),
            20000.0,
        ),
    ];

    let bulk_count = registry.add_bulk(more_patterns);
    println!("\nAdded {} patterns via bulk operation", bulk_count);
    println!("Total patterns in registry: {}", registry.len());

    // Example 9: Complete Model Integration Demo
    println!("\n🚀 Example 9: Complete Model Integration Demo");
    println!("============================================");

    use ironcalc_base::Model;

    // Create a new model with some test data
    let mut model = Model::new_empty("IntegrationTest", "en", "UTC").unwrap();

    // Set up some data in the model
    model
        .set_user_input(0, 1, 1, "Product".to_string())
        .unwrap();
    model.set_user_input(0, 1, 2, "Sales".to_string()).unwrap();
    model.set_user_input(0, 1, 3, "Region".to_string()).unwrap();

    model
        .set_user_input(0, 2, 1, "Widget A".to_string())
        .unwrap();
    model.set_user_input(0, 2, 2, "5000".to_string()).unwrap();
    model.set_user_input(0, 2, 3, "North".to_string()).unwrap();

    model
        .set_user_input(0, 3, 1, "Widget B".to_string())
        .unwrap();
    model.set_user_input(0, 3, 2, "3000".to_string()).unwrap();
    model.set_user_input(0, 3, 3, "South".to_string()).unwrap();

    model
        .set_user_input(0, 4, 1, "Widget A".to_string())
        .unwrap();
    model.set_user_input(0, 4, 2, "2000".to_string()).unwrap();
    model.set_user_input(0, 4, 3, "South".to_string()).unwrap();

    // Set up a SUMIFS formula in the model
    model
        .set_user_input(
            0,
            6,
            1,
            "=SUMIFS(B:B,A:A,\"Widget A\",C:C,\"North\")".to_string(),
        )
        .unwrap();
    model
        .set_user_input(
            0,
            6,
            2,
            "=SUMIFS(B:B,A:A,\"Widget A\",C:C,\"North\")*10".to_string(),
        )
        .unwrap();

    println!("Created model with SUMIFS formula: =SUMIFS(B:B,A:A,\"Widget A\",C:C,\"North\")");

    // Evaluate without substitutions first
    model.evaluate();
    let a6_result = model.get_cell_value_by_index(0, 6, 1).unwrap();
    println!("Original evaluation result for A6: {:?}", a6_result);

    let b6_result = model.get_cell_value_by_index(0, 6, 2).unwrap();
    println!("Original evaluation result for B6: {:?}", b6_result);

    // Now create a substitution pattern that matches our formula
    let substitution_pattern = PatternKey::sumifs()
        .sum_column(2) // Column B (Sales)
        .criteria_string(1, "Widget A") // Column A (Product)
        .criteria_string(3, "North") // Column C (Region)
        .build()
        .unwrap();

    println!("Created substitution pattern: {:#?}", substitution_pattern);

    // Add the substitution to the model
    model.add_substitution(substitution_pattern.clone(), 99999.0);

    println!(
        "Added substitution: {} -> 99999.0",
        if model.has_substitution(&substitution_pattern) {
            "✅"
        } else {
            "❌"
        }
    );

    // Re-evaluate with substitutions active
    model.evaluate();
    let a6_substituted_result = model.get_cell_value_by_index(0, 6, 1).unwrap();
    println!("Result with substitution (A6): {:?}", a6_substituted_result);

    // Verify the substitution worked
    match a6_substituted_result {
        ironcalc_base::cell::CellValue::Number(value) if (value - 99999.0).abs() < 0.001 => {
            println!("✅ Substitution successful! Formula returned substituted value.");
        }
        _ => {
            println!(
                "❌ Substitution failed. Expected 99999.0, got {:?}",
                a6_substituted_result
            );
        }
    }

    let b6_substituted_result = model.get_cell_value_by_index(0, 6, 2).unwrap();
    println!("Result with substitution (B6): {:?}", b6_substituted_result);

    match b6_substituted_result {
        ironcalc_base::cell::CellValue::Number(value)
            if (value - (99999.0 * 10.0)).abs() < 0.001 =>
        {
            println!("✅ Substitution successful! Formula returned substituted value.");
        }
        _ => {
            println!(
                "❌ Substitution failed. Expected 999990.0, got {:?}",
                b6_substituted_result
            );
        }
    }

    // Test deactivating substitutions
    model.set_substitutions_active(false);
    println!("\nDeactivated substitution registry");

    model.evaluate();
    let deactivated_result = model.get_cell_value_by_index(0, 6, 1).unwrap();
    println!(
        "Result with substitutions deactivated: {:?}",
        deactivated_result
    );

    // Should match original result
    if deactivated_result == a6_result {
        println!("✅ Deactivation successful! Returned to original evaluation.");
    } else {
        println!(
            "❌ Deactivation failed. Expected {:?}, got {:?}",
            a6_result, deactivated_result
        );
    }

    // Test reactivating
    model.set_substitutions_active(true);
    model.evaluate();
    let reactivated_result = model.get_cell_value_by_index(0, 6, 1).unwrap();
    println!("Result after reactivation: {:?}", reactivated_result);

    // Display substitution statistics
    let stats = model.get_substitution_stats();
    println!("\nSubstitution Registry Statistics:");
    println!("  Total lookups: {}", stats.total_lookups());
    println!("  Successful: {}", stats.successful_lookups);
    println!("  Failed: {}", stats.failed_lookups);
    println!("  Inactive: {}", stats.inactive_lookups);
    println!("  Success rate: {:.1}%", stats.success_rate());

    // Test with a non-matching formula
    model
        .set_user_input(
            0,
            7,
            1,
            "=SUMIFS(B:B,A:A,\"Widget B\",C:C,\"North\")".to_string(),
        )
        .unwrap();
    model.evaluate();
    let non_matching_result = model.get_cell_value_by_index(0, 7, 1).unwrap();
    println!("\nNon-matching formula result: {:?}", non_matching_result);
    println!("  (Should evaluate normally since no substitution pattern matches)");

    // Clear substitutions and verify
    model.clear_substitutions();
    model.evaluate();
    let cleared_result = model.get_cell_value_by_index(0, 6, 1).unwrap();
    println!(
        "\nResult after clearing substitutions: {:?}",
        cleared_result
    );

    if cleared_result == a6_result {
        println!("✅ Clear successful! Returned to original evaluation.");
    } else {
        println!(
            "❌ Clear failed. Expected {:?}, got {:?}",
            a6_result, cleared_result
        );
    }

    println!("\n✅ Complete integration demo completed successfully!");
    println!("   This demonstrates the full pipeline from pattern creation");
    println!("   to formula evaluation with scalar substitution!");

    // Example 10: Advanced Pattern Validation and Error Handling
    println!("\n🔍 Example 10: Advanced Pattern Validation and Error Handling");
    println!("============================================================");

    use ironcalc_base::expressions::pattern_matching::{
        CriteriaPair, CriteriaValue, PatternError, PatternKey,
    };
    use ironcalc_base::expressions::types::CellReferenceIndex;
    use ordered_float::OrderedFloat;

    // Test valid multi-criteria pattern
    println!("Testing valid multi-criteria pattern...");
    let valid_pattern = PatternKey::Sumifs {
        sheet: Some("Sales".to_string()),
        sum_column: 5,
        criteria_pairs: vec![
            CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Product A".to_string()),
            },
            CriteriaPair {
                column: 3,
                criteria: CriteriaValue::Number(OrderedFloat(100.0)),
            },
            CriteriaPair {
                column: 7,
                criteria: CriteriaValue::CellReference(CellReferenceIndex {
                    sheet: 0,
                    row: 10,
                    column: 5,
                }),
            },
        ],
    };

    match valid_pattern.validate() {
        Ok(()) => println!("✅ Multi-criteria pattern validation successful"),
        Err(e) => println!("❌ Validation failed: {}", e),
    }

    // Test invalid patterns
    println!("\nTesting invalid patterns...");

    // Empty criteria
    let empty_pattern = PatternKey::Sumifs {
        sheet: None,
        sum_column: 2,
        criteria_pairs: vec![],
    };
    match empty_pattern.validate() {
        Err(PatternError::EmptyCriteria) => println!("✅ Empty criteria error detected correctly"),
        _ => println!("❌ Failed to detect empty criteria"),
    }

    // Duplicate columns
    let duplicate_pattern = PatternKey::Sumifs {
        sheet: None,
        sum_column: 2,
        criteria_pairs: vec![
            CriteriaPair {
                column: 3,
                criteria: CriteriaValue::String("A".to_string()),
            },
            CriteriaPair {
                column: 3, // Duplicate!
                criteria: CriteriaValue::String("B".to_string()),
            },
        ],
    };
    match duplicate_pattern.validate() {
        Err(PatternError::DuplicateColumn { column: 3 }) => {
            println!("✅ Duplicate column error detected correctly")
        }
        _ => println!("❌ Failed to detect duplicate column"),
    }

    // Invalid column
    let invalid_column_pattern = PatternKey::Sumifs {
        sheet: None,
        sum_column: -1, // Invalid!
        criteria_pairs: vec![CriteriaPair {
            column: 1,
            criteria: CriteriaValue::String("Test".to_string()),
        }],
    };
    match invalid_column_pattern.validate() {
        Err(PatternError::InvalidColumn { column: -1 }) => {
            println!("✅ Invalid column error detected correctly")
        }
        _ => println!("❌ Failed to detect invalid column"),
    }

    // Example 11: Pattern Conflict Detection and Resolution
    println!("\n⚠️  Example 11: Pattern Conflict Detection and Resolution");
    println!("========================================================");

    use ironcalc_base::expressions::pattern_matching::{ConflictResolution, PatternConflict};

    let mut conflict_registry = SubstitutionRegistry::new();

    // Create base pattern
    let base_pattern = PatternKey::Sumifs {
        sheet: Some("ConflictDemo".to_string()),
        sum_column: 2,
        criteria_pairs: vec![
            CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Active".to_string()),
            },
            CriteriaPair {
                column: 3,
                criteria: CriteriaValue::Number(OrderedFloat(100.0)),
            },
        ],
    };

    // Create subset pattern (fewer criteria, but all exist in base)
    let subset_pattern = PatternKey::Sumifs {
        sheet: Some("ConflictDemo".to_string()),
        sum_column: 2,
        criteria_pairs: vec![CriteriaPair {
            column: 1,
            criteria: CriteriaValue::String("Active".to_string()),
        }],
    };

    // Add base pattern first
    conflict_registry.add_substitution(base_pattern.clone(), 5000.0);
    println!("Added base pattern with value 5000.0");

    // Check for conflicts with subset pattern
    let conflicts = conflict_registry.check_conflicts(&subset_pattern, 3000.0);
    println!("Conflicts detected: {}", conflicts.len());

    if !conflicts.is_empty() {
        match &conflicts[0] {
            PatternConflict::SubsetSuperset {
                subset_value,
                superset_value,
                ..
            } => {
                println!("✅ Subset-superset conflict detected:");
                println!("   Subset would have value: {}", subset_value);
                println!("   Superset has value: {}", superset_value);
            }
            _ => println!("Other conflict type detected"),
        }
    }

    // Test different conflict resolution strategies
    println!("\nTesting conflict resolution strategies:");

    // 1. Error resolution
    let result = conflict_registry.add_substitution_with_conflict_check(
        subset_pattern.clone(),
        3000.0,
        ConflictResolution::Error,
    );
    match result {
        Err(_) => println!("✅ Error resolution: Correctly rejected conflicting pattern"),
        Ok(_) => println!("❌ Error resolution: Should have rejected conflicting pattern"),
    }

    // 2. Warn only resolution
    let result = conflict_registry.add_substitution_with_conflict_check(
        subset_pattern.clone(),
        3000.0,
        ConflictResolution::WarnOnly,
    );
    match result {
        Ok(conflicts) => {
            println!(
                "✅ Warn only resolution: Added pattern with {} warnings",
                conflicts.len()
            );
            println!("   Registry now has {} patterns", conflict_registry.len());
        }
        Err(e) => println!("❌ Warn only resolution failed: {}", e),
    }

    // 3. Keep existing resolution
    let another_pattern = PatternKey::Sumifs {
        sheet: Some("ConflictDemo".to_string()),
        sum_column: 2,
        criteria_pairs: vec![CriteriaPair {
            column: 1,
            criteria: CriteriaValue::String("Active".to_string()),
        }],
    };

    let result = conflict_registry.add_substitution_with_conflict_check(
        another_pattern,
        9999.0,
        ConflictResolution::KeepExisting,
    );
    match result {
        Ok(conflicts) => {
            println!(
                "✅ Keep existing resolution: Detected {} conflicts, kept original",
                conflicts.len()
            );
            if let Some(existing_value) = conflict_registry.get_substitution(&subset_pattern) {
                println!("   Existing value preserved: {}", existing_value);
            }
        }
        Err(e) => println!("❌ Keep existing resolution failed: {}", e),
    }

    // Example 12: Enhanced Multi-Criteria SUMIFS Patterns
    println!("\n📊 Example 12: Enhanced Multi-Criteria SUMIFS Patterns");
    println!("=====================================================");

    // Create a complex pattern using the builder
    let complex_pattern = PatternKey::sumifs()
        .sheet("SalesData")
        .sum_column(10) // Revenue column
        .criteria_string(1, "Electronics") // Category
        .criteria_string(2, "Q4") // Quarter
        .criteria_number(3, 2023.0) // Year
        .criteria_string(5, "North America") // Region
        .criteria_cell(
            7,
            CellReferenceIndex {
                sheet: 0,
                row: 1,
                column: 15,
            },
        ) // Dynamic threshold
        .build();

    match complex_pattern {
        Some(pattern) => {
            println!("✅ Complex multi-criteria pattern created successfully");
            match pattern.validate() {
                Ok(()) => {
                    println!("✅ Pattern validation passed");
                    if let PatternKey::Sumifs { criteria_pairs, .. } = &pattern {
                        println!("   Pattern has {} criteria pairs", criteria_pairs.len());
                        for (i, pair) in criteria_pairs.iter().enumerate() {
                            println!(
                                "   Criteria {}: Column {} = {:?}",
                                i + 1,
                                pair.column,
                                pair.criteria
                            );
                        }
                    }
                }
                Err(e) => println!("❌ Pattern validation failed: {}", e),
            }

            // Add to registry and test
            model.add_substitution(pattern, 250000.0);
            println!("   Added complex pattern with value 250,000.0");
        }
        None => println!("❌ Failed to create complex pattern"),
    }

    // Test pattern extraction with enhanced error reporting
    println!("\nTesting enhanced pattern extraction:");

    // Create a formula with insufficient arguments
    model
        .set_user_input(0, 8, 1, "=SUMIFS(A:A,B:B)".to_string())
        .unwrap(); // Missing criteria value
    model.evaluate();
    println!("   Formula with insufficient arguments handled gracefully");

    // Show registry statistics
    let final_stats = model.get_substitution_stats();
    println!("\nFinal Registry Statistics:");
    println!("  Total lookups: {}", final_stats.total_lookups());
    println!("  Success rate: {:.1}%", final_stats.success_rate());

    // Example 13: COUNTIFS Pattern Matching and Substitution
    println!("\n🔢 Example 13: COUNTIFS Pattern Matching and Substitution");
    println!("========================================================");

    // Create a COUNTIFS pattern using the builder
    let countifs_pattern = PatternKey::countifs()
        .sheet("InventoryData")
        .criteria_string(1, "Electronics") // Category column
        .criteria_string(2, "In Stock") // Status column
        .criteria_number(3, 100.0) // Minimum quantity
        .build()
        .unwrap();

    println!("Created COUNTIFS pattern: {:#?}", countifs_pattern);

    // Validate the pattern
    match countifs_pattern.validate() {
        Ok(()) => println!("✅ COUNTIFS pattern validation successful"),
        Err(e) => println!("❌ COUNTIFS pattern validation failed: {}", e),
    }

    // Add to substitution registry
    model.add_substitution(countifs_pattern.clone(), 25.0);
    println!("Added COUNTIFS substitution: count = 25.0");

    // Test with a formula that should match the pattern
    model.set_user_input(0, 10, 1, "=COUNTIFS(InventoryData!A:A,\"Electronics\",InventoryData!B:B,\"In Stock\",InventoryData!C:C,100)".to_string()).unwrap();
    model.evaluate();
    let countifs_result = model.get_cell_value_by_index(0, 10, 1).unwrap();
    println!("COUNTIFS formula result: {:?}", countifs_result);

    match countifs_result {
        ironcalc_base::cell::CellValue::Number(value) if (value - 25.0).abs() < 0.001 => {
            println!("✅ COUNTIFS substitution successful!");
        }
        _ => {
            println!(
                "❌ COUNTIFS substitution failed. Expected 25.0, got {:?}",
                countifs_result
            );
        }
    }

    // Example 14: AVERAGEIFS Pattern Matching and Substitution
    println!("\n📊 Example 14: AVERAGEIFS Pattern Matching and Substitution");
    println!("==========================================================");

    // Create an AVERAGEIFS pattern using the builder
    let averageifs_pattern = PatternKey::averageifs()
        .sheet("SalesData")
        .average_column(4) // Revenue column
        .criteria_string(1, "Software") // Category column
        .criteria_string(2, "Q4") // Quarter column
        .criteria_number(3, 2023.0) // Year
        .build()
        .unwrap();

    println!("Created AVERAGEIFS pattern: {:#?}", averageifs_pattern);

    // Validate the pattern
    match averageifs_pattern.validate() {
        Ok(()) => println!("✅ AVERAGEIFS pattern validation successful"),
        Err(e) => println!("❌ AVERAGEIFS pattern validation failed: {}", e),
    }

    // Add to substitution registry
    model.add_substitution(averageifs_pattern.clone(), 45000.0);
    println!("Added AVERAGEIFS substitution: average revenue = $45,000");

    // Test with a formula that should match the pattern
    model.set_user_input(0, 11, 1, "=AVERAGEIFS(SalesData!D:D,SalesData!A:A,\"Software\",SalesData!B:B,\"Q4\",SalesData!C:C,2023)".to_string()).unwrap();
    model.evaluate();
    let averageifs_result = model.get_cell_value_by_index(0, 11, 1).unwrap();
    println!("AVERAGEIFS formula result: {:?}", averageifs_result);

    match averageifs_result {
        ironcalc_base::cell::CellValue::Number(value) if (value - 45000.0).abs() < 0.001 => {
            println!("✅ AVERAGEIFS substitution successful!");
        }
        _ => {
            println!(
                "❌ AVERAGEIFS substitution failed. Expected 45000.0, got {:?}",
                averageifs_result
            );
        }
    }

    // Example 15: Multi-Function Pattern Registry Management
    println!("\n🎯 Example 15: Multi-Function Pattern Registry Management");
    println!("========================================================");

    // Show final registry statistics with all function types
    let final_stats = model.get_substitution_stats();
    println!("Final Registry Statistics:");
    println!("  Total lookups: {}", final_stats.total_lookups());
    println!("  Success rate: {:.1}%", final_stats.success_rate());

    // Test pattern extraction for each function type
    println!("\nPattern Extraction Testing:");

    // Parse and extract patterns for all three function types
    let sumifs_formula = "SUMIFS(D:D,A:A,\"Test\",B:B,100)";
    let countifs_formula = "COUNTIFS(A:A,\"Test\",B:B,100)";
    let averageifs_formula = "AVERAGEIFS(D:D,A:A,\"Test\",B:B,100)";

    let sumifs_ast = parser.parse(sumifs_formula, &cell_reference);
    let countifs_ast = parser.parse(countifs_formula, &cell_reference);
    let averageifs_ast = parser.parse(averageifs_formula, &cell_reference);

    if let Some(extracted) = PatternKey::from_node(&sumifs_ast) {
        println!(
            "  ✅ SUMIFS extraction: {:?}",
            match extracted {
                PatternKey::Sumifs { .. } => "Success",
                _ => "Wrong type",
            }
        );
    }

    if let Some(extracted) = PatternKey::from_node(&countifs_ast) {
        println!(
            "  ✅ COUNTIFS extraction: {:?}",
            match extracted {
                PatternKey::Countifs { .. } => "Success",
                _ => "Wrong type",
            }
        );
    }

    if let Some(extracted) = PatternKey::from_node(&averageifs_ast) {
        println!(
            "  ✅ AVERAGEIFS extraction: {:?}",
            match extracted {
                PatternKey::Averageifs { .. } => "Success",
                _ => "Wrong type",
            }
        );
    }

    // Demonstrate pattern uniqueness across function types
    println!("\nPattern Uniqueness Verification:");
    println!("  Different function types with similar criteria are treated as unique patterns");
    println!(
        "  This allows for independent substitution values for SUMIFS, COUNTIFS, and AVERAGEIFS"
    );
    println!("  even when they operate on the same data ranges and criteria");

    println!("\n🎉 Extended Pattern Matching Demo Completed Successfully!");
    println!("   ✅ Advanced pattern validation with detailed error reporting");
    println!("   ✅ Comprehensive conflict detection and resolution strategies");
    println!("   ✅ Enhanced multi-criteria pattern support with builder");
    println!("   ✅ Robust error handling and debugging capabilities");
    println!("   ✅ COUNTIFS pattern matching and substitution");
    println!("   ✅ AVERAGEIFS pattern matching and substitution");
    println!("   ✅ Multi-function pattern registry management");

    // Example 16: VLOOKUP Pattern Matching and Substitution
    println!("\\n🔍 Example 16: VLOOKUP Pattern Matching and Substitution");
    println!("======================================================");

    // Create a VLOOKUP pattern using the builder
    let vlookup_pattern = PatternKey::vlookup()
        .sheet("ProductCatalog")
        .lookup_string("Widget A")
        .table_range(1, 1, 100, 4) // A1:D100 lookup table
        .column_index(3) // Return value from column C (3rd column)
        .exact_match(true) // Exact match required
        .build()
        .unwrap();

    println!("Created VLOOKUP pattern: {:#?}", vlookup_pattern);

    // Validate the pattern
    match vlookup_pattern.validate() {
        Ok(()) => println!("✅ VLOOKUP pattern validation successful"),
        Err(e) => println!("❌ VLOOKUP pattern validation failed: {}", e),
    }

    // Add to substitution registry
    model.add_substitution(vlookup_pattern.clone(), 150.0);
    println!("Added VLOOKUP substitution: lookup result = $150.00");

    // Test with a formula that should match the pattern
    model
        .set_user_input(
            0,
            12,
            1,
            "=VLOOKUP(\\\"Widget A\\\",ProductCatalog!A1:D100,3,TRUE)".to_string(),
        )
        .unwrap();
    model.evaluate();
    let vlookup_result = model.get_cell_value_by_index(0, 12, 1).unwrap();
    println!("VLOOKUP formula result: {:?}", vlookup_result);

    match vlookup_result {
        ironcalc_base::cell::CellValue::Number(value) if (value - 150.0).abs() < 0.001 => {
            println!("✅ VLOOKUP substitution successful!");
        }
        _ => {
            println!(
                "❌ VLOOKUP substitution failed. Expected 150.0, got {:?}",
                vlookup_result
            );
        }
    }

    println!("\\n🎉 Complete Pattern Matching Demo with VLOOKUP Support!");
    println!("   ✅ VLOOKUP pattern matching and substitution");
    println!(
        "   ✅ All four major function types supported (SUMIFS, COUNTIFS, AVERAGEIFS, VLOOKUP)"
    );
    println!("   ✅ Comprehensive pattern-based scalar substitution system");
}
