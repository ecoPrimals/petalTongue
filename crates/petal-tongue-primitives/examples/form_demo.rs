// Form Primitive Demo
//
// Demonstrates the Form primitive with validation across different field types

use petal_tongue_primitives::common::Color;
use petal_tongue_primitives::form::{Field, FieldType, Form};

#[derive(Debug, Clone)]
struct UserProfile {
    username: String,
    age: i64,
    email: String,
    bio: String,
    theme: String,
    notifications: bool,
    privacy_level: String,
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    FORM PRIMITIVE DEMO                       ║");
    println!("║          Generic Form Builder with Validation               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Example 1: User Profile Form
    println!("━━━ Example 1: User Profile Form ━━━");
    demo_user_profile_form();

    // Example 2: Validation Examples
    println!("\n━━━ Example 2: Validation Examples ━━━");
    demo_validation();

    // Example 3: All Field Types
    println!("\n━━━ Example 3: All Field Types ━━━");
    demo_all_field_types();

    println!("\n✅ Form primitive demo complete!");
}

fn demo_user_profile_form() {
    let mut form = Form::<UserProfile>::new("User Profile")
        .with_field(
            Field::text("username", "Username")
                .required()
                .with_help("Choose a unique username"),
        )
        .with_field(Field {
            id: "age".to_string(),
            label: "Age".to_string(),
            field_type: FieldType::Integer {
                default: 0,
                min: Some(13),
                max: Some(120),
                step: Some(1),
            },
            required: true,
            help_text: Some("You must be 13 or older".to_string()),
            extractor: None,
            setter: None,
        })
        .with_field(
            Field::text("email", "Email")
                .required()
                .with_help("We'll never share your email"),
        )
        .with_field(Field {
            id: "bio".to_string(),
            label: "Bio".to_string(),
            field_type: FieldType::TextArea {
                default: String::new(),
                rows: 5,
                max_length: Some(500),
            },
            required: false,
            help_text: Some("Tell us about yourself (max 500 chars)".to_string()),
            extractor: None,
            setter: None,
        })
        .with_field(Field::select(
            "theme",
            "Theme",
            vec!["Light".to_string(), "Dark".to_string(), "Auto".to_string()],
        ))
        .with_field(Field::checkbox("notifications", "Enable Notifications"));

    println!("Form created: {}", form.title);
    println!("Field count: {}", form.field_count());
    println!("Required fields: {}", form.required_fields().len());

    // Initialize with some values
    form.set_value("username", "alice_wonder");
    form.set_value("age", "25");
    form.set_value("email", "alice@example.com");
    form.set_value("notifications", "true");

    println!("\n📝 Form data:");
    for (key, value) in &form.data {
        println!("  {}: {}", key, value);
    }

    // Validate
    if form.validate() {
        println!("\n✅ Form is valid!");
    } else {
        println!("\n❌ Form has errors:");
        for error in &form.errors {
            println!("  - {}: {}", error.field_id, error.message);
        }
    }
}

fn demo_validation() {
    let mut form = Form::<UserProfile>::new("Validation Test");

    // Required field validation
    form = form.with_field(Field::text("required_field", "Required Field").required());

    // Text length validation
    form = form.with_field(Field {
        id: "short_text".to_string(),
        label: "Short Text".to_string(),
        field_type: FieldType::Text {
            default: String::new(),
            max_length: Some(10),
            pattern: None,
        },
        required: false,
        help_text: None,
        extractor: None,
        setter: None,
    });

    // Number range validation
    form = form.with_field(Field {
        id: "percentage".to_string(),
        label: "Percentage".to_string(),
        field_type: FieldType::Number {
            default: 0.0,
            min: Some(0.0),
            max: Some(100.0),
            step: Some(0.1),
        },
        required: false,
        help_text: None,
        extractor: None,
        setter: None,
    });

    println!("1. Testing empty required field:");
    form.validate();
    println!("  Errors: {}", form.errors.len());
    for error in &form.errors {
        println!("    - {}", error.message);
    }

    println!("\n2. Testing text too long:");
    form.set_value("required_field", "filled");
    form.set_value("short_text", "This text is way too long");
    form.validate();
    println!("  Errors: {}", form.errors.len());
    for error in &form.errors {
        println!("    - {}", error.message);
    }

    println!("\n3. Testing number out of range:");
    form.set_value("short_text", "short");
    form.set_value("percentage", "150.0");
    form.validate();
    println!("  Errors: {}", form.errors.len());
    for error in &form.errors {
        println!("    - {}", error.message);
    }

    println!("\n4. Testing all valid:");
    form.set_value("percentage", "75.5");
    if form.validate() {
        println!("  ✅ All validations passed!");
    }
}

fn demo_all_field_types() {
    let form = Form::<UserProfile>::new("All Field Types Showcase")
        .with_field(Field::text("text_field", "Text Field"))
        .with_field(Field {
            id: "textarea_field".to_string(),
            label: "Text Area Field".to_string(),
            field_type: FieldType::TextArea {
                default: String::new(),
                rows: 3,
                max_length: None,
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        })
        .with_field(Field::number("number_field", "Number Field"))
        .with_field(Field {
            id: "integer_field".to_string(),
            label: "Integer Field".to_string(),
            field_type: FieldType::Integer {
                default: 0,
                min: None,
                max: None,
                step: None,
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        })
        .with_field(Field::select(
            "select_field",
            "Select Field",
            vec![
                "Option 1".to_string(),
                "Option 2".to_string(),
                "Option 3".to_string(),
            ],
        ))
        .with_field(Field {
            id: "multiselect_field".to_string(),
            label: "Multi-Select Field".to_string(),
            field_type: FieldType::MultiSelect {
                options: vec!["A".to_string(), "B".to_string(), "C".to_string()],
                default_selected: vec![],
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        })
        .with_field(Field::checkbox("checkbox_field", "Checkbox Field"))
        .with_field(Field {
            id: "radio_field".to_string(),
            label: "Radio Field".to_string(),
            field_type: FieldType::Radio {
                options: vec!["Yes".to_string(), "No".to_string(), "Maybe".to_string()],
                default_index: None,
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        })
        .with_field(Field {
            id: "slider_field".to_string(),
            label: "Slider Field".to_string(),
            field_type: FieldType::Slider {
                min: 0.0,
                max: 100.0,
                default: 50.0,
                step: Some(5.0),
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        })
        .with_field(Field {
            id: "color_field".to_string(),
            label: "Color Field".to_string(),
            field_type: FieldType::Color {
                default: Color::rgb(100, 150, 200),
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

    println!("Total field types: {}", form.fields.len());
    println!("\nField types demonstrated:");
    for (i, field) in form.fields.iter().enumerate() {
        let field_type_name = match &field.field_type {
            FieldType::Text { .. } => "Text",
            FieldType::TextArea { .. } => "TextArea",
            FieldType::Number { .. } => "Number",
            FieldType::Integer { .. } => "Integer",
            FieldType::Select { .. } => "Select",
            FieldType::MultiSelect { .. } => "MultiSelect",
            FieldType::Checkbox { .. } => "Checkbox",
            FieldType::Radio { .. } => "Radio",
            FieldType::Slider { .. } => "Slider",
            FieldType::Color { .. } => "Color",
        };
        println!("  {}. {} ({})", i + 1, field.label, field_type_name);
    }
}
