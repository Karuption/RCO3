use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::{alphanumeric1, line_ending, multispace0, newline, space1, u32};
use nom::character::is_newline;
use nom::combinator::{not, opt};
use nom::multi::separated_list0;
use nom::sequence::{preceded, terminated};
use nom::{IResult, Parser};
use std::error::Error;

pub fn parse_command(input: String) -> Result<Command, String> {
    let inp = input.as_str();
    alt((
        preceded(tag("CAP "), parse_cap),
        preceded(tag("JOIN "), parse_join),
        preceded(tag("NICK "), parse_nick),
        preceded(tag("USER "), parse_user),
        preceded(tag("QUIT "), parse_quit),
    ))(inp)
    .map(|x| x.1)
    .map_err(|x| x.to_string())
}

#[derive(Debug)]
pub enum Command {
    CAP(String),
    Join(Vec<String>, Option<String>),
    List(String),
    Names(String),
    Nick(String, u32),
    Ping,
    Pong,
    Quit(String),
    Topic(String),
    User(String),
}

fn parse_cap(input: &str) -> IResult<&str, Command> {
    Ok((input, Command::CAP(input.to_string())))
}

fn parse_quit(input: &str) -> IResult<&str, Command> {
    let (rest, quit_msg) = take_until("\n")(input)?;
    Ok((rest, Command::Quit(quit_msg.to_string())))
}

fn parse_join(input: &str) -> IResult<&str, Command> {
    let (rest, list) = separated_list0(tag("#"), alphanumeric1)(input).map(|(x, y)| {
        let channels: Vec<String> = y.iter().map(|x| x.to_string()).collect();
        (x, channels)
    })?;

    Ok((rest, Command::Join(list, None)))
}

fn parse_nick(input: &str) -> IResult<&str, Command> {
    let (rest, username) = alphanumeric1(input)?;
    let (rest, hop_count) = opt(preceded(space1, u32))(rest).map(|x| (x.0, x.1.unwrap_or(0u32)))?;

    Ok((rest, Command::Nick(username.to_string(), hop_count)))
}

fn parse_user(input: &str) -> IResult<&str, Command> {
    let (rest, username) = terminated(alphanumeric1, multispace0)(input)?;
    // only parse USER <user> <mode> <unused> <realname> and only using username anyways
    Ok((rest, Command::User(username.to_string())))
}
