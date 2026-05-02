//! Project Fysarum: Dynamic Schema Registry
//!
//! This module handles byte-offset math. Because we are abandoning hardcoded 
//! C-structs, the engine needs a registry to understand how to read the raw 
//! bytes inside the zero-copy memory arena.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Int64,       // 8 bytes (e.g., Entity IDs, Timestamps)
    Float64,     // 8 bytes (e.g., Sensor metrics, Vector embeddings)
    FixedString(usize), // N bytes (e.g., FixedString(16) for a 16-char string)
}

impl DataType {
    /// Returns exactly how many bytes this data type consumes in memory.
    pub fn size_bytes(&self) -> usize {
        match self {
            DataType::Int64 => 8,
            DataType::Float64 => 8,
            DataType::FixedString(len) => *len,
        }
    }
}

/// Represents a single column in our database.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    /// The exact byte-offset where this field begins inside a row.
    pub offset: usize, 
}

/// The map that tells the CPU how to parse the raw NVMe bytes.
#[derive(Debug, Clone)]
pub struct Schema {
    fields: Vec<Field>,
    /// The total "stride" (length) of a single row in bytes.
    total_row_size: usize,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            total_row_size: 0,
        }
    }

    /// Appends a new column to the schema.
    /// It autonomously calculates the byte-offset based on previous fields.
    pub fn add_field(mut self, name: &str, data_type: DataType) -> Self {
        let offset = self.total_row_size;
        let size = data_type.size_bytes();
        
        self.fields.push(Field {
            name: name.to_string(),
            data_type,
            offset,
        });

        self.total_row_size += size;
        self
    }

    /// Returns the total size of a row (the "stride" the CPU must jump).
    pub fn row_size(&self) -> usize {
        self.total_row_size
    }

    /// Looks up a specific column by name to find its byte offset.
    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }
    
    /// Returns an iterator over all fields
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}