#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_csv_parsing() {
        let csv_content = "style,subject,lighting\nwatercolor,cat,soft\noil painting,dog,dramatic";
        let result = parse_csv(csv_content).unwrap();

        assert_eq!(result.columns.len(), 3);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("style").unwrap(), "watercolor");
        assert_eq!(result.rows[0].get("subject").unwrap(), "cat");
    }

    #[test]
    fn test_json_array_parsing() {
        let json_content = r#"[
            {"style": "watercolor", "subject": "cat"},
            {"style": "oil painting", "subject": "dog"}
        ]"#;
        let result = parse_json(json_content).unwrap();

        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_json_object_parsing() {
        let json_content = r#"{
            "style": ["watercolor", "oil painting"],
            "subject": ["cat", "dog"]
        }"#;
        let result = parse_json(json_content).unwrap();

        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_template_rendering() {
        let mut rows = Vec::new();
        let mut row1 = HashMap::new();
        row1.insert("style".to_string(), "watercolor".to_string());
        row1.insert("subject".to_string(), "cat".to_string());
        rows.push(row1);

        let mut row2 = HashMap::new();
        row2.insert("style".to_string(), "oil painting".to_string());
        row2.insert("subject".to_string(), "dog".to_string());
        rows.push(row2);

        let template = "A {{ style }} painting of {{ subject }}";
        let result = render_template(template, rows);

        assert_eq!(result.rendered.len(), 2);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.rendered[0], "A watercolor painting of cat");
        assert_eq!(result.rendered[1], "A oil painting of dog");
    }

    #[test]
    fn test_template_with_filter() {
        let mut rows = Vec::new();
        let mut row1 = HashMap::new();
        row1.insert("subject".to_string(), "cat".to_string());
        rows.push(row1);

        let template = "{{ subject | upper }}";
        let result = render_template(template, rows);

        assert_eq!(result.rendered.len(), 1);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.rendered[0], "CAT");
    }

    #[test]
    fn test_template_with_missing_variable() {
        let mut rows = Vec::new();
        let mut row1 = HashMap::new();
        row1.insert("style".to_string(), "watercolor".to_string());
        rows.push(row1);

        let template = "A {{ style }} painting of {{ missing_variable }}";
        let result = render_template(template, rows);

        assert_eq!(result.rendered.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].row, 0);
        assert!(result.errors[0].error.contains("undefined"));
    }

    #[test]
    fn test_template_syntax_error() {
        let mut rows = Vec::new();
        let mut row1 = HashMap::new();
        row1.insert("style".to_string(), "watercolor".to_string());
        rows.push(row1);

        let template = "A {{ style }} painting of {{ subject";  // Missing closing }}
        let result = render_template(template, rows);

        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].error.contains("syntax"));
    }
}
