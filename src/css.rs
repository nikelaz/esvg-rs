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

pub struct InlineStyle {
    pub props: Vec<CSSProp>,
}

impl InlineStyle {
    pub fn from_string(input_str: &str) -> Result<Self, String> {
        let props = CSSParser::parse_props(input_str)?;
        Ok(InlineStyle { props })
    }

    pub fn remove_prop(&mut self, prop_name: &str) {
        self.props.retain(|prop| prop.name != prop_name);
    }

    pub fn to_string(&self) -> String {
        self.props
            .iter()
            .map(|prop| prop.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

pub struct CSSParser;

impl CSSParser {
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

