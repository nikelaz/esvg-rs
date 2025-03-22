use cssparser::{Parser, ParserInput, RuleListParser, QualifiedRuleParser, DeclarationListParser};
use cssparser::{Token, CowRcStr};
use std::collections::HashMap;

#[derive(Debug)]
struct CssProperty {
  name: String,
  value: String,
}

#[derive(Debug)]
struct CssRule {
  selector: String,
  properties: Vec<CssProperty>,
}

struct CSSParser;

impl<'i> QualifiedRuleParser<'i> for CSSParser {
  type Prelude = String;  
  type QualifiedRule = CssRule;
  type Error = ();

  fn parse_prelude<'t>(&mut self, parser: &mut Parser<'i, 't>) -> Result<Self::Prelude, cssparser::BasicParseError<'i>> {
    // Convert the selector tokens to a string
    let mut selector = String::new();
    while let Ok(token) = parser.next_including_whitespace_and_comments() {
      match token {
        Token::Delim('{') => break,
        token => selector.push_str(&token.to_css_string()),
      }
    }
    Ok(selector.trim().to_string())
  }

  fn parse_block<'t>(
    &mut self,
    prelude: Self::Prelude,
    parser: &mut Parser<'i, 't>,
  ) -> Result<Self::QualifiedRule, cssparser::BasicParseError<'i>> {
    let mut properties = Vec::new();
        
    // Parse declarations (properties) within the rule
    let decl_parser = DeclarationListParser::new(parser, DeclarationParser);
    for decl in decl_parser {
      if let Ok(prop) = decl {
        properties.push(prop);
      }
    }

    Ok(CssRule {
      selector: prelude,
      properties,
    })
  }
}

struct DeclarationParser;

impl<'i> cssparser::DeclarationParser<'i> for DeclarationParser {
  type Declaration = CssProperty;
  type Error = ();

  fn parse_value<'t>(
    &mut self,
    name: CowRcStr<'i>,
    parser: &mut Parser<'i, 't>,
  ) -> Result<Self::Declaration, cssparser::BasicParseError<'i>> {
    let mut value = String::new();
    while let Ok(token) = parser.next_including_whitespace_and_comments() {
      value.push_str(&token.to_css_string());
    }
    
    Ok(CssProperty {
      name: name.to_string(),
      value: value.trim().to_string(),
    })
  }
}

pub fn parse_css(css: &str) -> Vec<CssRule> {
  let mut input = ParserInput::new(css);
  let mut parser = Parser::new(&mut input);
    
  let rule_parser = RuleListParser::new_for_stylesheet(&mut parser, CSSParser);
  rule_parser.filter_map(Result::ok).collect()
}

