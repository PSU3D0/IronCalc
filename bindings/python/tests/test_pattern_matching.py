#!/usr/bin/env python3
"""
Tests for IronCalc Pattern Matching functionality
"""

import pytest
import ironcalc

class TestPatternMatching:
    """Test class for pattern matching functionality"""
    
    def setup_method(self):
        """Set up test fixtures before each test method"""
        self.model = ironcalc.create("TestModel", "en", "UTC")
        
        # Set up test data
        self.model.set_user_input(0, 1, 1, "Product", reevaluate=False)
        self.model.set_user_input(0, 1, 2, "Sales", reevaluate=False)
        self.model.set_user_input(0, 1, 3, "Region", reevaluate=False)
        
        self.model.set_user_input(0, 2, 1, "Widget A", reevaluate=False)
        self.model.set_user_input(0, 2, 2, 5000, reevaluate=False)
        self.model.set_user_input(0, 2, 3, "North", reevaluate=False)
        
        self.model.set_user_input(0, 3, 1, "Widget B", reevaluate=False)
        self.model.set_user_input(0, 3, 2, 3000, reevaluate=False)
        self.model.set_user_input(0, 3, 3, "South", reevaluate=False)
        
        self.model.set_user_input(0, 4, 1, "Widget A", reevaluate=False)
        self.model.set_user_input(0, 4, 2, 2000, reevaluate=False)
        self.model.set_user_input(0, 4, 3, "South", reevaluate=False)

    def test_sumifs_pattern_creation(self):
        """Test SUMIFS pattern creation and validation"""
        pattern = (ironcalc.PyPatternKey.sumifs()
                  .sum_column(2)
                  .criteria_string(1, "Widget A")
                  .criteria_string(3, "North")
                  .build())
        
        assert pattern is not None
        assert pattern.pattern_type() == "SUMIFS"
        
        # Validation should pass
        pattern.validate()
        
    def test_sumifs_pattern_missing_sum_column(self):
        """Test SUMIFS pattern creation fails without sum column"""
        builder = ironcalc.PyPatternKey.sumifs().criteria_string(1, "Widget A")
        
        with pytest.raises(ValueError, match="Sum column is required"):
            builder.build()
    
    def test_sumifs_pattern_missing_criteria(self):
        """Test SUMIFS pattern creation fails without criteria"""
        builder = ironcalc.PyPatternKey.sumifs().sum_column(2)
        
        with pytest.raises(ValueError, match="At least one criteria pair is required"):
            builder.build()
    
    def test_countifs_pattern_creation(self):
        """Test COUNTIFS pattern creation and validation"""
        pattern = (ironcalc.PyPatternKey.countifs()
                  .criteria_string(1, "Widget A")
                  .criteria_number(2, 5000.0)
                  .build())
        
        assert pattern is not None
        assert pattern.pattern_type() == "COUNTIFS"
        pattern.validate()
    
    def test_averageifs_pattern_creation(self):
        """Test AVERAGEIFS pattern creation and validation"""
        pattern = (ironcalc.PyPatternKey.averageifs()
                  .average_column(2)
                  .criteria_string(1, "Widget A")
                  .build())
        
        assert pattern is not None
        assert pattern.pattern_type() == "AVERAGEIFS"
        pattern.validate()
    
    def test_vlookup_pattern_creation(self):
        """Test VLOOKUP pattern creation and validation"""
        pattern = (ironcalc.PyPatternKey.vlookup()
                  .lookup_string("Widget A")
                  .table_range(1, 1, 10, 4)
                  .column_index(2)
                  .exact_match(True)
                  .build())
        
        assert pattern is not None
        assert pattern.pattern_type() == "VLOOKUP"
        pattern.validate()
    
    def test_vlookup_pattern_missing_fields(self):
        """Test VLOOKUP pattern creation fails when missing required fields"""
        builder = (ironcalc.PyPatternKey.vlookup()
                  .lookup_string("Widget A")
                  .table_range(1, 1, 10, 4))
        
        with pytest.raises(ValueError, match="All VLOOKUP fields are required"):
            builder.build()
    
    def test_sumifs_substitution(self):
        """Test SUMIFS pattern substitution in model"""
        # Create pattern
        pattern = (ironcalc.PyPatternKey.sumifs()
                  .sum_column(2)
                  .criteria_string(1, "Widget A")
                  .criteria_string(3, "North")
                  .build())
        
        # Add formula to model
        self.model.set_user_input(0, 6, 1, '=SUMIFS(B:B,A:A,"Widget A",C:C,"North")')
        
        # Evaluate without substitution
        self.model.evaluate()
        original_result = self.model.evaluate_cell(None, 0, 6, 1)
        
        # Add substitution
        substitution_value = 99999.0
        self.model.add_substitution(pattern, substitution_value)
        
        # Check if pattern exists
        assert self.model.has_substitution(pattern)
        assert self.model.get_substitution(pattern) == substitution_value
        
        # Re-evaluate with substitution
        self.model.evaluate()
        substituted_result = self.model.evaluate_cell(None, 0, 6, 1)
        
        assert abs(substituted_result - substitution_value) < 0.001
    
    def test_countifs_substitution(self):
        """Test COUNTIFS pattern substitution in model"""
        pattern = (ironcalc.PyPatternKey.countifs()
                  .criteria_string(1, "Widget A")
                  .build())
        
        self.model.set_user_input(0, 7, 1, '=COUNTIFS(A:A,"Widget A")')
        
        substitution_value = 42.0
        self.model.add_substitution(pattern, substitution_value)
        
        self.model.evaluate()
        result = self.model.evaluate_cell(None, 0, 7, 1)
        
        assert abs(result - substitution_value) < 0.001
    
    def test_averageifs_substitution(self):
        """Test AVERAGEIFS pattern substitution in model"""
        pattern = (ironcalc.PyPatternKey.averageifs()
                  .average_column(2)
                  .criteria_string(3, "South")
                  .build())
        
        self.model.set_user_input(0, 8, 1, '=AVERAGEIFS(B:B,C:C,"South")')
        
        substitution_value = 3500.0
        self.model.add_substitution(pattern, substitution_value)
        
        self.model.evaluate()
        result = self.model.evaluate_cell(None, 0, 8, 1)
        
        assert abs(result - substitution_value) < 0.001
    
    def test_vlookup_substitution(self):
        """Test VLOOKUP pattern substitution in model"""
        # Set up lookup table
        self.model.set_user_input(0, 10, 1, "Widget A", reevaluate=False)
        self.model.set_user_input(0, 10, 2, 150.0, reevaluate=False)
        self.model.set_user_input(0, 11, 1, "Widget B", reevaluate=False)
        self.model.set_user_input(0, 11, 2, 200.0, reevaluate=False)
        
        pattern = (ironcalc.PyPatternKey.vlookup()
                  .lookup_string("Widget A")
                  .table_range(10, 1, 11, 2)
                  .column_index(2)
                  .exact_match(True)
                  .build())
        
        self.model.set_user_input(0, 13, 1, '=VLOOKUP("Widget A",A10:B11,2,TRUE)')
        
        substitution_value = 175.0
        self.model.add_substitution(pattern, substitution_value)
        
        self.model.evaluate()
        result = self.model.evaluate_cell(None, 0, 13, 1)
        
        assert abs(result - substitution_value) < 0.001
    
    def test_substitution_deactivation(self):
        """Test substitution registry deactivation"""
        pattern = (ironcalc.PyPatternKey.sumifs()
                  .sum_column(2)
                  .criteria_string(1, "Widget A")
                  .build())
        
        self.model.set_user_input(0, 6, 1, '=SUMIFS(B:B,A:A,"Widget A")')
        
        # Get original result
        self.model.evaluate()
        original_result = self.model.evaluate_cell(None, 0, 6, 1)
        
        # Add substitution
        self.model.add_substitution(pattern, 99999.0)
        self.model.evaluate()
        substituted_result = self.model.evaluate_cell(None, 0, 6, 1)
        
        # Deactivate substitutions
        self.model.set_substitutions_active(False)
        assert not self.model.substitutions_active()
        
        self.model.evaluate()
        deactivated_result = self.model.evaluate_cell(None, 0, 6, 1)
        
        # Should return to original result
        assert abs(deactivated_result - original_result) < 0.001
        
        # Reactivate
        self.model.set_substitutions_active(True)
        assert self.model.substitutions_active()
        
        self.model.evaluate()
        reactivated_result = self.model.evaluate_cell(None, 0, 6, 1)
        
        # Should return to substituted result
        assert abs(reactivated_result - substituted_result) < 0.001
    
    def test_substitution_statistics(self):
        """Test substitution registry statistics"""
        pattern = (ironcalc.PyPatternKey.sumifs()
                  .sum_column(2)
                  .criteria_string(1, "Widget A")
                  .build())
        
        self.model.add_substitution(pattern, 12345.0)
        self.model.set_user_input(0, 6, 1, '=SUMIFS(B:B,A:A,"Widget A")')
        
        # Trigger some lookups
        self.model.evaluate()
        self.model.evaluate_cell(None, 0, 6, 1)
        
        stats = self.model.get_substitution_stats()
        
        assert "total_lookups" in stats
        assert "successful_lookups" in stats
        assert "failed_lookups" in stats
        assert "success_rate" in stats
        assert stats["total_lookups"] > 0
    
    def test_bulk_substitutions(self):
        """Test bulk substitution operations"""
        pattern1 = (ironcalc.PyPatternKey.sumifs()
                   .sum_column(2)
                   .criteria_string(1, "Widget A")
                   .build())
        
        pattern2 = (ironcalc.PyPatternKey.countifs()
                   .criteria_string(1, "Widget B")
                   .build())
        
        bulk_patterns = [(pattern1, 1000.0), (pattern2, 20.0)]
        
        added_count = self.model.add_substitutions_bulk(bulk_patterns)
        assert added_count == 2
        
        assert self.model.has_substitution(pattern1)
        assert self.model.has_substitution(pattern2)
    
    def test_clear_substitutions(self):
        """Test clearing all substitutions"""
        pattern = (ironcalc.PyPatternKey.sumifs()
                  .sum_column(2)
                  .criteria_string(1, "Widget A")
                  .build())
        
        self.model.add_substitution(pattern, 12345.0)
        assert self.model.has_substitution(pattern)
        
        self.model.clear_substitutions()
        assert not self.model.has_substitution(pattern)
    
    def test_standalone_substitution_registry(self):
        """Test standalone SubstitutionRegistry"""
        registry = ironcalc.PySubstitutionRegistry()
        
        pattern = (ironcalc.PyPatternKey.sumifs()
                  .sum_column(2)
                  .criteria_string(1, "Test")
                  .build())
        
        assert registry.len() == 0
        assert registry.is_active()
        
        registry.add_substitution(pattern, 5000.0)
        assert registry.len() == 1
        assert registry.has_substitution(pattern)
        assert registry.get_substitution(pattern) == 5000.0
        
        # Test deactivation
        registry.set_active(False)
        assert not registry.is_active()
        
        # Test statistics
        stats = registry.stats()
        assert "total_lookups" in stats
        
        # Test clearing
        registry.clear()
        assert registry.len() == 0
    
    def test_criteria_value_types(self):
        """Test different criteria value types"""
        # String criteria
        string_criteria = ironcalc.PyCriteriaValue.string("Test")
        assert "String(Test)" in str(string_criteria)
        
        # Number criteria
        number_criteria = ironcalc.PyCriteriaValue.number(123.45)
        assert "Number(123.45)" in str(number_criteria)
        
        # Cell reference criteria
        cell_criteria = ironcalc.PyCriteriaValue.cell_ref(0, 1, 1)
        assert "CellRef(sheet=0, row=1, column=1)" in str(cell_criteria)
    
    def test_lookup_value_types(self):
        """Test different lookup value types"""
        # String lookup
        string_lookup = ironcalc.PyLookupValue.string("Widget A")
        assert "String(Widget A)" in str(string_lookup)
        
        # Number lookup
        number_lookup = ironcalc.PyLookupValue.number(42.0)
        assert "Number(42)" in str(number_lookup)
        
        # Cell reference lookup
        cell_lookup = ironcalc.PyLookupValue.cell_ref(0, 2, 3)
        assert "CellRef(sheet=0, row=2, column=3)" in str(cell_lookup)
    
    def test_table_range(self):
        """Test TableRange functionality"""
        table_range = ironcalc.PyTableRange(None, 1, 1, 10, 4)
        assert table_range.sheet is None
        assert table_range.start_row == 1
        assert table_range.start_column == 1
        assert table_range.end_row == 10
        assert table_range.end_column == 4
        
        table_range_with_sheet = ironcalc.PyTableRange("Sheet1", 1, 1, 10, 4)
        assert table_range_with_sheet.sheet == "Sheet1"
    
    def test_pattern_equality_and_hashing(self):
        """Test pattern equality and hashing for use in dictionaries"""
        pattern1 = (ironcalc.PyPatternKey.sumifs()
                   .sum_column(2)
                   .criteria_string(1, "Widget A")
                   .build())
        
        pattern2 = (ironcalc.PyPatternKey.sumifs()
                   .sum_column(2)
                   .criteria_string(1, "Widget A")
                   .build())
        
        pattern3 = (ironcalc.PyPatternKey.sumifs()
                   .sum_column(3)
                   .criteria_string(1, "Widget A")
                   .build())
        
        assert pattern1 == pattern2
        assert pattern1 != pattern3
        assert hash(pattern1) == hash(pattern2)
        assert hash(pattern1) != hash(pattern3)
    
    def test_pattern_string_representations(self):
        """Test string representations of patterns"""
        pattern = (ironcalc.PyPatternKey.sumifs()
                  .sum_column(2)
                  .criteria_string(1, "Widget A")
                  .build())
        
        pattern_str = str(pattern)
        assert "SUMIFS" in pattern_str or "Sumifs" in pattern_str
        
        # Test repr
        pattern_repr = repr(pattern)
        assert pattern_repr is not None

if __name__ == "__main__":
    pytest.main([__file__, "-v"])