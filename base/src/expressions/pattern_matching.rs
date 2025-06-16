/*!
# Pattern Matching for Formula Expressions

This module provides type-safe pattern matching for formula expressions, enabling
scalar substitution during evaluation for perturbation-based scenario modeling.

## Core Components

- [`PatternKey`]: Type-safe, hashable pattern identification
- [`CriteriaPair`]: Column-criteria pair for IFS functions
- [`CriteriaValue`]: Strongly-typed criteria values
- [`SumifsBuilder`]: Ergonomic builder for SUMIFS patterns

## Usage

```rust
use crate::expressions::pattern_matching::*;

// Create a SUMIFS pattern using the builder
let pattern = PatternKey::sumifs()
    .sheet("INJECT.MONTHLY")
    .sum_column(4) // Column D
    .criteria(6, CriteriaValue::String("Collections Clinic".to_string())) // Column F
    .build()
    .unwrap();

// Or create directly
let pattern = PatternKey::Sumifs {
    sheet: Some("INJECT.MONTHLY".to_string()),
    sum_column: 4,
    criteria_pairs: vec![
        CriteriaPair {
            column: 6,
            criteria: CriteriaValue::String("Collections Clinic".to_string()),
        }
    ],
};
```
*/

use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::Function;
use ordered_float::OrderedFloat;
use std::fmt;
use std::hash::Hash;

/// Errors that can occur during pattern validation or extraction
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternError {
    /// Pattern contains no criteria pairs
    EmptyCriteria,
    /// Pattern has too many criteria pairs (implementation limit)
    TooManyCriteria { max: usize, found: usize },
    /// Invalid column index (must be >= 0)
    InvalidColumn { column: i32 },
    /// Duplicate criteria column
    DuplicateColumn { column: i32 },
    /// Invalid sheet name
    InvalidSheetName { name: String },
    /// Pattern extraction failed due to unsupported node type
    UnsupportedNode { node_type: String },
    /// Formula has wrong number of arguments
    InvalidArgumentCount { expected: String, found: usize },
    /// Multi-column ranges are not supported for pattern matching
    MultiColumnRange { start: i32, end: i32 },
    /// Cell reference is out of bounds
    CellOutOfBounds { sheet: u32, row: i32, column: i32 },
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternError::EmptyCriteria => {
                write!(f, "Pattern must contain at least one criteria pair")
            }
            PatternError::TooManyCriteria { max, found } => {
                write!(
                    f,
                    "Too many criteria pairs: found {}, maximum supported is {}",
                    found, max
                )
            }
            PatternError::InvalidColumn { column } => {
                write!(f, "Invalid column index: {} (must be >= 0)", column)
            }
            PatternError::DuplicateColumn { column } => {
                write!(f, "Duplicate criteria column: {}", column)
            }
            PatternError::InvalidSheetName { name } => {
                write!(f, "Invalid sheet name: '{}'", name)
            }
            PatternError::UnsupportedNode { node_type } => {
                write!(
                    f,
                    "Unsupported node type for pattern extraction: {}",
                    node_type
                )
            }
            PatternError::InvalidArgumentCount { expected, found } => {
                write!(
                    f,
                    "Invalid argument count: expected {}, found {}",
                    expected, found
                )
            }
            PatternError::MultiColumnRange { start, end } => {
                write!(
                    f,
                    "Multi-column ranges not supported: columns {} to {}",
                    start, end
                )
            }
            PatternError::CellOutOfBounds { sheet, row, column } => {
                write!(
                    f,
                    "Cell reference out of bounds: sheet {}, row {}, column {}",
                    sheet, row, column
                )
            }
        }
    }
}

impl std::error::Error for PatternError {}

/// Type-safe pattern key for identifying formula patterns in substitution systems.
///
/// Patterns are designed to be hashable for O(1) lookup in HashMap-based registries.
/// Each variant represents a different function type that can be pattern-matched.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatternKey {
    /// SUMIFS function pattern
    ///
    /// Matches: `SUMIFS(sum_range, criteria_range1, criteria1, [criteria_range2, criteria2, ...])`
    Sumifs {
        /// Optional sheet name for the ranges (None for current sheet)
        sheet: Option<String>,
        /// Column index for the sum range (1-based)
        sum_column: i32,
        /// Vector of criteria pairs (column + criteria value)
        criteria_pairs: Vec<CriteriaPair>,
    },
    /// COUNTIFS function pattern
    ///
    /// Matches: `COUNTIFS(criteria_range1, criteria1, [criteria_range2, criteria2, ...])`
    Countifs {
        /// Optional sheet name for the ranges (None for current sheet)
        sheet: Option<String>,
        /// Vector of criteria pairs (column + criteria value)
        criteria_pairs: Vec<CriteriaPair>,
    },
    /// AVERAGEIFS function pattern
    ///
    /// Matches: `AVERAGEIFS(average_range, criteria_range1, criteria1, [criteria_range2, criteria2, ...])`
    Averageifs {
        /// Optional sheet name for the ranges (None for current sheet)
        sheet: Option<String>,
        /// Column index for the average range (1-based)
        average_column: i32,
        /// Vector of criteria pairs (column + criteria value)
        criteria_pairs: Vec<CriteriaPair>,
    },
    /// VLOOKUP function pattern
    ///
    /// Matches: `VLOOKUP(lookup_value, table_array, column_index_num, [range_lookup])`
    Vlookup {
        /// Optional sheet name for the table (None for current sheet)
        sheet: Option<String>,
        /// The value to look up
        lookup_value: LookupValue,
        /// The table range to search in
        table_range: TableRange,
        /// The column index to return (1-based)
        column_index: u32,
        /// Whether to use exact match (false) or approximate match (true)
        exact_match: bool,
    },
}

/// A column-criteria pair for IFS function patterns.
///
/// Represents one criteria condition: "in this column, match this criteria"
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CriteriaPair {
    /// Column index (1-based) for the criteria range
    pub column: i32,
    /// The criteria value to match against
    pub criteria: CriteriaValue,
}

/// Strongly-typed criteria values for pattern matching.
///
/// Supports the common types used in Excel/spreadsheet criteria:
/// - String literals: "Collections Clinic"
/// - Numeric values: 1811349806
/// - Cell references: C2, C30
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CriteriaValue {
    /// String literal criteria
    String(String),
    /// Numeric criteria (using OrderedFloat for Hash implementation)
    Number(OrderedFloat<f64>),
    /// Cell reference criteria (evaluates to cell's current value)
    CellReference(CellReferenceIndex),
}

/// Lookup value for VLOOKUP pattern matching.
///
/// Represents the value being searched for in the first column of the table.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LookupValue {
    /// String literal lookup value
    String(String),
    /// Numeric lookup value (using OrderedFloat for Hash implementation)
    Number(OrderedFloat<f64>),
    /// Cell reference lookup value (evaluates to cell's current value)
    CellReference(CellReferenceIndex),
}

impl LookupValue {
    /// Create a string lookup value
    pub fn string<S: Into<String>>(value: S) -> Self {
        LookupValue::String(value.into())
    }

    /// Create a numeric lookup value
    pub fn number(value: f64) -> Self {
        LookupValue::Number(OrderedFloat(value))
    }

    /// Create a cell reference lookup value
    pub fn cell_ref(sheet: u32, row: i32, column: i32) -> Self {
        LookupValue::CellReference(CellReferenceIndex { sheet, row, column })
    }
}

/// Table range for VLOOKUP pattern matching.
///
/// Represents a rectangular range that serves as the lookup table.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TableRange {
    /// Optional sheet name (None for current sheet)
    pub sheet: Option<String>,
    /// Starting row of the table (1-based)
    pub start_row: i32,
    /// Starting column of the table (1-based)
    pub start_column: i32,
    /// Ending row of the table (1-based)
    pub end_row: i32,
    /// Ending column of the table (1-based)
    pub end_column: i32,
}

impl PatternKey {
    /// Create a new SUMIFS builder for ergonomic pattern construction
    pub fn sumifs() -> SumifsBuilder {
        SumifsBuilder::new()
    }

    /// Create a new COUNTIFS builder for ergonomic pattern construction
    pub fn countifs() -> CountifsBuilder {
        CountifsBuilder::new()
    }

    /// Create a new AVERAGEIFS builder for ergonomic pattern construction
    pub fn averageifs() -> AverageifsBuilder {
        AverageifsBuilder::new()
    }

    /// Create a new VLOOKUP builder for ergonomic pattern construction
    pub fn vlookup() -> VlookupBuilder {
        VlookupBuilder::new()
    }

    /// Validate this pattern for correctness and consistency
    ///
    /// # Returns
    /// * `Ok(())` - If the pattern is valid
    /// * `Err(PatternError)` - If the pattern has validation errors
    pub fn validate(&self) -> Result<(), PatternError> {
        match self {
            PatternKey::Sumifs {
                sheet,
                sum_column,
                criteria_pairs,
            } => self.validate_sumifs(sheet, *sum_column, criteria_pairs),
            PatternKey::Countifs {
                sheet,
                criteria_pairs,
            } => self.validate_countifs(sheet, criteria_pairs),
            PatternKey::Averageifs {
                sheet,
                average_column,
                criteria_pairs,
            } => self.validate_averageifs(sheet, *average_column, criteria_pairs),
            PatternKey::Vlookup {
                sheet,
                lookup_value,
                table_range,
                column_index,
                exact_match: _,
            } => self.validate_vlookup(sheet, lookup_value, table_range, *column_index),
        }
    }

    /// Validate a SUMIFS pattern
    fn validate_sumifs(
        &self,
        sheet: &Option<String>,
        sum_column: i32,
        criteria_pairs: &[CriteriaPair],
    ) -> Result<(), PatternError> {
        // Check for empty criteria
        if criteria_pairs.is_empty() {
            return Err(PatternError::EmptyCriteria);
        }

        // Check for too many criteria (reasonable limit for performance)
        const MAX_CRITERIA: usize = 100;
        if criteria_pairs.len() > MAX_CRITERIA {
            return Err(PatternError::TooManyCriteria {
                max: MAX_CRITERIA,
                found: criteria_pairs.len(),
            });
        }

        // Validate sum column
        if sum_column < 0 {
            return Err(PatternError::InvalidColumn { column: sum_column });
        }

        // Validate sheet name if present
        if let Some(sheet_name) = sheet {
            if sheet_name.is_empty() || sheet_name.len() > 255 {
                return Err(PatternError::InvalidSheetName {
                    name: sheet_name.clone(),
                });
            }
        }

        // Validate each criteria pair
        let mut seen_columns = std::collections::HashSet::new();
        for pair in criteria_pairs {
            // Validate column index
            if pair.column < 0 {
                return Err(PatternError::InvalidColumn {
                    column: pair.column,
                });
            }

            // Check for duplicate columns
            if !seen_columns.insert(pair.column) {
                return Err(PatternError::DuplicateColumn {
                    column: pair.column,
                });
            }

            // Validate criteria value
            self.validate_criteria_value(&pair.criteria)?;
        }

        Ok(())
    }

    /// Validate a COUNTIFS pattern
    fn validate_countifs(
        &self,
        sheet: &Option<String>,
        criteria_pairs: &[CriteriaPair],
    ) -> Result<(), PatternError> {
        // Check for empty criteria
        if criteria_pairs.is_empty() {
            return Err(PatternError::EmptyCriteria);
        }

        // Check for too many criteria (reasonable limit for performance)
        const MAX_CRITERIA: usize = 100;
        if criteria_pairs.len() > MAX_CRITERIA {
            return Err(PatternError::TooManyCriteria {
                max: MAX_CRITERIA,
                found: criteria_pairs.len(),
            });
        }

        // Validate sheet name if present
        if let Some(sheet_name) = sheet {
            if sheet_name.is_empty() || sheet_name.len() > 255 {
                return Err(PatternError::InvalidSheetName {
                    name: sheet_name.clone(),
                });
            }
        }

        // Validate each criteria pair
        let mut seen_columns = std::collections::HashSet::new();
        for pair in criteria_pairs {
            // Validate column index
            if pair.column < 0 {
                return Err(PatternError::InvalidColumn {
                    column: pair.column,
                });
            }

            // Check for duplicate columns
            if !seen_columns.insert(pair.column) {
                return Err(PatternError::DuplicateColumn {
                    column: pair.column,
                });
            }

            // Validate criteria value
            self.validate_criteria_value(&pair.criteria)?;
        }

        Ok(())
    }

    /// Validate an AVERAGEIFS pattern
    fn validate_averageifs(
        &self,
        sheet: &Option<String>,
        average_column: i32,
        criteria_pairs: &[CriteriaPair],
    ) -> Result<(), PatternError> {
        // Check for empty criteria
        if criteria_pairs.is_empty() {
            return Err(PatternError::EmptyCriteria);
        }

        // Check for too many criteria (reasonable limit for performance)
        const MAX_CRITERIA: usize = 100;
        if criteria_pairs.len() > MAX_CRITERIA {
            return Err(PatternError::TooManyCriteria {
                max: MAX_CRITERIA,
                found: criteria_pairs.len(),
            });
        }

        // Validate average column
        if average_column < 0 {
            return Err(PatternError::InvalidColumn {
                column: average_column,
            });
        }

        // Validate sheet name if present
        if let Some(sheet_name) = sheet {
            if sheet_name.is_empty() || sheet_name.len() > 255 {
                return Err(PatternError::InvalidSheetName {
                    name: sheet_name.clone(),
                });
            }
        }

        // Validate each criteria pair
        let mut seen_columns = std::collections::HashSet::new();
        for pair in criteria_pairs {
            // Validate column index
            if pair.column < 0 {
                return Err(PatternError::InvalidColumn {
                    column: pair.column,
                });
            }

            // Check for duplicate columns
            if !seen_columns.insert(pair.column) {
                return Err(PatternError::DuplicateColumn {
                    column: pair.column,
                });
            }

            // Validate criteria value
            self.validate_criteria_value(&pair.criteria)?;
        }

        Ok(())
    }

    /// Validate a criteria value
    fn validate_criteria_value(&self, criteria: &CriteriaValue) -> Result<(), PatternError> {
        match criteria {
            CriteriaValue::String(s) => {
                // String criteria should not be empty (though Excel allows it)
                if s.is_empty() {
                    // This is a warning case, not an error - Excel allows empty string criteria
                }
                Ok(())
            }
            CriteriaValue::Number(n) => {
                // Check for NaN or infinite values
                if !n.is_finite() {
                    // For now, we allow NaN and infinite values as Excel does
                }
                Ok(())
            }
            CriteriaValue::CellReference(cell_ref) => {
                // Validate cell reference bounds
                const MAX_ROWS: i32 = 1_048_576; // Excel row limit
                const MAX_COLS: i32 = 16_384; // Excel column limit

                // Note: sheet is u32, so it can't be negative, but we still check bounds

                if cell_ref.row < 0 || cell_ref.row >= MAX_ROWS {
                    return Err(PatternError::CellOutOfBounds {
                        sheet: cell_ref.sheet,
                        row: cell_ref.row,
                        column: cell_ref.column,
                    });
                }

                if cell_ref.column < 0 || cell_ref.column >= MAX_COLS {
                    return Err(PatternError::CellOutOfBounds {
                        sheet: cell_ref.sheet,
                        row: cell_ref.row,
                        column: cell_ref.column,
                    });
                }

                Ok(())
            }
        }
    }

    /// Validate a VLOOKUP pattern
    fn validate_vlookup(
        &self,
        sheet: &Option<String>,
        lookup_value: &LookupValue,
        table_range: &TableRange,
        column_index: u32,
    ) -> Result<(), PatternError> {
        // Validate sheet name if present
        if let Some(sheet_name) = sheet {
            if sheet_name.is_empty() || sheet_name.len() > 255 {
                return Err(PatternError::InvalidSheetName {
                    name: sheet_name.clone(),
                });
            }
        }

        // Validate lookup value
        self.validate_lookup_value(lookup_value)?;

        // Validate table range
        self.validate_table_range(table_range)?;

        // Validate column index
        if column_index == 0 {
            return Err(PatternError::InvalidColumn { column: 0 });
        }

        // Check that column_index is within the table range
        let table_width = (table_range.end_column - table_range.start_column + 1) as u32;
        if column_index > table_width {
            return Err(PatternError::InvalidColumn {
                column: column_index as i32,
            });
        }

        Ok(())
    }

    /// Validate a lookup value
    fn validate_lookup_value(&self, lookup_value: &LookupValue) -> Result<(), PatternError> {
        match lookup_value {
            LookupValue::String(_) => Ok(()), // Always valid
            LookupValue::Number(n) => {
                // Check for NaN or infinite values
                if !n.is_finite() {
                    // For now, we allow NaN and infinite values as Excel does
                }
                Ok(())
            }
            LookupValue::CellReference(cell_ref) => {
                // Validate cell reference bounds
                const MAX_ROWS: i32 = 1_048_576; // Excel row limit
                const MAX_COLS: i32 = 16_384; // Excel column limit

                if cell_ref.row < 0 || cell_ref.row >= MAX_ROWS {
                    return Err(PatternError::CellOutOfBounds {
                        sheet: cell_ref.sheet,
                        row: cell_ref.row,
                        column: cell_ref.column,
                    });
                }

                if cell_ref.column < 0 || cell_ref.column >= MAX_COLS {
                    return Err(PatternError::CellOutOfBounds {
                        sheet: cell_ref.sheet,
                        row: cell_ref.row,
                        column: cell_ref.column,
                    });
                }

                Ok(())
            }
        }
    }

    /// Validate a table range
    fn validate_table_range(&self, table_range: &TableRange) -> Result<(), PatternError> {
        const MAX_ROWS: i32 = 1_048_576; // Excel row limit
        const MAX_COLS: i32 = 16_384; // Excel column limit

        // Validate sheet name if present
        if let Some(sheet_name) = &table_range.sheet {
            if sheet_name.is_empty() || sheet_name.len() > 255 {
                return Err(PatternError::InvalidSheetName {
                    name: sheet_name.clone(),
                });
            }
        }

        // Validate start coordinates
        if table_range.start_row < 0 || table_range.start_row >= MAX_ROWS {
            return Err(PatternError::CellOutOfBounds {
                sheet: 0, // We don't have sheet index in table range
                row: table_range.start_row,
                column: table_range.start_column,
            });
        }

        if table_range.start_column < 0 || table_range.start_column >= MAX_COLS {
            return Err(PatternError::CellOutOfBounds {
                sheet: 0,
                row: table_range.start_row,
                column: table_range.start_column,
            });
        }

        // Validate end coordinates
        if table_range.end_row < 0 || table_range.end_row >= MAX_ROWS {
            return Err(PatternError::CellOutOfBounds {
                sheet: 0,
                row: table_range.end_row,
                column: table_range.end_column,
            });
        }

        if table_range.end_column < 0 || table_range.end_column >= MAX_COLS {
            return Err(PatternError::CellOutOfBounds {
                sheet: 0,
                row: table_range.end_row,
                column: table_range.end_column,
            });
        }

        // Validate that start <= end
        if table_range.start_row > table_range.end_row {
            return Err(PatternError::InvalidColumn { column: -1 }); // Using column as general validation error
        }

        if table_range.start_column > table_range.end_column {
            return Err(PatternError::InvalidColumn { column: -1 });
        }

        Ok(())
    }

    /// Extract a pattern from a parsed AST node
    ///
    /// Analyzes the AST structure to identify patterns that can be used for
    /// scalar substitution. Currently supports SUMIFS functions.
    ///
    /// # Arguments
    /// * `node` - The parsed AST node to analyze
    ///
    /// # Returns
    /// * `Some(PatternKey)` - If a supported pattern is found
    /// * `None` - If no supported pattern matches or the structure is invalid
    ///
    /// # Examples
    /// ```
    /// // For SUMIFS(D:D, F:F, "Collections Clinic", H:H, 1811349806)
    /// // Returns: PatternKey::Sumifs with appropriate criteria pairs
    /// ```
    pub fn from_node(node: &Node) -> Option<PatternKey> {
        match node {
            Node::FunctionKind {
                kind: Function::Sumifs,
                args,
            } => Self::extract_sumifs_pattern(args, None),
            Node::FunctionKind {
                kind: Function::Countifs,
                args,
            } => Self::extract_countifs_pattern(args, None),
            Node::FunctionKind {
                kind: Function::Averageifs,
                args,
            } => Self::extract_averageifs_pattern(args, None),
            Node::FunctionKind {
                kind: Function::Vlookup,
                args,
            } => Self::extract_vlookup_pattern(args, None),
            _ => None,
        }
    }

    pub fn from_node_with_context(
        node: &Node,
        cell: crate::expressions::types::CellReferenceIndex,
    ) -> Option<PatternKey> {
        match node {
            Node::FunctionKind {
                kind: Function::Sumifs,
                args,
            } => Self::extract_sumifs_pattern(args, Some(cell)),
            Node::FunctionKind {
                kind: Function::Countifs,
                args,
            } => Self::extract_countifs_pattern(args, Some(cell)),
            Node::FunctionKind {
                kind: Function::Averageifs,
                args,
            } => Self::extract_averageifs_pattern(args, Some(cell)),
            Node::FunctionKind {
                kind: Function::Vlookup,
                args,
            } => Self::extract_vlookup_pattern(args, Some(cell)),
            _ => None,
        }
    }

    /// Extract a pattern from a parsed AST node with detailed error reporting
    ///
    /// # Returns
    /// * `Ok(PatternKey)` - If a valid pattern is extracted
    /// * `Err(PatternError)` - If pattern extraction fails with details
    pub fn try_from_node(node: &Node) -> Result<PatternKey, PatternError> {
        match node {
            Node::FunctionKind {
                kind: Function::Sumifs,
                args,
            } => Self::try_extract_sumifs_pattern(args, None),
            Node::FunctionKind {
                kind: Function::Countifs,
                args,
            } => Self::try_extract_countifs_pattern(args, None),
            Node::FunctionKind {
                kind: Function::Averageifs,
                args,
            } => Self::try_extract_averageifs_pattern(args, None),
            Node::FunctionKind {
                kind: Function::Vlookup,
                args,
            } => Self::try_extract_vlookup_pattern(args, None),
            _ => Err(PatternError::UnsupportedNode {
                node_type: format!("{:?}", node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
            }),
        }
    }

    /// Extract a pattern from a parsed AST node with context and detailed error reporting
    pub fn try_from_node_with_context(
        node: &Node,
        cell: crate::expressions::types::CellReferenceIndex,
    ) -> Result<PatternKey, PatternError> {
        match node {
            Node::FunctionKind {
                kind: Function::Sumifs,
                args,
            } => Self::try_extract_sumifs_pattern(args, Some(cell)),
            Node::FunctionKind {
                kind: Function::Countifs,
                args,
            } => Self::try_extract_countifs_pattern(args, Some(cell)),
            Node::FunctionKind {
                kind: Function::Averageifs,
                args,
            } => Self::try_extract_averageifs_pattern(args, Some(cell)),
            Node::FunctionKind {
                kind: Function::Vlookup,
                args,
            } => Self::try_extract_vlookup_pattern(args, Some(cell)),
            _ => Err(PatternError::UnsupportedNode {
                node_type: format!("{:?}", node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
            }),
        }
    }

    /// Extract a SUMIFS pattern with detailed error reporting
    fn try_extract_sumifs_pattern(
        args: &[Node],
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Result<PatternKey, PatternError> {
        // SUMIFS(sum_range, criteria_range1, criteria1, [criteria_range2, criteria2, ...])
        // Minimum 3 arguments required
        if args.len() < 3 {
            return Err(PatternError::InvalidArgumentCount {
                expected: "at least 3".to_string(),
                found: args.len(),
            });
        }

        if (args.len() - 1) % 2 != 0 {
            return Err(PatternError::InvalidArgumentCount {
                expected: "odd number (3, 5, 7, ...)".to_string(),
                found: args.len(),
            });
        }

        // Extract sum range - must be a single column range
        let sum_column = Self::try_extract_single_column_from_range(&args[0], cell)?;

        // Extract sheet name from the sum range
        let sheet = Self::extract_sheet_name(&args[0]);

        // Extract criteria pairs
        let mut criteria_pairs = Vec::new();
        let mut i = 1;
        while i + 1 < args.len() {
            let criteria_range = &args[i];
            let criteria_value = &args[i + 1];

            // Extract column from criteria range
            let column = Self::try_extract_single_column_from_range(criteria_range, cell)?;

            // Extract criteria value
            let criteria = Self::try_extract_criteria_value(criteria_value)?;

            criteria_pairs.push(CriteriaPair { column, criteria });

            i += 2;
        }

        let pattern = PatternKey::Sumifs {
            sheet,
            sum_column,
            criteria_pairs,
        };

        // Validate the extracted pattern
        pattern.validate()?;

        Ok(pattern)
    }

    /// Extract a SUMIFS pattern from the function arguments
    fn extract_sumifs_pattern(
        args: &[Node],
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Option<PatternKey> {
        // SUMIFS(sum_range, criteria_range1, criteria1, [criteria_range2, criteria2, ...])
        // Minimum 3 arguments required
        if args.len() < 3 || (args.len() - 1) % 2 != 0 {
            return None;
        }

        // Extract sum range - must be a single column range
        let sum_column = Self::extract_single_column_from_range(&args[0], cell)?;

        // Extract sheet name from the sum range
        let sheet = Self::extract_sheet_name(&args[0]);

        // Extract criteria pairs
        let mut criteria_pairs = Vec::new();
        let mut i = 1;
        while i + 1 < args.len() {
            let criteria_range = &args[i];
            let criteria_value = &args[i + 1];

            // Extract column from criteria range
            let column = Self::extract_single_column_from_range(criteria_range, cell)?;

            // Extract criteria value
            let criteria = Self::extract_criteria_value(criteria_value)?;

            criteria_pairs.push(CriteriaPair { column, criteria });

            i += 2;
        }

        Some(PatternKey::Sumifs {
            sheet,
            sum_column,
            criteria_pairs,
        })
    }

    /// Extract a single column index from a range node
    /// Returns Some(column) if the range spans exactly one column, None otherwise
    /// Converts from AST 0-based columns to PatternKey 1-based columns
    /// If cell context is provided, converts relative references to absolute
    fn extract_single_column_from_range(
        node: &Node,
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Option<i32> {
        match node {
            Node::RangeKind {
                column1,
                column2,
                absolute_column1,
                ..
            } => {
                if column1 == column2 {
                    let abs_column = if *absolute_column1 {
                        // Already absolute
                        *column1
                    } else {
                        // Relative reference - convert to absolute using cell context
                        if let Some(cell_ref) = cell {
                            column1 + cell_ref.column
                        } else {
                            // Without context, assume the relative reference is from A1 (0,0)
                            // This is the common case for sheet references like "DataSheet!D:D"
                            *column1
                        }
                    };
                    Some(abs_column) // Convert from 0-based to 1-based
                } else {
                    None // Multi-column ranges not supported
                }
            }
            Node::ReferenceKind {
                column,
                absolute_column,
                ..
            } => {
                let abs_column = if *absolute_column {
                    // Already absolute
                    *column
                } else {
                    // Relative reference - convert to absolute using cell context
                    if let Some(cell_ref) = cell {
                        column + cell_ref.column
                    } else {
                        // Without context, assume the relative reference is from A1 (0,0)
                        // This is the common case for sheet references like "DataSheet!D:D"
                        *column
                    }
                };
                Some(abs_column) // Convert from 0-based to 1-based
            }
            _ => None,
        }
    }

    /// Extract a COUNTIFS pattern with detailed error reporting
    fn try_extract_countifs_pattern(
        args: &[Node],
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Result<PatternKey, PatternError> {
        // COUNTIFS(criteria_range1, criteria1, [criteria_range2, criteria2, ...])
        // Minimum 2 arguments required, and must be even number
        if args.len() < 2 {
            return Err(PatternError::InvalidArgumentCount {
                expected: "at least 2".to_string(),
                found: args.len(),
            });
        }

        if args.len() % 2 != 0 {
            return Err(PatternError::InvalidArgumentCount {
                expected: "even number (2, 4, 6, ...)".to_string(),
                found: args.len(),
            });
        }

        // Extract sheet name from the first criteria range
        let sheet = Self::extract_sheet_name(&args[0]);

        // Extract criteria pairs
        let mut criteria_pairs = Vec::new();
        let mut i = 0;
        while i + 1 < args.len() {
            let criteria_range = &args[i];
            let criteria_value = &args[i + 1];

            // Extract column from criteria range
            let column = Self::try_extract_single_column_from_range(criteria_range, cell)?;

            // Extract criteria value
            let criteria = Self::try_extract_criteria_value(criteria_value)?;

            criteria_pairs.push(CriteriaPair { column, criteria });

            i += 2;
        }

        let pattern = PatternKey::Countifs {
            sheet,
            criteria_pairs,
        };

        // Validate the extracted pattern
        pattern.validate()?;

        Ok(pattern)
    }

    /// Extract a COUNTIFS pattern from the function arguments
    fn extract_countifs_pattern(
        args: &[Node],
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Option<PatternKey> {
        // COUNTIFS(criteria_range1, criteria1, [criteria_range2, criteria2, ...])
        // Minimum 2 arguments required, and must be even number
        if args.len() < 2 || args.len() % 2 != 0 {
            return None;
        }

        // Extract sheet name from the first criteria range
        let sheet = Self::extract_sheet_name(&args[0]);

        // Extract criteria pairs
        let mut criteria_pairs = Vec::new();
        let mut i = 0;
        while i + 1 < args.len() {
            let criteria_range = &args[i];
            let criteria_value = &args[i + 1];

            // Extract column from criteria range
            let column = Self::extract_single_column_from_range(criteria_range, cell)?;

            // Extract criteria value
            let criteria = Self::extract_criteria_value(criteria_value)?;

            criteria_pairs.push(CriteriaPair { column, criteria });

            i += 2;
        }

        Some(PatternKey::Countifs {
            sheet,
            criteria_pairs,
        })
    }

    /// Extract an AVERAGEIFS pattern with detailed error reporting
    fn try_extract_averageifs_pattern(
        args: &[Node],
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Result<PatternKey, PatternError> {
        // AVERAGEIFS(average_range, criteria_range1, criteria1, [criteria_range2, criteria2, ...])
        // Minimum 3 arguments required
        if args.len() < 3 {
            return Err(PatternError::InvalidArgumentCount {
                expected: "at least 3".to_string(),
                found: args.len(),
            });
        }

        if (args.len() - 1) % 2 != 0 {
            return Err(PatternError::InvalidArgumentCount {
                expected: "odd number (3, 5, 7, ...)".to_string(),
                found: args.len(),
            });
        }

        // Extract average range - must be a single column range
        let average_column = Self::try_extract_single_column_from_range(&args[0], cell)?;

        // Extract sheet name from the average range
        let sheet = Self::extract_sheet_name(&args[0]);

        // Extract criteria pairs
        let mut criteria_pairs = Vec::new();
        let mut i = 1;
        while i + 1 < args.len() {
            let criteria_range = &args[i];
            let criteria_value = &args[i + 1];

            // Extract column from criteria range
            let column = Self::try_extract_single_column_from_range(criteria_range, cell)?;

            // Extract criteria value
            let criteria = Self::try_extract_criteria_value(criteria_value)?;

            criteria_pairs.push(CriteriaPair { column, criteria });

            i += 2;
        }

        let pattern = PatternKey::Averageifs {
            sheet,
            average_column,
            criteria_pairs,
        };

        // Validate the extracted pattern
        pattern.validate()?;

        Ok(pattern)
    }

    /// Extract an AVERAGEIFS pattern from the function arguments
    fn extract_averageifs_pattern(
        args: &[Node],
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Option<PatternKey> {
        // AVERAGEIFS(average_range, criteria_range1, criteria1, [criteria_range2, criteria2, ...])
        // Minimum 3 arguments required
        if args.len() < 3 || (args.len() - 1) % 2 != 0 {
            return None;
        }

        // Extract average range - must be a single column range
        let average_column = Self::extract_single_column_from_range(&args[0], cell)?;

        // Extract sheet name from the average range
        let sheet = Self::extract_sheet_name(&args[0]);

        // Extract criteria pairs
        let mut criteria_pairs = Vec::new();
        let mut i = 1;
        while i + 1 < args.len() {
            let criteria_range = &args[i];
            let criteria_value = &args[i + 1];

            // Extract column from criteria range
            let column = Self::extract_single_column_from_range(criteria_range, cell)?;

            // Extract criteria value
            let criteria = Self::extract_criteria_value(criteria_value)?;

            criteria_pairs.push(CriteriaPair { column, criteria });

            i += 2;
        }

        Some(PatternKey::Averageifs {
            sheet,
            average_column,
            criteria_pairs,
        })
    }

    /// Extract a VLOOKUP pattern with detailed error reporting
    fn try_extract_vlookup_pattern(
        args: &[Node],
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Result<PatternKey, PatternError> {
        // VLOOKUP(lookup_value, table_array, column_index_num, [range_lookup])
        // Minimum 3 arguments required, 4th is optional
        if args.len() < 3 || args.len() > 4 {
            return Err(PatternError::InvalidArgumentCount {
                expected: "3 or 4".to_string(),
                found: args.len(),
            });
        }

        // Extract lookup value
        let lookup_value = Self::try_extract_lookup_value(&args[0])?;

        // Extract table array (must be a range)
        let table_range = Self::try_extract_table_range(&args[1], cell)?;

        // Extract column index (must be a positive integer)
        let column_index = Self::try_extract_column_index(&args[2])?;

        // Extract exact_match flag (optional, defaults to true for approximate match)
        let exact_match = if args.len() > 3 {
            !Self::try_extract_boolean(&args[3])? // VLOOKUP uses TRUE for approximate, we store exact_match
        } else {
            false // Default is approximate match (range_lookup = TRUE)
        };

        // Extract sheet name from the table range
        let sheet = table_range.sheet.clone();

        let pattern = PatternKey::Vlookup {
            sheet,
            lookup_value,
            table_range,
            column_index,
            exact_match,
        };

        // Validate the extracted pattern
        pattern.validate()?;

        Ok(pattern)
    }

    /// Extract a VLOOKUP pattern from the function arguments
    fn extract_vlookup_pattern(
        args: &[Node],
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Option<PatternKey> {
        // VLOOKUP(lookup_value, table_array, column_index_num, [range_lookup])
        if args.len() < 3 || args.len() > 4 {
            return None;
        }

        // Extract lookup value
        let lookup_value = Self::extract_lookup_value(&args[0])?;

        // Extract table array (must be a range)
        let table_range = Self::extract_table_range(&args[1], cell)?;

        // Extract column index (must be a positive integer)
        let column_index = Self::extract_column_index(&args[2])?;

        // Extract exact_match flag (optional, defaults to true for approximate match)
        let exact_match = if args.len() > 3 {
            !Self::extract_boolean(&args[3])? // VLOOKUP uses TRUE for approximate, we store exact_match
        } else {
            false // Default is approximate match (range_lookup = TRUE)
        };

        // Extract sheet name from the table range
        let sheet = table_range.sheet.clone();

        Some(PatternKey::Vlookup {
            sheet,
            lookup_value,
            table_range,
            column_index,
            exact_match,
        })
    }

    /// Extract a lookup value from a node
    fn extract_lookup_value(node: &Node) -> Option<LookupValue> {
        match node {
            Node::StringKind(s) => Some(LookupValue::String(s.clone())),
            Node::NumberKind(n) => Some(LookupValue::Number(OrderedFloat(*n))),
            Node::ReferenceKind {
                sheet_index,
                row,
                column,
                ..
            } => Some(LookupValue::CellReference(CellReferenceIndex {
                sheet: *sheet_index,
                row: *row,
                column: *column,
            })),
            _ => None,
        }
    }

    /// Extract a lookup value with detailed error reporting
    fn try_extract_lookup_value(node: &Node) -> Result<LookupValue, PatternError> {
        match node {
            Node::StringKind(s) => Ok(LookupValue::String(s.clone())),
            Node::NumberKind(n) => {
                if !n.is_finite() {
                    // We allow infinite/NaN values but could add validation here
                }
                Ok(LookupValue::Number(OrderedFloat(*n)))
            }
            Node::ReferenceKind {
                sheet_index,
                row,
                column,
                ..
            } => {
                let cell_ref = CellReferenceIndex {
                    sheet: *sheet_index,
                    row: *row,
                    column: *column,
                };

                let lookup_value = LookupValue::CellReference(cell_ref);
                // Use a dummy pattern to validate the lookup value
                let dummy_pattern = PatternKey::Vlookup {
                    sheet: None,
                    lookup_value: lookup_value.clone(),
                    table_range: TableRange {
                        sheet: None,
                        start_row: 1,
                        start_column: 1,
                        end_row: 1,
                        end_column: 1,
                    },
                    column_index: 1,
                    exact_match: true,
                };
                dummy_pattern.validate_lookup_value(&lookup_value)?;

                Ok(lookup_value)
            }
            _ => Err(PatternError::UnsupportedNode {
                node_type: format!("{:?}", node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
            }),
        }
    }

    /// Extract a table range from a range node
    fn extract_table_range(
        node: &Node,
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Option<TableRange> {
        match node {
            Node::RangeKind {
                sheet_name,
                row1,
                column1,
                row2,
                column2,
                absolute_row1,
                absolute_column1,
                absolute_row2,
                absolute_column2,
                ..
            } => {
                // Convert relative references to absolute if we have context
                let start_row = if *absolute_row1 {
                    *row1
                } else if let Some(cell_ref) = cell {
                    row1 + cell_ref.row
                } else {
                    return None;
                };

                let start_column = if *absolute_column1 {
                    *column1
                } else if let Some(cell_ref) = cell {
                    column1 + cell_ref.column
                } else {
                    return None;
                };

                let end_row = if *absolute_row2 {
                    *row2
                } else if let Some(cell_ref) = cell {
                    row2 + cell_ref.row
                } else {
                    return None;
                };

                let end_column = if *absolute_column2 {
                    *column2
                } else if let Some(cell_ref) = cell {
                    column2 + cell_ref.column
                } else {
                    return None;
                };

                Some(TableRange {
                    sheet: sheet_name.clone(),
                    start_row,
                    start_column,
                    end_row,
                    end_column,
                })
            }
            _ => None,
        }
    }

    /// Extract a table range with detailed error reporting
    fn try_extract_table_range(
        node: &Node,
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Result<TableRange, PatternError> {
        match node {
            Node::RangeKind {
                sheet_name,
                row1,
                column1,
                row2,
                column2,
                absolute_row1,
                absolute_column1,
                absolute_row2,
                absolute_column2,
                ..
            } => {
                // Convert relative references to absolute if we have context
                let start_row = if *absolute_row1 {
                    *row1
                } else if let Some(cell_ref) = cell {
                    row1 + cell_ref.row
                } else {
                    return Err(PatternError::UnsupportedNode {
                        node_type: "relative reference without context".to_string(),
                    });
                };

                let start_column = if *absolute_column1 {
                    *column1
                } else if let Some(cell_ref) = cell {
                    column1 + cell_ref.column
                } else {
                    return Err(PatternError::UnsupportedNode {
                        node_type: "relative reference without context".to_string(),
                    });
                };

                let end_row = if *absolute_row2 {
                    *row2
                } else if let Some(cell_ref) = cell {
                    row2 + cell_ref.row
                } else {
                    return Err(PatternError::UnsupportedNode {
                        node_type: "relative reference without context".to_string(),
                    });
                };

                let end_column = if *absolute_column2 {
                    *column2
                } else if let Some(cell_ref) = cell {
                    column2 + cell_ref.column
                } else {
                    return Err(PatternError::UnsupportedNode {
                        node_type: "relative reference without context".to_string(),
                    });
                };

                let table_range = TableRange {
                    sheet: sheet_name.clone(),
                    start_row,
                    start_column,
                    end_row,
                    end_column,
                };

                // Use a dummy pattern to validate the table range
                let dummy_pattern = PatternKey::Vlookup {
                    sheet: None,
                    lookup_value: LookupValue::String("dummy".to_string()),
                    table_range: table_range.clone(),
                    column_index: 1,
                    exact_match: true,
                };
                dummy_pattern.validate_table_range(&table_range)?;

                Ok(table_range)
            }
            _ => Err(PatternError::UnsupportedNode {
                node_type: format!("{:?}", node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
            }),
        }
    }

    /// Extract a column index from a node
    fn extract_column_index(node: &Node) -> Option<u32> {
        match node {
            Node::NumberKind(n) => {
                if n.fract() == 0.0 && *n > 0.0 && *n <= u32::MAX as f64 {
                    Some(*n as u32)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Extract a column index with detailed error reporting
    fn try_extract_column_index(node: &Node) -> Result<u32, PatternError> {
        match node {
            Node::NumberKind(n) => {
                if n.fract() != 0.0 {
                    return Err(PatternError::InvalidColumn { column: *n as i32 });
                }
                if *n <= 0.0 {
                    return Err(PatternError::InvalidColumn { column: *n as i32 });
                }
                if *n > u32::MAX as f64 {
                    return Err(PatternError::InvalidColumn {
                        column: u32::MAX as i32,
                    });
                }
                Ok(*n as u32)
            }
            _ => Err(PatternError::UnsupportedNode {
                node_type: format!("{:?}", node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
            }),
        }
    }

    /// Extract a boolean value from a node
    fn extract_boolean(node: &Node) -> Option<bool> {
        match node {
            Node::BooleanKind(b) => Some(*b),
            Node::NumberKind(n) => Some(*n != 0.0),
            _ => None,
        }
    }

    /// Extract a boolean value with detailed error reporting
    fn try_extract_boolean(node: &Node) -> Result<bool, PatternError> {
        match node {
            Node::BooleanKind(b) => Ok(*b),
            Node::NumberKind(n) => Ok(*n != 0.0),
            _ => Err(PatternError::UnsupportedNode {
                node_type: format!("{:?}", node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
            }),
        }
    }

    /// Extract the sheet name from a range or reference node
    fn extract_sheet_name(node: &Node) -> Option<String> {
        match node {
            Node::RangeKind { sheet_name, .. } => sheet_name.clone(),
            Node::ReferenceKind { sheet_name, .. } => sheet_name.clone(),
            _ => None,
        }
    }

    /// Extract a criteria value from a node
    fn extract_criteria_value(node: &Node) -> Option<CriteriaValue> {
        match node {
            Node::StringKind(s) => Some(CriteriaValue::String(s.clone())),
            Node::NumberKind(n) => Some(CriteriaValue::Number(OrderedFloat(*n))),
            Node::ReferenceKind {
                sheet_index,
                row,
                column,
                ..
            } => Some(CriteriaValue::CellReference(CellReferenceIndex {
                sheet: *sheet_index,
                row: *row,
                column: *column,
            })),
            _ => None,
        }
    }

    /// Enhanced column extraction with detailed error reporting
    fn try_extract_single_column_from_range(
        node: &Node,
        cell: Option<crate::expressions::types::CellReferenceIndex>,
    ) -> Result<i32, PatternError> {
        match node {
            Node::RangeKind {
                column1,
                column2,
                absolute_column1,
                ..
            } => {
                if column1 != column2 {
                    return Err(PatternError::MultiColumnRange {
                        start: *column1,
                        end: *column2,
                    });
                }

                let abs_column = if *absolute_column1 {
                    // Already absolute
                    *column1
                } else {
                    // Relative reference - convert to absolute using cell context
                    if let Some(cell_ref) = cell {
                        column1 + cell_ref.column
                    } else {
                        return Err(PatternError::UnsupportedNode {
                            node_type: "relative reference without context".to_string(),
                        });
                    }
                };

                if abs_column < 0 {
                    return Err(PatternError::InvalidColumn { column: abs_column });
                }

                Ok(abs_column)
            }
            Node::ReferenceKind {
                column,
                absolute_column,
                ..
            } => {
                let abs_column = if *absolute_column {
                    // Already absolute
                    *column
                } else {
                    // Relative reference - convert to absolute using cell context
                    if let Some(cell_ref) = cell {
                        column + cell_ref.column
                    } else {
                        return Err(PatternError::UnsupportedNode {
                            node_type: "relative reference without context".to_string(),
                        });
                    }
                };

                if abs_column < 0 {
                    return Err(PatternError::InvalidColumn { column: abs_column });
                }

                Ok(abs_column)
            }
            _ => Err(PatternError::UnsupportedNode {
                node_type: format!("{:?}", node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
            }),
        }
    }

    /// Enhanced criteria value extraction with detailed error reporting
    fn try_extract_criteria_value(node: &Node) -> Result<CriteriaValue, PatternError> {
        match node {
            Node::StringKind(s) => Ok(CriteriaValue::String(s.clone())),
            Node::NumberKind(n) => {
                if !n.is_finite() {
                    // We allow infinite/NaN values but could add validation here
                }
                Ok(CriteriaValue::Number(OrderedFloat(*n)))
            }
            Node::ReferenceKind {
                sheet_index,
                row,
                column,
                ..
            } => {
                let cell_ref = CellReferenceIndex {
                    sheet: *sheet_index,
                    row: *row,
                    column: *column,
                };

                // Validate the cell reference
                let criteria_value = CriteriaValue::CellReference(cell_ref);
                // Use a dummy pattern to validate the criteria value
                let dummy_pattern = PatternKey::Sumifs {
                    sheet: None,
                    sum_column: 0,
                    criteria_pairs: vec![CriteriaPair {
                        column: 0,
                        criteria: criteria_value.clone(),
                    }],
                };
                dummy_pattern.validate_criteria_value(&criteria_value)?;

                Ok(criteria_value)
            }
            _ => Err(PatternError::UnsupportedNode {
                node_type: format!("{:?}", node)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string(),
            }),
        }
    }
}

/// Builder pattern for constructing SUMIFS patterns ergonomically.
///
/// Provides a fluent interface for building complex patterns:
///
/// ```rust
/// let pattern = PatternKey::sumifs()
///     .sheet("DataSheet")
///     .sum_column(4)
///     .criteria(6, CriteriaValue::String("Active".to_string()))
///     .criteria(8, CriteriaValue::Number(OrderedFloat(100.0)))
///     .build()
///     .unwrap();
/// ```
#[derive(Default)]
pub struct SumifsBuilder {
    sheet: Option<String>,
    sum_column: Option<i32>,
    criteria_pairs: Vec<CriteriaPair>,
}

impl SumifsBuilder {
    /// Create a new SUMIFS pattern builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sheet name for all ranges in this pattern
    ///
    /// # Arguments
    /// * `sheet` - Sheet name, or anything that can be converted to String
    pub fn sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheet = Some(sheet.into());
        self
    }

    /// Set the column for the sum range
    ///
    /// # Arguments
    /// * `column` - 1-based column index (1=A, 2=B, 3=C, 4=D, etc.)
    pub fn sum_column(mut self, column: i32) -> Self {
        self.sum_column = Some(column);
        self
    }

    /// Add a criteria pair to the pattern
    ///
    /// # Arguments
    /// * `column` - 1-based column index for the criteria range
    /// * `criteria` - The criteria value to match
    pub fn criteria(mut self, column: i32, criteria: CriteriaValue) -> Self {
        self.criteria_pairs.push(CriteriaPair { column, criteria });
        self
    }

    /// Add a string criteria pair
    ///
    /// Convenience method for the common case of string criteria
    pub fn criteria_string(self, column: i32, criteria: impl Into<String>) -> Self {
        self.criteria(column, CriteriaValue::String(criteria.into()))
    }

    /// Add a numeric criteria pair
    ///
    /// Convenience method for numeric criteria
    pub fn criteria_number(self, column: i32, criteria: f64) -> Self {
        self.criteria(column, CriteriaValue::Number(OrderedFloat(criteria)))
    }

    /// Add a cell reference criteria pair
    ///
    /// Convenience method for cell reference criteria
    pub fn criteria_cell(self, column: i32, cell_ref: CellReferenceIndex) -> Self {
        self.criteria(column, CriteriaValue::CellReference(cell_ref))
    }

    /// Build the final PatternKey
    ///
    /// Returns None if required fields are missing (sum_column is required)
    pub fn build(self) -> Option<PatternKey> {
        Some(PatternKey::Sumifs {
            sheet: self.sheet,
            sum_column: self.sum_column?,
            criteria_pairs: self.criteria_pairs,
        })
    }
}

/// Builder pattern for constructing COUNTIFS patterns ergonomically.
///
/// Provides a fluent interface for building complex patterns:
///
/// ```rust
/// let pattern = PatternKey::countifs()
///     .sheet("DataSheet")
///     .criteria_string(6, "Active")
///     .criteria_number(8, 100.0)
///     .build()
///     .unwrap();
/// ```
#[derive(Default)]
pub struct CountifsBuilder {
    sheet: Option<String>,
    criteria_pairs: Vec<CriteriaPair>,
}

impl CountifsBuilder {
    /// Create a new COUNTIFS pattern builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sheet name for all ranges in this pattern
    ///
    /// # Arguments
    /// * `sheet` - Sheet name, or anything that can be converted to String
    pub fn sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheet = Some(sheet.into());
        self
    }

    /// Add a criteria pair to the pattern
    ///
    /// # Arguments
    /// * `column` - 1-based column index for the criteria range
    /// * `criteria` - The criteria value to match
    pub fn criteria(mut self, column: i32, criteria: CriteriaValue) -> Self {
        self.criteria_pairs.push(CriteriaPair { column, criteria });
        self
    }

    /// Add a string criteria pair
    ///
    /// Convenience method for the common case of string criteria
    pub fn criteria_string(self, column: i32, criteria: impl Into<String>) -> Self {
        self.criteria(column, CriteriaValue::String(criteria.into()))
    }

    /// Add a numeric criteria pair
    ///
    /// Convenience method for numeric criteria
    pub fn criteria_number(self, column: i32, criteria: f64) -> Self {
        self.criteria(column, CriteriaValue::Number(OrderedFloat(criteria)))
    }

    /// Add a cell reference criteria pair
    ///
    /// Convenience method for cell reference criteria
    pub fn criteria_cell(self, column: i32, cell_ref: CellReferenceIndex) -> Self {
        self.criteria(column, CriteriaValue::CellReference(cell_ref))
    }

    /// Build the final PatternKey
    ///
    /// Returns None if no criteria pairs are provided
    pub fn build(self) -> Option<PatternKey> {
        if self.criteria_pairs.is_empty() {
            return None;
        }
        Some(PatternKey::Countifs {
            sheet: self.sheet,
            criteria_pairs: self.criteria_pairs,
        })
    }
}

/// Builder pattern for constructing AVERAGEIFS patterns ergonomically.
///
/// Provides a fluent interface for building complex patterns:
///
/// ```rust
/// let pattern = PatternKey::averageifs()
///     .sheet("DataSheet")
///     .average_column(4)
///     .criteria_string(6, "Active")
///     .criteria_number(8, 100.0)
///     .build()
///     .unwrap();
/// ```
#[derive(Default)]
pub struct AverageifsBuilder {
    sheet: Option<String>,
    average_column: Option<i32>,
    criteria_pairs: Vec<CriteriaPair>,
}

impl AverageifsBuilder {
    /// Create a new AVERAGEIFS pattern builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sheet name for all ranges in this pattern
    ///
    /// # Arguments
    /// * `sheet` - Sheet name, or anything that can be converted to String
    pub fn sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheet = Some(sheet.into());
        self
    }

    /// Set the column for the average range
    ///
    /// # Arguments
    /// * `column` - 1-based column index (1=A, 2=B, 3=C, 4=D, etc.)
    pub fn average_column(mut self, column: i32) -> Self {
        self.average_column = Some(column);
        self
    }

    /// Add a criteria pair to the pattern
    ///
    /// # Arguments
    /// * `column` - 1-based column index for the criteria range
    /// * `criteria` - The criteria value to match
    pub fn criteria(mut self, column: i32, criteria: CriteriaValue) -> Self {
        self.criteria_pairs.push(CriteriaPair { column, criteria });
        self
    }

    /// Add a string criteria pair
    ///
    /// Convenience method for the common case of string criteria
    pub fn criteria_string(self, column: i32, criteria: impl Into<String>) -> Self {
        self.criteria(column, CriteriaValue::String(criteria.into()))
    }

    /// Add a numeric criteria pair
    ///
    /// Convenience method for numeric criteria
    pub fn criteria_number(self, column: i32, criteria: f64) -> Self {
        self.criteria(column, CriteriaValue::Number(OrderedFloat(criteria)))
    }

    /// Add a cell reference criteria pair
    ///
    /// Convenience method for cell reference criteria
    pub fn criteria_cell(self, column: i32, cell_ref: CellReferenceIndex) -> Self {
        self.criteria(column, CriteriaValue::CellReference(cell_ref))
    }

    /// Build the final PatternKey
    ///
    /// Returns None if required fields are missing (average_column is required)
    pub fn build(self) -> Option<PatternKey> {
        Some(PatternKey::Averageifs {
            sheet: self.sheet,
            average_column: self.average_column?,
            criteria_pairs: self.criteria_pairs,
        })
    }
}

/// Builder pattern for constructing VLOOKUP patterns ergonomically.
///
/// Provides a fluent interface for building VLOOKUP patterns:
///
/// ```rust
/// let pattern = PatternKey::vlookup()
///     .sheet("ProductData")
///     .lookup_string("Widget A")
///     .table_range(1, 1, 100, 5)
///     .column_index(3)
///     .exact_match(true)
///     .build()
///     .unwrap();
/// ```
#[derive(Default)]
pub struct VlookupBuilder {
    sheet: Option<String>,
    lookup_value: Option<LookupValue>,
    table_range: Option<TableRange>,
    column_index: Option<u32>,
    exact_match: bool,
}

impl VlookupBuilder {
    /// Create a new VLOOKUP pattern builder
    pub fn new() -> Self {
        Self {
            exact_match: false, // Default to approximate match (Excel default)
            ..Default::default()
        }
    }

    /// Set the sheet name for the table range
    ///
    /// # Arguments
    /// * `sheet` - Sheet name, or anything that can be converted to String
    pub fn sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheet = Some(sheet.into());
        self
    }

    /// Set the lookup value using a LookupValue enum
    ///
    /// # Arguments
    /// * `lookup_value` - The value to search for
    pub fn lookup_value(mut self, lookup_value: LookupValue) -> Self {
        self.lookup_value = Some(lookup_value);
        self
    }

    /// Set the lookup value to a string
    ///
    /// Convenience method for string lookup values
    pub fn lookup_string(self, value: impl Into<String>) -> Self {
        self.lookup_value(LookupValue::String(value.into()))
    }

    /// Set the lookup value to a number
    ///
    /// Convenience method for numeric lookup values
    pub fn lookup_number(self, value: f64) -> Self {
        self.lookup_value(LookupValue::Number(OrderedFloat(value)))
    }

    /// Set the lookup value to a cell reference
    ///
    /// Convenience method for cell reference lookup values
    pub fn lookup_cell_ref(self, sheet: u32, row: i32, column: i32) -> Self {
        self.lookup_value(LookupValue::CellReference(CellReferenceIndex {
            sheet,
            row,
            column,
        }))
    }

    /// Set the table range using a TableRange struct
    ///
    /// # Arguments
    /// * `table_range` - The table range to search in
    pub fn table(mut self, table_range: TableRange) -> Self {
        self.table_range = Some(table_range);
        self
    }

    /// Set the table range using individual coordinates
    ///
    /// # Arguments
    /// * `start_row` - Starting row (1-based)
    /// * `start_column` - Starting column (1-based)
    /// * `end_row` - Ending row (1-based)
    /// * `end_column` - Ending column (1-based)
    pub fn table_range(
        mut self,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> Self {
        self.table_range = Some(TableRange {
            sheet: self.sheet.clone(),
            start_row,
            start_column,
            end_row,
            end_column,
        });
        self
    }

    /// Set the table range with a specific sheet
    ///
    /// # Arguments
    /// * `sheet` - Sheet name for the table
    /// * `start_row` - Starting row (1-based)
    /// * `start_column` - Starting column (1-based)
    /// * `end_row` - Ending row (1-based)
    /// * `end_column` - Ending column (1-based)
    pub fn table_range_with_sheet(
        mut self,
        sheet: impl Into<String>,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> Self {
        let sheet_name = Some(sheet.into());
        self.table_range = Some(TableRange {
            sheet: sheet_name,
            start_row,
            start_column,
            end_row,
            end_column,
        });
        self
    }

    /// Set the column index to return from the table
    ///
    /// # Arguments
    /// * `column_index` - 1-based column index within the table (1=first column, 2=second column, etc.)
    pub fn column_index(mut self, column_index: u32) -> Self {
        self.column_index = Some(column_index);
        self
    }

    /// Set whether to use exact match
    ///
    /// # Arguments
    /// * `exact_match` - true for exact match, false for approximate match (Excel default)
    pub fn exact_match(mut self, exact_match: bool) -> Self {
        self.exact_match = exact_match;
        self
    }

    /// Build the final PatternKey
    ///
    /// Returns None if required fields are missing (lookup_value, table_range, and column_index are required)
    pub fn build(self) -> Option<PatternKey> {
        Some(PatternKey::Vlookup {
            sheet: self.sheet,
            lookup_value: self.lookup_value?,
            table_range: self.table_range?,
            column_index: self.column_index?,
            exact_match: self.exact_match,
        })
    }
}

impl CriteriaValue {
    /// Create a string criteria value
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Create a numeric criteria value
    pub fn number(value: f64) -> Self {
        Self::Number(OrderedFloat(value))
    }

    /// Create a cell reference criteria value
    pub fn cell_ref(sheet: u32, row: i32, column: i32) -> Self {
        Self::CellReference(CellReferenceIndex { sheet, row, column })
    }
}

/// Registry for managing scalar substitutions for pattern-matched formulas.
///
/// The SubstitutionRegistry provides a centralized store for mapping PatternKeys
/// to scalar values. It supports activation/deactivation, bulk operations, and
/// provides statistics about registered substitutions.
///
/// # Thread Safety
///
/// SubstitutionRegistry is not thread-safe by default. For multi-threaded scenarios,
/// wrap it in appropriate synchronization primitives:
///
/// ```rust,ignore
/// use std::sync::{Arc, Mutex};
/// let registry = Arc::new(Mutex::new(SubstitutionRegistry::new()));
/// ```
///
/// For high-performance concurrent read scenarios, consider using `Arc<RwLock<SubstitutionRegistry>>`
/// to allow multiple concurrent readers while ensuring exclusive write access.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use crate::expressions::pattern_matching::*;
///
/// let mut registry = SubstitutionRegistry::new();
///
/// // Add a substitution
/// let pattern = PatternKey::sumifs()
///     .sheet("DataSheet")
///     .sum_column(4)
///     .criteria_string(6, "Active")
///     .build()
///     .unwrap();
///
/// registry.add_substitution(pattern.clone(), 42000.0);
///
/// // Query the substitution
/// assert_eq!(registry.get_substitution(&pattern), Some(&42000.0));
///
/// // Deactivate to disable lookups
/// registry.set_active(false);
/// assert_eq!(registry.get_substitution(&pattern), None);
/// ```
#[derive(Debug, Clone)]
pub struct SubstitutionRegistry {
    /// The main storage for pattern-to-value mappings
    substitutions: std::collections::HashMap<PatternKey, f64>,
    /// Whether the registry is currently active for lookups
    active: bool,
    /// Statistics about registry usage
    stats: RegistryStats,
}

/// Statistics and metrics for the SubstitutionRegistry
#[derive(Debug, Clone, Default)]
pub struct RegistryStats {
    /// Total number of successful lookups
    pub successful_lookups: u64,
    /// Total number of failed lookups (key not found)
    pub failed_lookups: u64,
    /// Total number of lookups attempted while inactive
    pub inactive_lookups: u64,
    /// Number of substitutions added
    pub additions: u64,
    /// Number of substitutions removed
    pub removals: u64,
}

impl SubstitutionRegistry {
    /// Create a new, active SubstitutionRegistry
    pub fn new() -> Self {
        Self {
            substitutions: std::collections::HashMap::new(),
            active: true,
            stats: RegistryStats::default(),
        }
    }

    /// Create a new SubstitutionRegistry with specified initial capacity
    ///
    /// This can improve performance when you know approximately how many
    /// substitutions will be stored.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            substitutions: std::collections::HashMap::with_capacity(capacity),
            active: true,
            stats: RegistryStats::default(),
        }
    }

    /// Set whether the registry is active for lookups
    ///
    /// When inactive, all lookup operations return None, but the data remains
    /// stored and can be reactivated later.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Check if the registry is currently active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Add a substitution to the registry
    ///
    /// # Arguments
    /// * `pattern` - The pattern key to substitute
    /// * `value` - The scalar value to use as replacement
    ///
    /// # Returns
    /// The previous value if the pattern was already present, None otherwise
    pub fn add_substitution(&mut self, pattern: PatternKey, value: f64) -> Option<f64> {
        self.stats.additions += 1;
        self.substitutions.insert(pattern, value)
    }

    /// Remove a substitution from the registry
    ///
    /// # Arguments
    /// * `pattern` - The pattern key to remove
    ///
    /// # Returns
    /// The removed value if the pattern was present, None otherwise
    pub fn remove_substitution(&mut self, pattern: &PatternKey) -> Option<f64> {
        if let Some(value) = self.substitutions.remove(pattern) {
            self.stats.removals += 1;
            Some(value)
        } else {
            None
        }
    }

    /// Get a substitution value for a pattern
    ///
    /// Returns None if the registry is inactive, the pattern is not found,
    /// or any other lookup failure.
    ///
    /// # Arguments
    /// * `pattern` - The pattern key to look up
    ///
    /// # Returns
    /// The scalar value if found and registry is active, None otherwise
    pub fn get_substitution(&mut self, pattern: &PatternKey) -> Option<&f64> {
        if !self.active {
            self.stats.inactive_lookups += 1;
            return None;
        }

        match self.substitutions.get(pattern) {
            Some(value) => {
                self.stats.successful_lookups += 1;
                Some(value)
            }
            None => {
                self.stats.failed_lookups += 1;
                None
            }
        }
    }

    /// Check if a pattern has a substitution (regardless of active status)
    pub fn contains_pattern(&self, pattern: &PatternKey) -> bool {
        self.substitutions.contains_key(pattern)
    }

    /// Get the number of substitutions in the registry
    pub fn len(&self) -> usize {
        self.substitutions.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.substitutions.is_empty()
    }

    /// Clear all substitutions from the registry
    pub fn clear(&mut self) {
        let removed_count = self.substitutions.len() as u64;
        self.substitutions.clear();
        self.stats.removals += removed_count;
    }

    /// Add multiple substitutions at once
    ///
    /// # Arguments
    /// * `substitutions` - Iterator of (PatternKey, f64) pairs
    ///
    /// # Returns
    /// Number of substitutions actually added
    pub fn add_bulk<I>(&mut self, substitutions: I) -> usize
    where
        I: IntoIterator<Item = (PatternKey, f64)>,
    {
        let mut count = 0;
        for (pattern, value) in substitutions {
            self.add_substitution(pattern, value);
            count += 1;
        }
        count
    }

    /// Get all patterns currently in the registry
    pub fn patterns(&self) -> impl Iterator<Item = &PatternKey> {
        self.substitutions.keys()
    }

    /// Get all values currently in the registry
    pub fn values(&self) -> impl Iterator<Item = &f64> {
        self.substitutions.values()
    }

    /// Get all pattern-value pairs currently in the registry
    pub fn entries(&self) -> impl Iterator<Item = (&PatternKey, &f64)> {
        self.substitutions.iter()
    }

    /// Get the current statistics for this registry
    pub fn stats(&self) -> &RegistryStats {
        &self.stats
    }

    /// Reset all statistics to zero
    pub fn reset_stats(&mut self) {
        self.stats = RegistryStats::default();
    }

    /// Get memory usage estimate in bytes
    ///
    /// This is an approximation based on the HashMap capacity and
    /// the estimated size of stored keys and values.
    pub fn memory_usage_estimate(&self) -> usize {
        // Rough estimate: HashMap overhead + (PatternKey + f64) * count
        // PatternKey size varies but we estimate ~100 bytes average
        // f64 is 8 bytes
        // HashMap overhead is approximately capacity * 16 bytes
        let capacity = self.substitutions.capacity();
        let overhead = capacity * 16;
        let data_size = self.substitutions.len() * (100 + 8);
        overhead + data_size
    }

    /// Check for potential conflicts when adding a new substitution
    ///
    /// Identifies patterns that might conflict due to overlapping criteria or
    /// ambiguous matching conditions.
    ///
    /// # Returns
    /// * `Vec<PatternConflict>` - List of detected conflicts
    pub fn check_conflicts(
        &self,
        new_pattern: &PatternKey,
        new_value: f64,
    ) -> Vec<PatternConflict> {
        let mut conflicts = Vec::new();

        for (existing_pattern, existing_value) in &self.substitutions {
            if let Some(conflict) = self.detect_pattern_conflict(
                existing_pattern,
                *existing_value,
                new_pattern,
                new_value,
            ) {
                conflicts.push(conflict);
            }
        }

        conflicts
    }

    /// Detect conflict between two patterns
    fn detect_pattern_conflict(
        &self,
        pattern1: &PatternKey,
        value1: f64,
        pattern2: &PatternKey,
        value2: f64,
    ) -> Option<PatternConflict> {
        match (pattern1, pattern2) {
            (
                PatternKey::Sumifs {
                    sheet: sheet1,
                    sum_column: sum1,
                    criteria_pairs: criteria1,
                },
                PatternKey::Sumifs {
                    sheet: sheet2,
                    sum_column: sum2,
                    criteria_pairs: criteria2,
                },
            ) => {
                // Check for exact duplicate
                if pattern1 == pattern2 {
                    if (value1 - value2).abs() < f64::EPSILON {
                        return Some(PatternConflict::ExactDuplicate {
                            pattern: pattern1.clone(),
                            value: value1,
                        });
                    } else {
                        return Some(PatternConflict::SamePatternDifferentValues {
                            pattern: pattern1.clone(),
                            value1,
                            value2,
                        });
                    }
                }

                // Check for subset/superset conflicts
                if sheet1 == sheet2 && sum1 == sum2 {
                    if self.is_criteria_subset(criteria1, criteria2) {
                        return Some(PatternConflict::SubsetSuperset {
                            subset_pattern: pattern1.clone(),
                            subset_value: value1,
                            superset_pattern: pattern2.clone(),
                            superset_value: value2,
                        });
                    } else if self.is_criteria_subset(criteria2, criteria1) {
                        return Some(PatternConflict::SubsetSuperset {
                            subset_pattern: pattern2.clone(),
                            subset_value: value2,
                            superset_pattern: pattern1.clone(),
                            superset_value: value1,
                        });
                    }
                }

                None
            }
            (
                PatternKey::Countifs {
                    sheet: sheet1,
                    criteria_pairs: criteria1,
                },
                PatternKey::Countifs {
                    sheet: sheet2,
                    criteria_pairs: criteria2,
                },
            ) => {
                // Check for exact duplicate
                if pattern1 == pattern2 {
                    if (value1 - value2).abs() < f64::EPSILON {
                        return Some(PatternConflict::ExactDuplicate {
                            pattern: pattern1.clone(),
                            value: value1,
                        });
                    } else {
                        return Some(PatternConflict::SamePatternDifferentValues {
                            pattern: pattern1.clone(),
                            value1,
                            value2,
                        });
                    }
                }

                // Check for subset/superset conflicts
                if sheet1 == sheet2 {
                    if self.is_criteria_subset(criteria1, criteria2) {
                        return Some(PatternConflict::SubsetSuperset {
                            subset_pattern: pattern1.clone(),
                            subset_value: value1,
                            superset_pattern: pattern2.clone(),
                            superset_value: value2,
                        });
                    } else if self.is_criteria_subset(criteria2, criteria1) {
                        return Some(PatternConflict::SubsetSuperset {
                            subset_pattern: pattern2.clone(),
                            subset_value: value2,
                            superset_pattern: pattern1.clone(),
                            superset_value: value1,
                        });
                    }
                }

                None
            }
            (
                PatternKey::Averageifs {
                    sheet: sheet1,
                    average_column: avg1,
                    criteria_pairs: criteria1,
                },
                PatternKey::Averageifs {
                    sheet: sheet2,
                    average_column: avg2,
                    criteria_pairs: criteria2,
                },
            ) => {
                // Check for exact duplicate
                if pattern1 == pattern2 {
                    if (value1 - value2).abs() < f64::EPSILON {
                        return Some(PatternConflict::ExactDuplicate {
                            pattern: pattern1.clone(),
                            value: value1,
                        });
                    } else {
                        return Some(PatternConflict::SamePatternDifferentValues {
                            pattern: pattern1.clone(),
                            value1,
                            value2,
                        });
                    }
                }

                // Check for subset/superset conflicts
                if sheet1 == sheet2 && avg1 == avg2 {
                    if self.is_criteria_subset(criteria1, criteria2) {
                        return Some(PatternConflict::SubsetSuperset {
                            subset_pattern: pattern1.clone(),
                            subset_value: value1,
                            superset_pattern: pattern2.clone(),
                            superset_value: value2,
                        });
                    } else if self.is_criteria_subset(criteria2, criteria1) {
                        return Some(PatternConflict::SubsetSuperset {
                            subset_pattern: pattern2.clone(),
                            subset_value: value2,
                            superset_pattern: pattern1.clone(),
                            superset_value: value1,
                        });
                    }
                }

                None
            }
            (
                PatternKey::Vlookup {
                    sheet: _sheet1,
                    lookup_value: lookup1,
                    table_range: table1,
                    column_index: col1,
                    exact_match: exact1,
                },
                PatternKey::Vlookup {
                    sheet: _sheet2,
                    lookup_value: lookup2,
                    table_range: table2,
                    column_index: col2,
                    exact_match: exact2,
                },
            ) => {
                // Check for exact duplicate
                if pattern1 == pattern2 {
                    if (value1 - value2).abs() < f64::EPSILON {
                        return Some(PatternConflict::ExactDuplicate {
                            pattern: pattern1.clone(),
                            value: value1,
                        });
                    } else {
                        return Some(PatternConflict::SamePatternDifferentValues {
                            pattern: pattern1.clone(),
                            value1,
                            value2,
                        });
                    }
                }

                // VLOOKUP patterns can have conflicts if they have same lookup value,
                // overlapping table ranges, and same column index
                if lookup1 == lookup2 && col1 == col2 && exact1 == exact2 {
                    // Check if table ranges overlap
                    if self.do_table_ranges_overlap(table1, table2) {
                        // This could be a potential conflict - same lookup value could return different results
                        // depending on which table is used. For now, we treat this as a subset/superset conflict
                        // where the smaller table is a subset of the larger table
                        let table1_size = (table1.end_row - table1.start_row + 1)
                            * (table1.end_column - table1.start_column + 1);
                        let table2_size = (table2.end_row - table2.start_row + 1)
                            * (table2.end_column - table2.start_column + 1);

                        if table1_size < table2_size {
                            return Some(PatternConflict::SubsetSuperset {
                                subset_pattern: pattern1.clone(),
                                subset_value: value1,
                                superset_pattern: pattern2.clone(),
                                superset_value: value2,
                            });
                        } else if table2_size < table1_size {
                            return Some(PatternConflict::SubsetSuperset {
                                subset_pattern: pattern2.clone(),
                                subset_value: value2,
                                superset_pattern: pattern1.clone(),
                                superset_value: value1,
                            });
                        }
                    }
                }

                None
            }
            // Different function types don't conflict
            _ => None,
        }
    }

    /// Check if criteria_a is a subset of criteria_b
    fn is_criteria_subset(&self, criteria_a: &[CriteriaPair], criteria_b: &[CriteriaPair]) -> bool {
        if criteria_a.len() >= criteria_b.len() {
            return false; // A subset must have fewer elements
        }

        // Check if all criteria in A exist in B
        for pair_a in criteria_a {
            if !criteria_b.iter().any(|pair_b| pair_a == pair_b) {
                return false;
            }
        }

        true
    }

    /// Check if two table ranges overlap
    fn do_table_ranges_overlap(&self, table1: &TableRange, table2: &TableRange) -> bool {
        // Tables must be on the same sheet to overlap
        if table1.sheet != table2.sheet {
            return false;
        }

        // Check for rectangle overlap using standard algorithm
        // Two rectangles overlap if they overlap in both X and Y dimensions
        let x_overlap =
            table1.start_column <= table2.end_column && table2.start_column <= table1.end_column;
        let y_overlap = table1.start_row <= table2.end_row && table2.start_row <= table1.end_row;

        x_overlap && y_overlap
    }

    /// Add a substitution with conflict checking
    ///
    /// # Arguments
    /// * `pattern` - The pattern to add
    /// * `value` - The substitution value
    /// * `resolution` - How to handle conflicts
    ///
    /// # Returns
    /// * `Ok(Vec<PatternConflict>)` - Conflicts that were detected (empty if none)
    /// * `Err(PatternError)` - If the pattern is invalid or operation fails
    pub fn add_substitution_with_conflict_check(
        &mut self,
        pattern: PatternKey,
        value: f64,
        resolution: ConflictResolution,
    ) -> Result<Vec<PatternConflict>, PatternError> {
        // Validate the pattern first
        pattern.validate()?;

        // Check for conflicts
        let conflicts = self.check_conflicts(&pattern, value);

        if !conflicts.is_empty() {
            match resolution {
                ConflictResolution::Error => {
                    return Err(PatternError::InvalidSheetName {
                        name: format!("Conflicts detected: {:?}", conflicts),
                    });
                }
                ConflictResolution::OverwriteExisting => {
                    // Remove conflicting patterns and proceed
                    for conflict in &conflicts {
                        match conflict {
                            PatternConflict::ExactDuplicate { pattern, .. }
                            | PatternConflict::SamePatternDifferentValues { pattern, .. } => {
                                self.substitutions.remove(pattern);
                            }
                            PatternConflict::SubsetSuperset {
                                subset_pattern,
                                superset_pattern,
                                ..
                            } => {
                                // Remove the superset (more general) pattern to avoid ambiguity
                                self.substitutions.remove(superset_pattern);
                                self.substitutions.remove(subset_pattern);
                            }
                        }
                    }
                }
                ConflictResolution::KeepExisting => {
                    // Don't add the new pattern if there are conflicts
                    return Ok(conflicts);
                }
                ConflictResolution::WarnOnly => {
                    // Add anyway but return the conflicts as warnings
                }
            }
        }

        // Add the substitution
        self.substitutions.insert(pattern, value);
        self.stats.additions += 1;

        Ok(conflicts)
    }
}

/// Types of conflicts that can occur between patterns
#[derive(Debug, Clone, PartialEq)]
pub enum PatternConflict {
    /// Exact same pattern with same value (redundant)
    ExactDuplicate { pattern: PatternKey, value: f64 },
    /// Same pattern with different values (ambiguous)
    SamePatternDifferentValues {
        pattern: PatternKey,
        value1: f64,
        value2: f64,
    },
    /// One pattern is a subset of another (ambiguous matching)
    SubsetSuperset {
        subset_pattern: PatternKey,
        subset_value: f64,
        superset_pattern: PatternKey,
        superset_value: f64,
    },
}

/// How to resolve conflicts when adding substitutions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Fail with an error if conflicts are detected
    Error,
    /// Overwrite existing conflicting patterns
    OverwriteExisting,
    /// Keep existing patterns, don't add the new one
    KeepExisting,
    /// Add anyway but return conflicts as warnings
    WarnOnly,
}

impl Default for SubstitutionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryStats {
    /// Get the total number of lookup attempts
    pub fn total_lookups(&self) -> u64 {
        self.successful_lookups + self.failed_lookups + self.inactive_lookups
    }

    /// Get the success rate as a percentage (0.0 to 100.0)
    ///
    /// Only considers active lookups (excludes inactive_lookups)
    pub fn success_rate(&self) -> f64 {
        let active_lookups = self.successful_lookups + self.failed_lookups;
        if active_lookups == 0 {
            0.0
        } else {
            (self.successful_lookups as f64 / active_lookups as f64) * 100.0
        }
    }

    /// Get the net change in registry size (additions - removals)
    pub fn net_changes(&self) -> i64 {
        self.additions as i64 - self.removals as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_key_hash_equality() {
        let pattern1 = PatternKey::Sumifs {
            sheet: Some("Sheet1".to_string()),
            sum_column: 4,
            criteria_pairs: vec![CriteriaPair {
                column: 6,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        let pattern2 = PatternKey::Sumifs {
            sheet: Some("Sheet1".to_string()),
            sum_column: 4,
            criteria_pairs: vec![CriteriaPair {
                column: 6,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        let pattern3 = PatternKey::Sumifs {
            sheet: Some("Sheet1".to_string()),
            sum_column: 5, // Different column
            criteria_pairs: vec![CriteriaPair {
                column: 6,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        // Test equality
        assert_eq!(pattern1, pattern2);
        assert_ne!(pattern1, pattern3);

        // Test hash equality for HashMap usage
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(pattern1.clone(), 42.0);

        assert_eq!(map.get(&pattern2), Some(&42.0));
        assert_eq!(map.get(&pattern3), None);
    }

    #[test]
    fn test_criteria_value_types() {
        let string_criteria = CriteriaValue::String("Test".to_string());
        let number_criteria = CriteriaValue::Number(OrderedFloat(123.45));
        let cell_criteria = CriteriaValue::CellReference(CellReferenceIndex {
            sheet: 0,
            row: 1,
            column: 1,
        });

        // Test they can be hashed and compared
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(string_criteria.clone());
        set.insert(number_criteria.clone());
        set.insert(cell_criteria.clone());

        assert!(set.contains(&string_criteria));
        assert!(set.contains(&number_criteria));
        assert!(set.contains(&cell_criteria));
    }

    #[test]
    fn test_sumifs_builder() {
        let pattern = PatternKey::sumifs()
            .sheet("TestSheet")
            .sum_column(4)
            .criteria_string(6, "Collections Clinic")
            .criteria_number(8, 1811349806.0)
            .build()
            .unwrap();

        match pattern {
            PatternKey::Sumifs {
                sheet,
                sum_column,
                criteria_pairs,
            } => {
                assert_eq!(sheet, Some("TestSheet".to_string()));
                assert_eq!(sum_column, 4);
                assert_eq!(criteria_pairs.len(), 2);

                assert_eq!(criteria_pairs[0].column, 6);
                assert_eq!(
                    criteria_pairs[0].criteria,
                    CriteriaValue::String("Collections Clinic".to_string())
                );

                assert_eq!(criteria_pairs[1].column, 8);
                assert_eq!(
                    criteria_pairs[1].criteria,
                    CriteriaValue::Number(OrderedFloat(1811349806.0))
                );
            }
            _ => panic!("Expected Sumifs pattern"),
        }
    }

    #[test]
    fn test_builder_missing_required_field() {
        let pattern = PatternKey::sumifs()
            .sheet("TestSheet")
            .criteria_string(6, "Test")
            .build(); // Missing sum_column

        assert!(pattern.is_none());
    }

    #[test]
    fn test_criteria_value_convenience_constructors() {
        let string_val = CriteriaValue::string("Test");
        let number_val = CriteriaValue::number(42.0);
        let cell_val = CriteriaValue::cell_ref(0, 1, 1);

        assert_eq!(string_val, CriteriaValue::String("Test".to_string()));
        assert_eq!(number_val, CriteriaValue::Number(OrderedFloat(42.0)));
        assert_eq!(
            cell_val,
            CriteriaValue::CellReference(CellReferenceIndex {
                sheet: 0,
                row: 1,
                column: 1
            })
        );
    }

    #[test]
    fn test_complex_multi_criteria_pattern() {
        let pattern = PatternKey::sumifs()
            .sheet("INJECT.MONTHLY")
            .sum_column(4) // D column
            .criteria_string(6, "Collections Clinic") // F column
            .criteria_string(7, "Active") // G column
            .criteria_number(8, 100.0) // H column
            .build()
            .unwrap();

        // Test that it can be used in HashMap
        use std::collections::HashMap;
        let mut substitutions = HashMap::new();
        substitutions.insert(pattern.clone(), 42000.0);

        // Create identical pattern and verify lookup works
        let lookup_pattern = PatternKey::sumifs()
            .sheet("INJECT.MONTHLY")
            .sum_column(4)
            .criteria_string(6, "Collections Clinic")
            .criteria_string(7, "Active")
            .criteria_number(8, 100.0)
            .build()
            .unwrap();

        assert_eq!(substitutions.get(&lookup_pattern), Some(&42000.0));
    }

    #[test]
    fn test_from_node_basic_sumifs() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test basic SUMIFS with string criteria using absolute references
        let formula = "SUMIFS($D:$D,$F:$F,\"Collections Clinic\")";
        let ast = parser.parse(formula, &cell_reference);
        let pattern = PatternKey::from_node(&ast).unwrap();

        let expected = PatternKey::Sumifs {
            sheet: None,
            sum_column: 4, // D column (1-based: A=1, B=2, C=3, D=4)
            criteria_pairs: vec![CriteriaPair {
                column: 6, // F column (1-based: A=1, B=2, C=3, D=4, E=5, F=6)
                criteria: CriteriaValue::String("Collections Clinic".to_string()),
            }],
        };

        assert_eq!(pattern, expected);
    }

    #[test]
    fn test_from_node_multi_criteria_sumifs() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test SUMIFS with multiple criteria types using absolute references
        let formula = "SUMIFS($D:$D,$F:$F,\"Collections Clinic\",$H:$H,1811349806)";
        let ast = parser.parse(formula, &cell_reference);
        let pattern = PatternKey::from_node(&ast).unwrap();

        let expected = PatternKey::Sumifs {
            sheet: None,
            sum_column: 4, // D column (1-based)
            criteria_pairs: vec![
                CriteriaPair {
                    column: 6, // F column (1-based)
                    criteria: CriteriaValue::String("Collections Clinic".to_string()),
                },
                CriteriaPair {
                    column: 8, // H column (1-based)
                    criteria: CriteriaValue::Number(OrderedFloat(1811349806.0)),
                },
            ],
        };

        assert_eq!(pattern, expected);
    }

    #[test]
    fn test_from_node_cell_reference_criteria() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test SUMIFS with cell reference criteria using absolute references
        let formula = "SUMIFS($D:$D,$F:$F,$C$2)";
        let ast = parser.parse(formula, &cell_reference);
        let pattern = PatternKey::from_node(&ast).unwrap();

        let expected = PatternKey::Sumifs {
            sheet: None,
            sum_column: 4, // D column (1-based)
            criteria_pairs: vec![CriteriaPair {
                column: 6, // F column (1-based)
                criteria: CriteriaValue::CellReference(CellReferenceIndex {
                    sheet: 0,
                    row: 2,    // C2 (row 2 = 2 in 1-based)
                    column: 3, // C column (1-based in CellReferenceIndex)
                }),
            }],
        };

        assert_eq!(pattern, expected);
    }

    #[test]
    fn test_from_node_with_sheet_reference() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string(), "DataSheet".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test SUMIFS with explicit sheet reference
        let formula = "SUMIFS(DataSheet!D:D,DataSheet!F:F,\"Active\")";
        let ast = parser.parse(formula, &cell_reference);
        let pattern = PatternKey::from_node(&ast).unwrap();

        let expected = PatternKey::Sumifs {
            sheet: Some("DataSheet".to_string()),
            sum_column: 3, // D column (0-based)
            criteria_pairs: vec![CriteriaPair {
                column: 5, // F column (0-based)
                criteria: CriteriaValue::String("Active".to_string()),
            }],
        };

        assert_eq!(pattern, expected);
    }

    #[test]
    fn test_from_node_non_sumifs_function() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test non-SUMIFS function returns None
        let formula = "SUM(D:D)";
        let ast = parser.parse(formula, &cell_reference);
        let pattern = PatternKey::from_node(&ast);

        assert!(pattern.is_none());
    }

    #[test]
    fn test_from_node_invalid_sumifs_arguments() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test SUMIFS with insufficient arguments
        let formula = "SUMIFS(D:D,F:F)"; // Missing criteria
        let ast = parser.parse(formula, &cell_reference);
        let pattern = PatternKey::from_node(&ast);

        assert!(pattern.is_none());
    }

    #[test]
    fn test_from_node_multi_column_range_rejected() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test SUMIFS with multi-column range (should be rejected)
        let formula = "SUMIFS(D:E,F:F,\"Active\")"; // D:E spans multiple columns
        let ast = parser.parse(formula, &cell_reference);
        let pattern = PatternKey::from_node(&ast);

        assert!(pattern.is_none());
    }

    // SubstitutionRegistry tests

    #[test]
    fn test_substitution_registry_basic_operations() {
        let mut registry = SubstitutionRegistry::new();

        // Test initial state
        assert!(registry.is_active());
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        // Create a test pattern
        let pattern = PatternKey::sumifs()
            .sheet("TestSheet")
            .sum_column(4)
            .criteria_string(6, "Active")
            .build()
            .unwrap();

        // Test adding substitution
        let previous = registry.add_substitution(pattern.clone(), 42000.0);
        assert!(previous.is_none());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.contains_pattern(&pattern));

        // Test getting substitution
        let value = registry.get_substitution(&pattern);
        assert_eq!(value, Some(&42000.0));

        // Test updating substitution
        let previous = registry.add_substitution(pattern.clone(), 50000.0);
        assert_eq!(previous, Some(42000.0));
        assert_eq!(registry.len(), 1); // Should still be 1

        // Test removal
        let removed = registry.remove_substitution(&pattern);
        assert_eq!(removed, Some(50000.0));
        assert!(registry.is_empty());
        assert!(!registry.contains_pattern(&pattern));
    }

    #[test]
    fn test_substitution_registry_activation() {
        let mut registry = SubstitutionRegistry::new();

        let pattern = PatternKey::sumifs()
            .sheet("TestSheet")
            .sum_column(4)
            .criteria_string(6, "Active")
            .build()
            .unwrap();

        registry.add_substitution(pattern.clone(), 42000.0);

        // Test active lookup
        assert!(registry.is_active());
        assert_eq!(registry.get_substitution(&pattern), Some(&42000.0));

        // Test deactivation
        registry.set_active(false);
        assert!(!registry.is_active());
        assert_eq!(registry.get_substitution(&pattern), None);

        // Test data still exists when inactive
        assert!(registry.contains_pattern(&pattern));
        assert_eq!(registry.len(), 1);

        // Test reactivation
        registry.set_active(true);
        assert_eq!(registry.get_substitution(&pattern), Some(&42000.0));
    }

    #[test]
    fn test_substitution_registry_bulk_operations() {
        let mut registry = SubstitutionRegistry::new();

        // Create multiple patterns
        let pattern1 = PatternKey::sumifs()
            .sheet("Sheet1")
            .sum_column(4)
            .criteria_string(6, "Active")
            .build()
            .unwrap();

        let pattern2 = PatternKey::sumifs()
            .sheet("Sheet2")
            .sum_column(5)
            .criteria_string(7, "Inactive")
            .build()
            .unwrap();

        let pattern3 = PatternKey::sumifs()
            .sum_column(6)
            .criteria_number(8, 123.45)
            .build()
            .unwrap();

        // Test bulk addition
        let substitutions = vec![
            (pattern1.clone(), 1000.0),
            (pattern2.clone(), 2000.0),
            (pattern3.clone(), 3000.0),
        ];

        let count = registry.add_bulk(substitutions);
        assert_eq!(count, 3);
        assert_eq!(registry.len(), 3);

        // Test iteration
        let patterns: Vec<_> = registry.patterns().collect();
        assert_eq!(patterns.len(), 3);

        let values: Vec<_> = registry.values().collect();
        assert_eq!(values.len(), 3);

        let entries: Vec<_> = registry.entries().collect();
        assert_eq!(entries.len(), 3);

        // Test clear
        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_substitution_registry_statistics() {
        let mut registry = SubstitutionRegistry::new();

        let pattern1 = PatternKey::sumifs()
            .sum_column(4)
            .criteria_string(6, "Active")
            .build()
            .unwrap();

        let pattern2 = PatternKey::sumifs()
            .sum_column(5)
            .criteria_string(7, "Inactive")
            .build()
            .unwrap();

        // Test initial stats
        let stats = registry.stats();
        assert_eq!(stats.total_lookups(), 0);
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.net_changes(), 0);

        // Add patterns and test addition stats
        registry.add_substitution(pattern1.clone(), 1000.0);
        registry.add_substitution(pattern2.clone(), 2000.0);

        let stats = registry.stats();
        assert_eq!(stats.additions, 2);
        assert_eq!(stats.removals, 0);
        assert_eq!(stats.net_changes(), 2);

        // Test successful lookups
        registry.get_substitution(&pattern1);
        registry.get_substitution(&pattern2);

        let stats = registry.stats();
        assert_eq!(stats.successful_lookups, 2);
        assert_eq!(stats.failed_lookups, 0);
        assert_eq!(stats.success_rate(), 100.0);

        // Test failed lookups
        let nonexistent_pattern = PatternKey::sumifs()
            .sum_column(10)
            .criteria_string(11, "Nonexistent")
            .build()
            .unwrap();

        registry.get_substitution(&nonexistent_pattern);

        let stats = registry.stats();
        assert_eq!(stats.successful_lookups, 2);
        assert_eq!(stats.failed_lookups, 1);
        assert!((stats.success_rate() - 66.66666666666667).abs() < 0.001); // Approximately 66.67%

        // Test inactive lookups
        registry.set_active(false);
        registry.get_substitution(&pattern1);

        let stats = registry.stats();
        assert_eq!(stats.inactive_lookups, 1);
        assert_eq!(stats.total_lookups(), 4);

        // Test removal stats
        registry.set_active(true);
        registry.remove_substitution(&pattern1);

        let stats = registry.stats();
        assert_eq!(stats.removals, 1);
        assert_eq!(stats.net_changes(), 1);

        // Test stats reset
        registry.reset_stats();
        let stats = registry.stats();
        assert_eq!(stats.total_lookups(), 0);
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.removals, 0);
    }

    #[test]
    fn test_substitution_registry_with_capacity() {
        let registry = SubstitutionRegistry::with_capacity(100);
        assert!(registry.is_active());
        assert!(registry.is_empty());

        // Memory usage should account for capacity
        let memory_usage = registry.memory_usage_estimate();
        assert!(memory_usage > 0);
    }

    #[test]
    fn test_substitution_registry_memory_usage() {
        let mut registry = SubstitutionRegistry::new();

        let initial_usage = registry.memory_usage_estimate();

        // Add some patterns
        for i in 0..10 {
            let pattern = PatternKey::sumifs()
                .sum_column(i + 1)
                .criteria_string(6, &format!("Value{}", i))
                .build()
                .unwrap();
            registry.add_substitution(pattern, i as f64 * 1000.0);
        }

        let usage_with_data = registry.memory_usage_estimate();
        assert!(usage_with_data > initial_usage);
    }

    #[test]
    fn test_substitution_registry_default() {
        let registry = SubstitutionRegistry::default();
        assert!(registry.is_active());
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_stats_edge_cases() {
        let stats = RegistryStats::default();

        // Test success rate with no lookups
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.total_lookups(), 0);
        assert_eq!(stats.net_changes(), 0);

        // Test with only inactive lookups
        let stats = RegistryStats {
            successful_lookups: 0,
            failed_lookups: 0,
            inactive_lookups: 5,
            additions: 0,
            removals: 0,
        };

        assert_eq!(stats.success_rate(), 0.0); // No active lookups
        assert_eq!(stats.total_lookups(), 5);
    }

    // ==================== Milestone 5 Tests ====================

    #[test]
    fn test_pattern_validation_valid_patterns() {
        // Test valid single criteria pattern
        let pattern = PatternKey::Sumifs {
            sheet: Some("Sheet1".to_string()),
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };
        assert!(pattern.validate().is_ok());

        // Test valid multi-criteria pattern
        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 5,
            criteria_pairs: vec![
                CriteriaPair {
                    column: 1,
                    criteria: CriteriaValue::String("Active".to_string()),
                },
                CriteriaPair {
                    column: 3,
                    criteria: CriteriaValue::Number(OrderedFloat(100.0)),
                },
                CriteriaPair {
                    column: 7,
                    criteria: CriteriaValue::CellReference(CellReferenceIndex {
                        sheet: 0,
                        row: 5,
                        column: 2,
                    }),
                },
            ],
        };
        assert!(pattern.validate().is_ok());
    }

    #[test]
    fn test_pattern_validation_errors() {
        // Test empty criteria
        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![],
        };
        match pattern.validate() {
            Err(PatternError::EmptyCriteria) => {}
            _ => panic!("Expected EmptyCriteria error"),
        }

        // Test invalid sum column
        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: -1,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };
        match pattern.validate() {
            Err(PatternError::InvalidColumn { column: -1 }) => {}
            _ => panic!("Expected InvalidColumn error"),
        }

        // Test invalid criteria column
        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: -5,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };
        match pattern.validate() {
            Err(PatternError::InvalidColumn { column: -5 }) => {}
            _ => panic!("Expected InvalidColumn error"),
        }

        // Test duplicate columns
        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![
                CriteriaPair {
                    column: 3,
                    criteria: CriteriaValue::String("Test1".to_string()),
                },
                CriteriaPair {
                    column: 3, // Duplicate!
                    criteria: CriteriaValue::String("Test2".to_string()),
                },
            ],
        };
        match pattern.validate() {
            Err(PatternError::DuplicateColumn { column: 3 }) => {}
            _ => panic!("Expected DuplicateColumn error"),
        }

        // Test invalid sheet name
        let pattern = PatternKey::Sumifs {
            sheet: Some("".to_string()), // Empty sheet name
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };
        match pattern.validate() {
            Err(PatternError::InvalidSheetName { .. }) => {}
            _ => panic!("Expected InvalidSheetName error"),
        }

        // Test cell reference out of bounds
        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::CellReference(CellReferenceIndex {
                    sheet: 0,
                    row: -1, // Invalid row
                    column: 2,
                }),
            }],
        };
        match pattern.validate() {
            Err(PatternError::CellOutOfBounds { .. }) => {}
            _ => panic!("Expected CellOutOfBounds error"),
        }
    }

    #[test]
    fn test_too_many_criteria() {
        // Create a pattern with too many criteria pairs
        let mut criteria_pairs = Vec::new();
        for i in 0..101 {
            // Exceeds MAX_CRITERIA (100)
            criteria_pairs.push(CriteriaPair {
                column: i,
                criteria: CriteriaValue::String(format!("Test{}", i)),
            });
        }

        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 0,
            criteria_pairs,
        };

        match pattern.validate() {
            Err(PatternError::TooManyCriteria {
                max: 100,
                found: 101,
            }) => {}
            _ => panic!("Expected TooManyCriteria error"),
        }
    }

    #[test]
    fn test_conflict_detection_exact_duplicate() {
        let mut registry = SubstitutionRegistry::new();

        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        // Add first substitution
        registry.add_substitution(pattern.clone(), 100.0);

        // Check for conflicts with same pattern and same value
        let conflicts = registry.check_conflicts(&pattern, 100.0);
        assert_eq!(conflicts.len(), 1);
        match &conflicts[0] {
            PatternConflict::ExactDuplicate { value, .. } => {
                assert_eq!(*value, 100.0);
            }
            _ => panic!("Expected ExactDuplicate conflict"),
        }
    }

    #[test]
    fn test_conflict_detection_same_pattern_different_values() {
        let mut registry = SubstitutionRegistry::new();

        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        // Add first substitution
        registry.add_substitution(pattern.clone(), 100.0);

        // Check for conflicts with same pattern but different value
        let conflicts = registry.check_conflicts(&pattern, 200.0);
        assert_eq!(conflicts.len(), 1);
        match &conflicts[0] {
            PatternConflict::SamePatternDifferentValues { value1, value2, .. } => {
                assert_eq!(*value1, 100.0);
                assert_eq!(*value2, 200.0);
            }
            _ => panic!("Expected SamePatternDifferentValues conflict"),
        }
    }

    #[test]
    fn test_conflict_detection_subset_superset() {
        let mut registry = SubstitutionRegistry::new();

        // Create a superset pattern (more criteria)
        let superset_pattern = PatternKey::Sumifs {
            sheet: None,
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

        // Create a subset pattern (fewer criteria, but all exist in superset)
        let subset_pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Active".to_string()),
            }],
        };

        // Add superset first
        registry.add_substitution(superset_pattern.clone(), 200.0);

        // Check for conflicts when adding subset
        let conflicts = registry.check_conflicts(&subset_pattern, 100.0);
        assert_eq!(conflicts.len(), 1);
        match &conflicts[0] {
            PatternConflict::SubsetSuperset {
                subset_value,
                superset_value,
                ..
            } => {
                assert_eq!(*subset_value, 100.0);
                assert_eq!(*superset_value, 200.0);
            }
            _ => panic!("Expected SubsetSuperset conflict"),
        }
    }

    #[test]
    fn test_conflict_resolution_error() {
        let mut registry = SubstitutionRegistry::new();

        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        // Add first substitution
        registry.add_substitution(pattern.clone(), 100.0);

        // Try to add conflicting pattern with Error resolution
        let result = registry.add_substitution_with_conflict_check(
            pattern,
            200.0,
            ConflictResolution::Error,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_conflict_resolution_overwrite() {
        let mut registry = SubstitutionRegistry::new();

        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        // Add first substitution
        registry.add_substitution(pattern.clone(), 100.0);
        assert_eq!(registry.len(), 1);

        // Add conflicting pattern with OverwriteExisting resolution
        let result = registry.add_substitution_with_conflict_check(
            pattern.clone(),
            200.0,
            ConflictResolution::OverwriteExisting,
        );

        assert!(result.is_ok());
        assert_eq!(registry.len(), 1); // Should still be 1 (overwritten)
        assert_eq!(registry.get_substitution(&pattern), Some(&200.0));
    }

    #[test]
    fn test_conflict_resolution_keep_existing() {
        let mut registry = SubstitutionRegistry::new();

        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        // Add first substitution
        registry.add_substitution(pattern.clone(), 100.0);

        // Try to add conflicting pattern with KeepExisting resolution
        let result = registry.add_substitution_with_conflict_check(
            pattern.clone(),
            200.0,
            ConflictResolution::KeepExisting,
        );

        assert!(result.is_ok());
        let conflicts = result.unwrap();
        assert_eq!(conflicts.len(), 1); // Should report the conflict
        assert_eq!(registry.get_substitution(&pattern), Some(&100.0)); // Should keep original
    }

    #[test]
    fn test_conflict_resolution_warn_only() {
        let mut registry = SubstitutionRegistry::new();

        let pattern = PatternKey::Sumifs {
            sheet: None,
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        // Add first substitution
        registry.add_substitution(pattern.clone(), 100.0);

        // Add conflicting pattern with WarnOnly resolution
        let result = registry.add_substitution_with_conflict_check(
            pattern.clone(),
            200.0,
            ConflictResolution::WarnOnly,
        );

        assert!(result.is_ok());
        let conflicts = result.unwrap();
        assert_eq!(conflicts.len(), 1); // Should report the conflict
        assert_eq!(registry.get_substitution(&pattern), Some(&200.0)); // Should add anyway
    }

    #[test]
    fn test_enhanced_pattern_extraction_with_errors() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test with insufficient arguments
        let formula = "SUMIFS(D:D,F:F)"; // Missing criteria value
        let ast = parser.parse(formula, &cell_reference);
        let result = PatternKey::try_from_node(&ast);
        match result {
            Err(PatternError::InvalidArgumentCount { .. }) => {}
            _ => panic!("Expected InvalidArgumentCount error"),
        }

        // Test with even number of arguments (should be odd)
        let formula = "SUMIFS(D:D,F:F,\"Test\",G:G)"; // Missing criteria value for G:G
        let ast = parser.parse(formula, &cell_reference);
        let result = PatternKey::try_from_node(&ast);
        match result {
            Err(PatternError::InvalidArgumentCount { .. }) => {}
            _ => panic!("Expected InvalidArgumentCount error"),
        }
    }

    #[test]
    fn test_pattern_error_display() {
        let error = PatternError::EmptyCriteria;
        assert_eq!(
            error.to_string(),
            "Pattern must contain at least one criteria pair"
        );

        let error = PatternError::TooManyCriteria {
            max: 100,
            found: 150,
        };
        assert_eq!(
            error.to_string(),
            "Too many criteria pairs: found 150, maximum supported is 100"
        );

        let error = PatternError::InvalidColumn { column: -5 };
        assert_eq!(error.to_string(), "Invalid column index: -5 (must be >= 0)");

        let error = PatternError::DuplicateColumn { column: 3 };
        assert_eq!(error.to_string(), "Duplicate criteria column: 3");

        let error = PatternError::MultiColumnRange { start: 1, end: 5 };
        assert_eq!(
            error.to_string(),
            "Multi-column ranges not supported: columns 1 to 5"
        );
    }

    #[test]
    fn test_multi_criteria_patterns() {
        // Test creating and validating patterns with many criteria
        let mut criteria_pairs = Vec::new();
        for i in 0..50 {
            // Well under the limit of 100
            criteria_pairs.push(CriteriaPair {
                column: i,
                criteria: CriteriaValue::String(format!("Criteria{}", i)),
            });
        }

        let pattern = PatternKey::Sumifs {
            sheet: Some("MultiSheet".to_string()),
            sum_column: 1000,
            criteria_pairs,
        };

        // Should validate successfully
        assert!(pattern.validate().is_ok());

        // Test building with the builder
        let mut builder = PatternKey::sumifs().sheet("TestSheet").sum_column(5);

        for i in 0..10 {
            builder = builder.criteria_string(i * 2, &format!("Value{}", i));
        }

        let built_pattern = builder.build().unwrap();
        assert!(built_pattern.validate().is_ok());

        match built_pattern {
            PatternKey::Sumifs { criteria_pairs, .. } => {
                assert_eq!(criteria_pairs.len(), 10);
            }
            _ => panic!("Expected Sumifs pattern"),
        }
    }

    #[test]
    fn test_countifs_builder() {
        let pattern = PatternKey::countifs()
            .sheet("TestSheet")
            .criteria_string(6, "Active")
            .criteria_number(8, 100.0)
            .build()
            .unwrap();

        match pattern {
            PatternKey::Countifs {
                sheet,
                criteria_pairs,
            } => {
                assert_eq!(sheet, Some("TestSheet".to_string()));
                assert_eq!(criteria_pairs.len(), 2);

                assert_eq!(criteria_pairs[0].column, 6);
                assert_eq!(
                    criteria_pairs[0].criteria,
                    CriteriaValue::String("Active".to_string())
                );

                assert_eq!(criteria_pairs[1].column, 8);
                assert_eq!(
                    criteria_pairs[1].criteria,
                    CriteriaValue::Number(OrderedFloat(100.0))
                );
            }
            _ => panic!("Expected Countifs pattern"),
        }
    }

    #[test]
    fn test_countifs_builder_missing_criteria() {
        let pattern = PatternKey::countifs().sheet("TestSheet").build(); // No criteria pairs

        assert!(pattern.is_none());
    }

    #[test]
    fn test_averageifs_builder() {
        let pattern = PatternKey::averageifs()
            .sheet("DataSheet")
            .average_column(4)
            .criteria_string(6, "Electronics")
            .criteria_number(8, 2023.0)
            .build()
            .unwrap();

        match pattern {
            PatternKey::Averageifs {
                sheet,
                average_column,
                criteria_pairs,
            } => {
                assert_eq!(sheet, Some("DataSheet".to_string()));
                assert_eq!(average_column, 4);
                assert_eq!(criteria_pairs.len(), 2);

                assert_eq!(criteria_pairs[0].column, 6);
                assert_eq!(
                    criteria_pairs[0].criteria,
                    CriteriaValue::String("Electronics".to_string())
                );

                assert_eq!(criteria_pairs[1].column, 8);
                assert_eq!(
                    criteria_pairs[1].criteria,
                    CriteriaValue::Number(OrderedFloat(2023.0))
                );
            }
            _ => panic!("Expected Averageifs pattern"),
        }
    }

    #[test]
    fn test_averageifs_builder_missing_average_column() {
        let pattern = PatternKey::averageifs()
            .sheet("TestSheet")
            .criteria_string(6, "Test")
            .build(); // Missing average_column

        assert!(pattern.is_none());
    }

    #[test]
    fn test_from_node_countifs() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test basic COUNTIFS with string criteria
        let formula = "COUNTIFS($A:$A,\"Active\",$B:$B,\">100\")";
        let ast = parser.parse(formula, &cell_reference);

        if let Some(pattern) = PatternKey::from_node(&ast) {
            match pattern {
                PatternKey::Countifs {
                    sheet,
                    criteria_pairs,
                } => {
                    assert_eq!(sheet, None);
                    assert_eq!(criteria_pairs.len(), 2);
                    assert_eq!(criteria_pairs[0].column, 1); // A column
                    assert_eq!(criteria_pairs[1].column, 2); // B column
                }
                _ => panic!("Expected Countifs pattern"),
            }
        } else {
            panic!("Failed to extract pattern");
        }
    }

    #[test]
    fn test_from_node_averageifs() {
        use crate::expressions::parser::Parser;
        use crate::expressions::types::CellReferenceRC;
        use std::collections::HashMap;

        let worksheets = vec!["Sheet1".to_string()];
        let mut parser = Parser::new(worksheets, vec![], HashMap::new());

        let cell_reference = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };

        // Test basic AVERAGEIFS with string criteria
        let formula = "AVERAGEIFS($C:$C,$A:$A,\"Active\",$B:$B,\">100\")";
        let ast = parser.parse(formula, &cell_reference);

        if let Some(pattern) = PatternKey::from_node(&ast) {
            match pattern {
                PatternKey::Averageifs {
                    sheet,
                    average_column,
                    criteria_pairs,
                } => {
                    assert_eq!(sheet, None);
                    assert_eq!(average_column, 3); // C column
                    assert_eq!(criteria_pairs.len(), 2);
                    assert_eq!(criteria_pairs[0].column, 1); // A column
                    assert_eq!(criteria_pairs[1].column, 2); // B column
                }
                _ => panic!("Expected Averageifs pattern"),
            }
        } else {
            panic!("Failed to extract pattern");
        }
    }

    #[test]
    fn test_pattern_validation_countifs() {
        // Valid pattern
        let valid_pattern = PatternKey::Countifs {
            sheet: Some("TestSheet".to_string()),
            criteria_pairs: vec![
                CriteriaPair {
                    column: 1,
                    criteria: CriteriaValue::String("Active".to_string()),
                },
                CriteriaPair {
                    column: 2,
                    criteria: CriteriaValue::Number(OrderedFloat(100.0)),
                },
            ],
        };
        assert!(valid_pattern.validate().is_ok());

        // Invalid pattern - empty criteria
        let empty_pattern = PatternKey::Countifs {
            sheet: None,
            criteria_pairs: vec![],
        };
        assert!(matches!(
            empty_pattern.validate(),
            Err(PatternError::EmptyCriteria)
        ));

        // Invalid pattern - duplicate columns
        let duplicate_pattern = PatternKey::Countifs {
            sheet: None,
            criteria_pairs: vec![
                CriteriaPair {
                    column: 1,
                    criteria: CriteriaValue::String("A".to_string()),
                },
                CriteriaPair {
                    column: 1, // Duplicate
                    criteria: CriteriaValue::String("B".to_string()),
                },
            ],
        };
        assert!(matches!(
            duplicate_pattern.validate(),
            Err(PatternError::DuplicateColumn { column: 1 })
        ));
    }

    #[test]
    fn test_pattern_validation_averageifs() {
        // Valid pattern
        let valid_pattern = PatternKey::Averageifs {
            sheet: Some("TestSheet".to_string()),
            average_column: 3,
            criteria_pairs: vec![
                CriteriaPair {
                    column: 1,
                    criteria: CriteriaValue::String("Active".to_string()),
                },
                CriteriaPair {
                    column: 2,
                    criteria: CriteriaValue::Number(OrderedFloat(100.0)),
                },
            ],
        };
        assert!(valid_pattern.validate().is_ok());

        // Invalid pattern - negative average column
        let invalid_pattern = PatternKey::Averageifs {
            sheet: None,
            average_column: -1,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };
        assert!(matches!(
            invalid_pattern.validate(),
            Err(PatternError::InvalidColumn { column: -1 })
        ));
    }

    #[test]
    fn test_pattern_conflicts_different_function_types() {
        let mut registry = SubstitutionRegistry::new();

        // Create patterns of different function types but similar structure
        let sumifs_pattern = PatternKey::Sumifs {
            sheet: Some("Sheet1".to_string()),
            sum_column: 3,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Active".to_string()),
            }],
        };

        let countifs_pattern = PatternKey::Countifs {
            sheet: Some("Sheet1".to_string()),
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Active".to_string()),
            }],
        };

        let averageifs_pattern = PatternKey::Averageifs {
            sheet: Some("Sheet1".to_string()),
            average_column: 3,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Active".to_string()),
            }],
        };

        // Add all patterns - they shouldn't conflict despite structural similarity
        registry.add_substitution(sumifs_pattern.clone(), 1000.0);
        registry.add_substitution(countifs_pattern.clone(), 50.0);
        registry.add_substitution(averageifs_pattern.clone(), 20.0);

        // Verify all patterns are present
        assert_eq!(registry.len(), 3);
        assert_eq!(registry.get_substitution(&sumifs_pattern), Some(&1000.0));
        assert_eq!(registry.get_substitution(&countifs_pattern), Some(&50.0));
        assert_eq!(registry.get_substitution(&averageifs_pattern), Some(&20.0));
    }

    #[test]
    fn test_vlookup_builder_basic() {
        let pattern = PatternKey::vlookup()
            .sheet("ProductData")
            .lookup_string("Widget A")
            .table_range(1, 1, 100, 5)
            .column_index(3)
            .exact_match(true)
            .build()
            .unwrap();

        match pattern {
            PatternKey::Vlookup {
                sheet,
                lookup_value,
                table_range,
                column_index,
                exact_match,
            } => {
                assert_eq!(sheet, Some("ProductData".to_string()));
                assert_eq!(lookup_value, LookupValue::String("Widget A".to_string()));
                assert_eq!(table_range.start_row, 1);
                assert_eq!(table_range.start_column, 1);
                assert_eq!(table_range.end_row, 100);
                assert_eq!(table_range.end_column, 5);
                assert_eq!(column_index, 3);
                assert!(exact_match);
            }
            _ => panic!("Expected VLOOKUP pattern"),
        }
    }

    #[test]
    fn test_vlookup_builder_with_number() {
        let pattern = PatternKey::vlookup()
            .lookup_number(123.45)
            .table_range(1, 1, 50, 3)
            .column_index(2)
            .build()
            .unwrap();

        match pattern {
            PatternKey::Vlookup {
                lookup_value,
                exact_match,
                ..
            } => {
                assert_eq!(lookup_value, LookupValue::Number(OrderedFloat(123.45)));
                assert!(!exact_match); // Default should be approximate match
            }
            _ => panic!("Expected VLOOKUP pattern"),
        }
    }

    #[test]
    fn test_vlookup_builder_with_cell_reference() {
        let pattern = PatternKey::vlookup()
            .lookup_cell_ref(0, 5, 2) // Sheet 0, row 5, column 2 (B5)
            .table_range_with_sheet("LookupTable", 1, 1, 100, 4)
            .column_index(4)
            .exact_match(false)
            .build()
            .unwrap();

        match pattern {
            PatternKey::Vlookup {
                sheet,
                lookup_value,
                table_range,
                ..
            } => {
                assert_eq!(sheet, None); // No global sheet set, table has its own sheet
                assert_eq!(
                    lookup_value,
                    LookupValue::CellReference(CellReferenceIndex {
                        sheet: 0,
                        row: 5,
                        column: 2
                    })
                );
                assert_eq!(table_range.sheet, Some("LookupTable".to_string()));
            }
            _ => panic!("Expected VLOOKUP pattern"),
        }
    }

    #[test]
    fn test_vlookup_validation() {
        // Valid pattern
        let valid_pattern = PatternKey::Vlookup {
            sheet: Some("Data".to_string()),
            lookup_value: LookupValue::String("Test".to_string()),
            table_range: TableRange {
                sheet: Some("Data".to_string()),
                start_row: 1,
                start_column: 1,
                end_row: 10,
                end_column: 3,
            },
            column_index: 2,
            exact_match: true,
        };
        assert!(valid_pattern.validate().is_ok());

        // Invalid pattern - column index out of range
        let invalid_pattern = PatternKey::Vlookup {
            sheet: Some("Data".to_string()),
            lookup_value: LookupValue::String("Test".to_string()),
            table_range: TableRange {
                sheet: Some("Data".to_string()),
                start_row: 1,
                start_column: 1,
                end_row: 10,
                end_column: 3,
            },
            column_index: 5, // Table only has 3 columns
            exact_match: true,
        };
        assert!(invalid_pattern.validate().is_err());

        // Invalid pattern - zero column index
        let zero_column_pattern = PatternKey::Vlookup {
            sheet: Some("Data".to_string()),
            lookup_value: LookupValue::String("Test".to_string()),
            table_range: TableRange {
                sheet: Some("Data".to_string()),
                start_row: 1,
                start_column: 1,
                end_row: 10,
                end_column: 3,
            },
            column_index: 0,
            exact_match: true,
        };
        assert!(zero_column_pattern.validate().is_err());
    }

    #[test]
    fn test_vlookup_conflict_detection() {
        let mut registry = SubstitutionRegistry::new();

        let pattern1 = PatternKey::Vlookup {
            sheet: Some("Data".to_string()),
            lookup_value: LookupValue::String("Product A".to_string()),
            table_range: TableRange {
                sheet: Some("Data".to_string()),
                start_row: 1,
                start_column: 1,
                end_row: 100,
                end_column: 3,
            },
            column_index: 2,
            exact_match: true,
        };

        let pattern2 = PatternKey::Vlookup {
            sheet: Some("Data".to_string()),
            lookup_value: LookupValue::String("Product A".to_string()),
            table_range: TableRange {
                sheet: Some("Data".to_string()),
                start_row: 1,
                start_column: 1,
                end_row: 50,
                end_column: 3,
            },
            column_index: 2,
            exact_match: true,
        };

        // Add first pattern
        registry.add_substitution(pattern1.clone(), 100.0);

        // Check for conflicts with second pattern (smaller table range)
        let conflicts = registry.check_conflicts(&pattern2, 200.0);
        assert!(!conflicts.is_empty());

        // Should detect subset/superset conflict
        match &conflicts[0] {
            PatternConflict::SubsetSuperset {
                subset_pattern,
                superset_pattern,
                ..
            } => {
                assert_eq!(subset_pattern, &pattern2); // smaller table is subset
                assert_eq!(superset_pattern, &pattern1); // larger table is superset
            }
            _ => panic!("Expected subset/superset conflict"),
        }
    }

    #[test]
    fn test_vlookup_different_function_types_no_conflict() {
        let mut registry = SubstitutionRegistry::new();

        // VLOOKUP pattern
        let vlookup_pattern = PatternKey::Vlookup {
            sheet: Some("Data".to_string()),
            lookup_value: LookupValue::String("Test".to_string()),
            table_range: TableRange {
                sheet: Some("Data".to_string()),
                start_row: 1,
                start_column: 1,
                end_row: 10,
                end_column: 3,
            },
            column_index: 2,
            exact_match: true,
        };

        // SUMIFS pattern with similar structure
        let sumifs_pattern = PatternKey::Sumifs {
            sheet: Some("Data".to_string()),
            sum_column: 2,
            criteria_pairs: vec![CriteriaPair {
                column: 1,
                criteria: CriteriaValue::String("Test".to_string()),
            }],
        };

        // Add both patterns - they shouldn't conflict despite similar data
        registry.add_substitution(vlookup_pattern.clone(), 1000.0);
        registry.add_substitution(sumifs_pattern.clone(), 500.0);

        // Verify both patterns are present
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get_substitution(&vlookup_pattern), Some(&1000.0));
        assert_eq!(registry.get_substitution(&sumifs_pattern), Some(&500.0));
    }

    #[test]
    fn test_vlookup_lookup_value_convenience_constructors() {
        // Test LookupValue convenience constructors
        assert_eq!(
            LookupValue::string("test"),
            LookupValue::String("test".to_string())
        );
        assert_eq!(
            LookupValue::number(42.0),
            LookupValue::Number(OrderedFloat(42.0))
        );
        assert_eq!(
            LookupValue::cell_ref(0, 1, 2),
            LookupValue::CellReference(CellReferenceIndex {
                sheet: 0,
                row: 1,
                column: 2
            })
        );
    }

    #[test]
    fn test_table_range_overlap_detection() {
        let registry = SubstitutionRegistry::new();

        // Overlapping ranges
        let range1 = TableRange {
            sheet: Some("Sheet1".to_string()),
            start_row: 1,
            start_column: 1,
            end_row: 10,
            end_column: 5,
        };

        let range2 = TableRange {
            sheet: Some("Sheet1".to_string()),
            start_row: 5,
            start_column: 3,
            end_row: 15,
            end_column: 8,
        };

        assert!(registry.do_table_ranges_overlap(&range1, &range2));

        // Non-overlapping ranges
        let range3 = TableRange {
            sheet: Some("Sheet1".to_string()),
            start_row: 20,
            start_column: 1,
            end_row: 30,
            end_column: 5,
        };

        assert!(!registry.do_table_ranges_overlap(&range1, &range3));

        // Different sheets
        let range4 = TableRange {
            sheet: Some("Sheet2".to_string()),
            start_row: 1,
            start_column: 1,
            end_row: 10,
            end_column: 5,
        };

        assert!(!registry.do_table_ranges_overlap(&range1, &range4));
    }
}
