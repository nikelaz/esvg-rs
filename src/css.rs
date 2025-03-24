use regex::Regex;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct CSSProp {
  pub name: String,
  pub value: String
}

impl CSSProp {
    pub fn to_string(&self) -> String {
        return format!("{}: {};", self.name, self.value);
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct CSSRule {
  pub selector: String,
  pub props: Vec<CSSProp>
}

#[derive(Debug)]
#[derive(Clone)]
pub struct CSSPropsList {
    pub list: Vec<CSSProp>
}

impl CSSPropsList {
    pub fn new(input_str: &str) -> Self {
        let props = CSSParser::parse_props(input_str).unwrap();

        CSSPropsList {
            list: props
        }
    }

    pub fn remove(&mut self, key: &str) {
        let mut props_to_remove = Vec::new();

        for prop in &self.list {
            if prop.name == key {
                props_to_remove.push(prop.name.clone());
            }
        }

        self.list.retain(|x| !props_to_remove.contains(&x.name));
    }

    pub fn to_string(&mut self) -> String {
        self.list
            .iter()
            .map(|prop| prop.to_string())
            .collect::<Vec<_>>() 
            .join(" ")
    }
}

pub struct CSSParser;

impl CSSParser {
    pub fn from_string(input_str: &str) -> Result<Vec<CSSRule>, String> {
        let rules = CSSParser::parse_rules(input_str)?;
        Ok(rules)
    }

    pub fn parse_rules(input_str: &str) -> Result<Vec<CSSRule>, String> {
        let re = Regex::new(r"(?s)([^{}]+)\s*\{([^}]*)\}").map_err(|e| e.to_string())?;
        let mut results = Vec::new();

        for cap in re.captures_iter(input_str) {
            let selector = cap[1].trim().to_string();
            let body = cap[2].trim().to_string();
            let css_rule = CSSRule {
                selector,
                props: CSSParser::parse_props(body.as_str())?,
            };
            results.push(css_rule);
        }

        Ok(results)
    }

    pub fn parse_props(css_block: &str) -> Result<Vec<CSSProp>, String> {
        let pattern = Regex::new(r"\s*([^:]+)\s*:\s*([^;]+)\s*;?").map_err(|e| e.to_string())?;
        let mut props = Vec::new();

        for cap in pattern.captures_iter(css_block) {
            let prop_name = cap[1].trim().to_string();
            let prop_value = cap[2].trim().to_string();
            let prop = CSSProp {
                name: prop_name,
                value: prop_value,
            };
            props.push(prop);
        }

        Ok(props)
    }
}
