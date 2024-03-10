use crate::{PairMap, StringPairMap};
use rand::Rng;

pub struct Field {
    pub name: String,
    pub value: String,
}

pub struct FieldBuilder {
    name: Option<String>,
    value: Option<String>,
}

impl FieldBuilder {
    pub fn new() -> FieldBuilder {
        FieldBuilder {
            name: None,
            value: None,
        }
    }

    pub fn name(&mut self, name: &str) -> &FieldBuilder {
        self.name = Some(name.to_string());
        self
    }

    pub fn value(&mut self, value: &str) -> &FieldBuilder {
        self.value = Some(value.to_string());
        self
    }

    pub fn build(self) -> Field {
        Field {
            name: self
                .name
                .unwrap_or_else(|| panic!("field name is required")),
            value: self
                .value
                .unwrap_or_else(|| panic!("field value is required")),
        }
    }
}

pub struct Form {
    pub boundary: String,
    pub fields: Vec<Field>,
}

impl From<&str> for Form {
    fn from(input: &str) -> Form {
        let mut form = Form::new();

        let mut lines = input.lines().filter(|line| !line.is_empty());
        let boundary = lines.next().unwrap();
        let boundary = boundary.trim_start_matches("--");

        if !boundary.starts_with("quartz") {
            eprintln!(
                "invalid form. Expected boundary to start with 'quartz' but got '{}'",
                boundary
            );

            return form;
        }

        form.boundary = boundary.to_string();

        while let Some(line) = lines.next() {
            if line.starts_with("--") {
                continue;
            }

            if line.starts_with("Content-Disposition") {
                let mut builder = FieldBuilder::new();

                let mut idx = line.find("name=").unwrap();
                idx += "name=".len();

                let name = &line[idx..];
                let name = name.trim_matches('"');
                builder.name(name);

                let mut value = Vec::new();
                while let Some(line) = lines.next() {
                    if line.contains(&form.boundary()) {
                        break;
                    }

                    value.push(line);
                }

                builder.value(&value.join("\n"));
                form.fields.push(builder.build());
            }
        }

        form
    }
}

impl Form {
    pub fn new() -> Form {
        Form {
            boundary: Self::gen_boundary(),
            fields: Vec::new(),
        }
    }

    pub fn gen_boundary() -> String {
        let mut boundary = String::from("quartz");

        for _ in 0..16 {
            let idx = rand::thread_rng().gen_range(0..=61);
            let c = match idx {
                0..=25 => (b'a' + idx as u8) as char,
                26..=51 => (b'A' + (idx - 26) as u8) as char,
                _ => (b'0' + (idx - 52) as u8) as char,
            };

            boundary.push(c);
        }

        boundary
    }

    pub fn insert(&mut self, input: &str) {
        let (name, value) = StringPairMap::pair(input)
            .unwrap_or_else(|| panic!("malformed key-value pair. Expected <key>=<value>"));

        if name.contains(&self.boundary) || value.contains(&self.boundary) {
            self.boundary = Self::gen_boundary();
        }

        self.fields.push(Field { name, value });
    }

    pub fn boundary(&self) -> &str {
        &self.boundary
    }

    pub fn content_type(&self) -> String {
        format!("multipart/form-data; boundary={}", self.boundary())
    }

    pub fn into_body(self) -> String {
        let mut result = String::new();

        for field in self.fields.iter() {
            result.push_str("\r\n");
            result.push_str("--");
            result.push_str(&self.boundary());
            result.push_str("\r\n");
            result.push_str(
                format!("Content-Disposition: form-data; name=\"{}\"", field.name).as_str(),
            );
            result.push_str("\r\n\r\n");
            result.push_str(field.value.as_str());
        }

        result.push_str("\r\n");
        result.push_str("--");
        result.push_str(&self.boundary());
        result.push_str("--");

        result
    }
}
