use combine::{ParseResult, parser, Parser};
use combine::{choice, many1};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Item};
use helpers::{semi, ident, string};
use tokenizer::TokenStream;
use grammar::{value, bool, Code};


pub fn directives<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    choice((
        ident("proxy_pass").with(parser(value)).skip(semi())
            .map(Item::ProxyPass),
        ident("proxy_set_header").with(parser(value)).and(parser(value))
            .skip(semi())
            .map(|(field, value)| Item::ProxySetHeader { field, value }),
        ident("proxy_method").with(parser(value)).skip(semi())
            .map(Item::ProxyMethod),
        ident("proxy_cache").with(parser(value)).skip(semi())
            .map(Item::ProxyCache),
        ident("proxy_cache_key").with(parser(value)).skip(semi())
            .map(Item::ProxyCacheKey),
        ident("proxy_cache_valid")
            .with(many1(parser(value)))
            .and_then(|mut v: Vec<_>| {
                use ast::ProxyCacheValid::*;
                use value::Item::*;
                let time = v.pop().unwrap();
                if v.len() == 0 {
                    return Ok(Normal(time));
                }
                let mut codes = Vec::new();
                let items = v.len();
                for item in v {
                    match &item.data[..] {
                        [Literal(x)] if x == "any" => {
                            if items == 1 {
                                return Ok(Any(time));
                            } else {
                                return Err(Error::unexpected_message(
                                    "`any` must be sole argument before time. \
                                     It's not allowed to combine `any` and \
                                     other codes"));
                            }
                        }
                        [Literal(x)] => {
                            match Code::parse(x) {
                                Ok(code) => {
                                    codes.push(code.as_code())
                                }
                                Err(_) => {
                                    return Err(Error::unexpected_message(
                                        format!("invalid http code {:?}", x)));
                                }
                            }
                        }
                        _ => {
                            return Err(Error::unexpected_message(
                                "variables aren't allowed in list of codes"));
                        }
                    }
                }
                return Ok(Specific(codes, time));
            })
            .skip(semi()).map(Item::ProxyCacheValid),
        ident("proxy_read_timeout").with(parser(value)).skip(semi())
            .map(Item::ProxyReadTimeout),
        ident("proxy_connect_timeout").with(parser(value)).skip(semi())
            .map(Item::ProxyConnectTimeout),
        ident("proxy_hide_header").with(parser(value)).skip(semi())
            .map(Item::ProxyHideHeader),
        ident("proxy_pass_header").with(parser(value)).skip(semi())
            .map(Item::ProxyPassHeader),
        ident("proxy_pass_request_headers").with(parser(bool)).skip(semi())
            .map(Item::ProxyPassRequestHeaders),
        ident("proxy_pass_request_body").with(parser(bool)).skip(semi())
            .map(Item::ProxyPassRequestBody),
        ident("proxy_intercept_errors").with(parser(bool)).skip(semi())
            .map(Item::ProxyInterceptErrors),
        ident("proxy_buffering").with(parser(bool)).skip(semi())
            .map(Item::ProxyBuffering),
        ident("proxy_ignore_headers").with(many1(string())).skip(semi())
            .map(|v: Vec<_>| {
                v.into_iter().map(|v| v.value.to_string()).collect()
            })
            .map(Item::ProxyIgnoreHeaders),
        ident("proxy_http_version")
            .with(string()).and_then(|v| {
                match v.value {
                    "1.0" => Ok(ast::ProxyHttpVersion::V1_0),
                    "1.1" => Ok(ast::ProxyHttpVersion::V1_1),
                    _ => Err(Error::unexpected_message("invalid variable")),
                }
            })
            .skip(semi())
            .map(Item::ProxyHttpVersion),
        ident("proxy_next_upstream_tries").with(parser(value)).skip(semi())
            .map(Item::ProxyNextUpstreamTries),
        ident("proxy_next_upstream_timeout").with(parser(value)).skip(semi())
            .map(Item::ProxyNextUpstreamTimeout),
    )).parse_stream(input)
}
