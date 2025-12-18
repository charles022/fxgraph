use arrow::array::{
    Array, ArrayRef, BooleanArray, Float32Array, Float32Builder, Int32Array, StringArray,
    StringBuilder, RecordBatch, AsArray,
};
use arrow::compute::{
    cast, filter_record_batch, lexsort_to_indices, max, sort_to_indices, take, SortColumn,
    SortOptions,
};
use arrow::datatypes::{DataType, Field, Schema};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

fn main() {
    println!("--- 1. Creating Table (RecordBatch) ---");
    let batch = create_dummy_data();
    print_batch(&batch);

    println!("\n--- 2. Filter (Region = 'US' AND Sales > 150.0) ---");
    filter_example(&batch);

    println!("\n--- 3. Multi-Column Sort (Region ASC, Sales DESC) ---");
    sort_example(&batch);

    println!("\n--- 4. Column Aggregation (Max Sales) ---");
    max_example(&batch);

    println!("\n--- 5. Manual Group By (Sum Sales by Region) ---");
    group_by_example(&batch);

    println!("\n--- 6. Manual Pivot Table (Row: Region, Col: Product, Val: Sum(Sales)) ---");
    pivot_example(&batch);

    println!("\n--- 7. Unique Capabilities (Direct Indexing & HashMaps) ---");
    unique_operations_example(&batch);
}

/// Helper to create a dummy dataset
fn create_dummy_data() -> RecordBatch {
    let regions = StringArray::from(vec!["US", "US", "UK", "US", "UK", "EU", "EU", "US"]);
    let products = StringArray::from(vec!["A", "B", "A", "A", "B", "A", "B", "B"]);
    let sales = Float32Array::from(vec![100.0, 200.0, 150.0, 300.0, 120.0, 80.0, 220.0, 110.0]);
    let ids = Int32Array::from(vec![1, 2, 3, 4, 5, 6, 7, 8]);

    let schema = Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("region", DataType::Utf8, false),
        Field::new("product", DataType::Utf8, false),
        Field::new("sales", DataType::Float32, false),
    ]);

    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(ids),
            Arc::new(regions),
            Arc::new(products),
            Arc::new(sales),
        ],
    )
    .unwrap()
}

/// Helper to print a RecordBatch to console
fn print_batch(batch: &RecordBatch) {
    let ids: &Int32Array = batch.column(0).as_primitive();
    let regions: &StringArray = batch.column(1).as_string();
    let products: &StringArray = batch.column(2).as_string();
    let sales: &Float32Array = batch.column(3).as_primitive();

    println!("{:<5} {:<10} {:<10} {:<10}", "ID", "Region", "Product", "Sales");
    for i in 0..batch.num_rows() {
        println!(
            "{:<5} {:<10} {:<10} {:<10.1}",
            ids.value(i),
            regions.value(i),
            products.value(i),
            sales.value(i)
        );
    }
}

// ==========================================
// 2. Filter Example
// ==========================================
fn filter_example(batch: &RecordBatch) {
    let regions: &StringArray = batch.column(1).as_string();
    let sales: &Float32Array = batch.column(3).as_primitive();

    // In Raw Arrow, we build a BooleanArray (mask) manually or via comparison kernels.
    // Logic: (region == "US") && (sales > 150.0)
    
    // We can use the 'arrow::compute::kernels::cmp' module, or standard iterators for complex logic.
    // Using iterators is often more readable for custom logic in "Manual" mode.
    let mask: BooleanArray = (0..batch.num_rows())
        .map(|i| {
            let region = regions.value(i);
            let sale = sales.value(i);
            Some(region == "US" && sale > 150.0)
        })
        .collect();

    // Apply the filter kernel to the whole batch
    let filtered_batch = filter_record_batch(batch, &mask).unwrap();
    print_batch(&filtered_batch);
}

// ==========================================
// 3. Multi-Column Sort Example
// ==========================================
fn sort_example(batch: &RecordBatch) {
    let region_col = batch.column(1);
    let sales_col = batch.column(3);

    // Define sort columns: Region (Ascending), Sales (Descending)
    let sort_columns = vec![
        SortColumn {
            values: region_col.clone(),
            options: Some(SortOptions {
                descending: false,
                nulls_first: false,
            }),
        },
        SortColumn {
            values: sales_col.clone(),
            options: Some(SortOptions {
                descending: true,
                nulls_first: true, // typical for desc
            }),
        },
    ];

    // 1. Calculate Indices (The "Permutation")
    // This returns an array like [3, 1, 7, 0, ...] representing the new order
    let indices = lexsort_to_indices(&sort_columns, None).unwrap();

    // 2. Reorder (Take) the data using the indices
    // We must manually apply 'take' to every column in the batch
    let sorted_columns: Vec<ArrayRef> = batch
        .columns()
        .iter()
        .map(|col| take(col, &indices, None).unwrap())
        .collect();

    let sorted_batch = RecordBatch::try_new(batch.schema(), sorted_columns).unwrap();
    print_batch(&sorted_batch);
}

// ==========================================
// 4. Max Example
// ==========================================
fn max_example(batch: &RecordBatch) {
    let sales: &Float32Array = batch.column(3).as_primitive();
    
    // arrow::compute::max returns Option<T>
    let max_val = max(sales).unwrap();
    println!("Max Sales: {}", max_val);
}

// ==========================================
// 5. Group By Example (Manual)
// ==========================================
fn group_by_example(batch: &RecordBatch) {
    // In Polars this is one line. In Arrow, we build it.
    // Goal: Map<Region, Sum(Sales)>
    
    let regions: &StringArray = batch.column(1).as_string();
    let sales: &Float32Array = batch.column(3).as_primitive();

    let mut sums = HashMap::new();

    // 1. Single pass aggregation
    for i in 0..batch.num_rows() {
        let r = regions.value(i);
        let s = sales.value(i);
        
        *sums.entry(r.to_string()).or_insert(0.0) += s;
    }

    // 2. Convert back to Arrow Arrays for display
    let mut region_builder = StringBuilder::new();
    let mut sales_builder = Float32Builder::new();

    for (region, total_sales) in sums {
        region_builder.append_value(region);
        sales_builder.append_value(total_sales);
    }

    let result_batch = RecordBatch::try_from_iter(vec![
        ("region", Arc::new(region_builder.finish()) as ArrayRef),
        ("sum_sales", Arc::new(sales_builder.finish()) as ArrayRef),
    ]).unwrap();

    print_batch(&result_batch);
}

// ==========================================
// 6. Pivot Table Example (Manual)
// ==========================================
fn pivot_example(batch: &RecordBatch) {
    // Pivot: Region (Rows) x Product (Cols) -> Sum(Sales)
    
    let regions: &StringArray = batch.column(1).as_string();
    let products: &StringArray = batch.column(2).as_string();
    let sales: &Float32Array = batch.column(3).as_primitive();

    // 1. Identify Unique Rows and Cols
    let mut unique_regions = HashSet::new();
    let mut unique_products = HashSet::new();
    // Map<(Region, Product), Sales>
    let mut value_map: HashMap<(String, String), f32> = HashMap::new();

    for i in 0..batch.num_rows() {
        let r = regions.value(i);
        let p = products.value(i);
        let s = sales.value(i);

        unique_regions.insert(r);
        unique_products.insert(p);
        *value_map.entry((r.to_string(), p.to_string())).or_insert(0.0) += s;
    }

    // Sort keys for consistent output
    let mut sorted_regions: Vec<_> = unique_regions.into_iter().collect();
    sorted_regions.sort();
    let mut sorted_products: Vec<_> = unique_products.into_iter().collect();
    sorted_products.sort();

    // 2. Build Columns
    let mut result_columns: Vec<ArrayRef> = Vec::new();
    
    // First column: Region Name
    let mut region_col_builder = StringBuilder::new();
    for r in &sorted_regions {
        region_col_builder.append_value(r);
    }
    result_columns.push(Arc::new(region_col_builder.finish()));

    // Subsequent columns: One per Product
    for p in &sorted_products {
        let mut product_col_builder = Float32Builder::new();
        for r in &sorted_regions {
            // Lookup value
            let val = value_map.get(&(r.clone().to_string(), p.clone().to_string())).unwrap_or(&0.0);
            product_col_builder.append_value(*val);
        }
        result_columns.push(Arc::new(product_col_builder.finish()));
    }

    // 3. Build Schema
    let mut fields = vec![Field::new("Region", DataType::Utf8, false)];
    for p in &sorted_products {
        fields.push(Field::new(p, DataType::Float32, false));
    }

    let pivot_batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), result_columns).unwrap();
    print_batch(&pivot_batch);
}

// ==========================================
// 7. Unique/Efficient Operations (Polars alternatives)
// ==========================================
fn unique_operations_example(batch: &RecordBatch) {
    let ids: &Int32Array = batch.column(0).as_primitive();
    let regions: &StringArray = batch.column(1).as_string();
    let sales: &Float32Array = batch.column(3).as_primitive();

    // --- A. Lookup via HashMap (O(1) random access) ---
    // Polars doesn't let you easily "own" a HashIndex to specific rows.
    // In Arrow, you can build a map of "Region Name -> List of Row Indices"
    // and keep it around for ultra-fast lookups later.
    println!("A. HashMap Index Lookup:");
    let mut region_index: HashMap<&str, Vec<usize>> = HashMap::new();
    for i in 0..batch.num_rows() {
        region_index.entry(regions.value(i)).or_default().push(i);
    }

    // Now we can instantly get all sales for "UK" without scanning the array again
    if let Some(indices) = region_index.get("UK") {
        for &idx in indices {
            println!("   Row {}: Sales = {}", idx, sales.value(idx));
        }
    }

    // --- B. Direct `usize` Indexing (Zero-Copy) ---
    // In Polars, `df[i]` often involves overhead or cloning scalar values.
    // In Arrow, `array.value(i)` is effectively a pointer offset.
    // This allows for extremely high-performance custom kernels.
    println!("\nB. Direct Indexing (Custom Logic):");
    let mut weighted_score = 0.0;
    for i in 0..batch.num_rows() {
        // Complex logic that might be hard to express in DataFrame Expressions
        // e.g., "If ID is even, multiply sales by 1.5, else by 1.0"
        let multiplier = if ids.value(i) % 2 == 0 { 1.5 } else { 1.0 };
        weighted_score += sales.value(i) * multiplier;
    }
    println!("   Calculated Weighted Score: {}", weighted_score);

    // --- C. Partial Slicing (References) ---
    // We can pass a "Slice" of an array to a function without copying data.
    // Polars usually slices the whole DataFrame. Arrow allows per-column slicing.
    println!("\nC. Zero-Copy Slicing:");
    // Slice rows 2 to 5 (length 3)
    let sales_slice = sales.slice(2, 3); 
    // This `sales_slice` points to the same memory block as `sales`. No allocation.
    let as_primitive: &Float32Array = sales_slice.as_primitive();
    println!("   Slice (2..5): {:?}", as_primitive.values());
}
