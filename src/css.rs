use regex::Regex;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct CSSProp {
  pub name: String,
  pub value: String
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct CSSRule {
  pub selector: String,
  pub props: Vec<CSSProp>
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
