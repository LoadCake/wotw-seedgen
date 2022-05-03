use std::str::FromStr;

use seedgen_derive::FromStr;

use crate::VItem;

use crate::header::{HeaderCommand, ParameterDefault, V, VString, ParameterType};
use crate::header::tokenizer::TokenKind;

use super::{Parser, ParseError, parse_ident, parse_number, parse_v_number, parse_string, parse_icon, Expectation};

#[derive(FromStr)]
#[ParseFromIdentifier]
enum HeaderCommandKind {
    Include,
    Exclude,
    Add,
    Remove,
    Name,
    Display,
    Description,
    Price,
    Icon,
    Parameter,
    Set,
    #[Ident = "if"] StartIf,
    EndIf,
}

impl HeaderCommand {
    pub fn parse(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
        let kind = parse_ident!(parser, Expectation::HeaderCommand);
        match kind {
            HeaderCommandKind::Include => parse_include(parser),
            HeaderCommandKind::Exclude => parse_exclude(parser),
            HeaderCommandKind::Add => parse_add(parser),
            HeaderCommandKind::Remove => parse_remove(parser),
            HeaderCommandKind::Name => parse_name(parser),
            HeaderCommandKind::Display => parse_display(parser),
            HeaderCommandKind::Description => parse_description(parser),
            HeaderCommandKind::Price => parse_price(parser),
            HeaderCommandKind::Icon => parse_icon_command(parser),
            HeaderCommandKind::Parameter => parse_parameter(parser),
            HeaderCommandKind::Set => parse_set(parser),
            HeaderCommandKind::StartIf => parse_if(parser),
            HeaderCommandKind::EndIf => Ok(HeaderCommand::EndIf),
        }
    }
}
impl FromStr for HeaderCommand {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(input);
        let command = HeaderCommand::parse(&mut parser).map_err(|err| err.verbose_display())?;
        let remaining = parser.remaining();
        if remaining.is_empty() {
            Ok(command)
        } else {
            Err(format!("Input left after parsing command: \"{remaining}\""))
        }
    }
}

fn parse_item_amount(parser: &mut Parser) -> Result<V<i32>, ParseError> {
    if match parser.current_token().kind {
        TokenKind::Number => {
            let peeked = parser.peek_token();
            if peeked.kind == TokenKind::Ident {
                let range = peeked.range.clone();
                parser.read(range) == "x"
            } else { false }
        },
        TokenKind::Dollar => true,
        _ => false,
    } {
        let amount = parse_v_number!(parser, Expectation::Integer);
        parser.next_token();
        parser.skip(TokenKind::Whitespace);
        return Ok(amount);
    }
    Ok(V::Literal(1))
}

fn parse_include(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let name = parse_ident!(parser, Expectation::Identifier);
    Ok(HeaderCommand::Include { name })
}
fn parse_exclude(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let name = parse_ident!(parser, Expectation::Identifier);
    Ok(HeaderCommand::Exclude { name })
}
fn parse_add(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let amount = parse_item_amount(parser)?;
    let item = VItem::parse(parser)?;
    Ok(HeaderCommand::Add { item, amount })
}
fn parse_remove(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let amount = parse_item_amount(parser)?;
    let item = VItem::parse(parser)?;
    Ok(HeaderCommand::Remove { item, amount })
}
fn parse_name(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let name = VString(parse_string(parser)?.to_owned());
    Ok(HeaderCommand::Name { item, name })
}
fn parse_display(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let name = VString(parse_string(parser)?.to_owned());
    Ok(HeaderCommand::Display { item, name })
}
fn parse_description(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let description = VString(parse_string(parser)?.to_owned());
    Ok(HeaderCommand::Description { item, description })
}
fn parse_price(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let price = parse_v_number!(parser, Expectation::Integer);
    Ok(HeaderCommand::Price { item, price })
}
fn parse_icon_command(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let icon = parse_icon(parser)?;
    Ok(HeaderCommand::Icon { item, icon })
}
fn parse_parameter(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let identifier = parse_ident!(parser, Expectation::Identifier);
    parser.eat(TokenKind::Whitespace)?;
    let parameter_type = parse_ident!(parser, Expectation::ParameterType);
    parser.eat(TokenKind::Colon)?;
    let default = match parameter_type {
        ParameterType::Bool => ParameterDefault::Bool(parse_ident!(parser, Expectation::Boolean)),
        ParameterType::Int => ParameterDefault::Int(parse_number!(parser, Expectation::Integer)),
        ParameterType::Float => ParameterDefault::Float(parse_number!(parser, Expectation::Float)),
        ParameterType::String => ParameterDefault::String(parse_string(parser)?.to_owned()),
    };
    Ok(HeaderCommand::Parameter { identifier, default })
}
fn parse_set(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let mut state = String::new();
    loop {
        let token = parser.eat(TokenKind::Ident)?;
        state.push_str(parser.read_token(&token));
        if parser.current_token().kind == TokenKind::Dot {
            state.push('.');
            parser.next_token();
        } else {
            break;
        }
    }
    Ok(HeaderCommand::Set { state })
}
fn parse_if(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat(TokenKind::Whitespace)?;
    let parameter = parse_ident!(parser, Expectation::Identifier);
    parser.eat(TokenKind::Whitespace)?;
    let token = parser.next_token();
    let value = parser.read_token(&token).to_owned();
    Ok(HeaderCommand::If { parameter, value })
}