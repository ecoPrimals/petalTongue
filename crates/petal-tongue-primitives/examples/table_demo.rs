//! Table Primitive Demo
//!
//! Demonstrates the table primitive with sample data.
//!
//! Run with: cargo run --example table_demo

use petal_tongue_primitives::table::{Column, SortDirection, Table};

#[derive(Debug, Clone)]
struct Package {
    name: String,
    version: String,
    downloads: u64,
    license: String,
}

fn main() {
    println!("🌸 petalTongue Table Primitive Demo\n");

    // Sample data: Rust crates
    let packages = vec![
        Package {
            name: "tokio".to_string(),
            version: "1.35.0".to_string(),
            downloads: 150_000_000,
            license: "MIT".to_string(),
        },
        Package {
            name: "serde".to_string(),
            version: "1.0.195".to_string(),
            downloads: 200_000_000,
            license: "MIT/Apache-2.0".to_string(),
        },
        Package {
            name: "clap".to_string(),
            version: "4.4.18".to_string(),
            downloads: 80_000_000,
            license: "MIT/Apache-2.0".to_string(),
        },
        Package {
            name: "reqwest".to_string(),
            version: "0.11.23".to_string(),
            downloads: 95_000_000,
            license: "MIT/Apache-2.0".to_string(),
        },
        Package {
            name: "anyhow".to_string(),
            version: "1.0.79".to_string(),
            downloads: 120_000_000,
            license: "MIT/Apache-2.0".to_string(),
        },
    ];

    // Create table with columns
    let mut table = Table::new()
        .with_column(Column::new("Package", |p: &Package| p.name.clone()).width(20))
        .with_column(Column::new("Version", |p: &Package| p.version.clone()).width(12))
        .with_column(
            Column::new("Downloads", |p: &Package| format!("{:>12}", p.downloads)).width(15),
        )
        .with_column(Column::new("License", |p: &Package| p.license.clone()).width(20))
        .with_data(packages);

    // Display original table
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Original Table ({} rows)", table.row_count());
    println!("═══════════════════════════════════════════════════════════════════════════");
    print_table(&table);
    println!();

    // Sort by downloads (descending)
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Sorted by Downloads (Descending)");
    println!("═══════════════════════════════════════════════════════════════════════════");
    table.sort_by(|a, b| b.downloads.cmp(&a.downloads));
    print_table(&table);
    println!();

    // Sort by name (ascending)
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Sorted by Name (Ascending)");
    println!("═══════════════════════════════════════════════════════════════════════════");
    table.sort_by_column(0, SortDirection::Ascending);
    print_table(&table);
    println!();

    // Demonstrate pagination
    let mut paginated_table = Table::new()
        .with_column(Column::new("Package", |p: &Package| p.name.clone()).width(20))
        .with_column(Column::new("Version", |p: &Package| p.version.clone()).width(12))
        .with_data(table.data().to_vec())
        .with_pagination(2);

    println!("═══════════════════════════════════════════════════════════════════════════");
    println!(
        "  Paginated View (Page 1 of {})",
        paginated_table
            .pagination()
            .unwrap()
            .total_pages(paginated_table.row_count())
    );
    println!("═══════════════════════════════════════════════════════════════════════════");
    print_table(&paginated_table);
    println!();

    // Next page
    let row_count = paginated_table.row_count();
    paginated_table
        .pagination_mut()
        .unwrap()
        .next_page(row_count);
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!(
        "  Paginated View (Page 2 of {})",
        paginated_table
            .pagination()
            .unwrap()
            .total_pages(paginated_table.row_count())
    );
    println!("═══════════════════════════════════════════════════════════════════════════");
    print_table(&paginated_table);
    println!();

    // Selection demonstration
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  With Selection (Row 2 selected)");
    println!("═══════════════════════════════════════════════════════════════════════════");
    table.select_row(2);
    print_table_with_selection(&table);
    println!();

    println!("✅ Demo complete!");
}

fn print_table<T>(table: &Table<T>) {
    // Print headers
    print!("  ");
    for col in table.columns() {
        if col.is_visible() {
            let width = col.column_width().unwrap_or(15);
            print!("{:<width$}  ", col.name(), width = width);
        }
    }
    println!();

    // Print separator
    print!("  ");
    for col in table.columns() {
        if col.is_visible() {
            let width = col.column_width().unwrap_or(15);
            print!("{:<width$}  ", "─".repeat(width), width = width);
        }
    }
    println!();

    // Print rows (respecting pagination)
    for row_data in table.visible_rows() {
        print!("  ");
        for col in table.columns() {
            if col.is_visible() {
                let width = col.column_width().unwrap_or(15);
                let value = col.extract(row_data);
                print!("{:<width$}  ", value, width = width);
            }
        }
        println!();
    }
}

fn print_table_with_selection<T>(table: &Table<T>) {
    // Print headers
    print!("    ");
    for col in table.columns() {
        if col.is_visible() {
            let width = col.column_width().unwrap_or(15);
            print!("{:<width$}  ", col.name(), width = width);
        }
    }
    println!();

    // Print separator
    print!("    ");
    for col in table.columns() {
        if col.is_visible() {
            let width = col.column_width().unwrap_or(15);
            print!("{:<width$}  ", "─".repeat(width), width = width);
        }
    }
    println!();

    // Print rows with selection indicator
    for (idx, row_data) in table.visible_rows().iter().enumerate() {
        let is_selected = table.selected_row() == Some(idx);
        print!("  {} ", if is_selected { ">" } else { " " });

        for col in table.columns() {
            if col.is_visible() {
                let width = col.column_width().unwrap_or(15);
                let value = col.extract(row_data);
                print!("{:<width$}  ", value, width = width);
            }
        }
        println!();
    }
}
