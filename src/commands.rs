use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1, take_until, take_while1};
use nom::character::complete::{
    alphanumeric1, multispace0, multispace1, not_line_ending, space1, u32,
};
use nom::combinator::{all_consuming, not, opt, recognize};
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{preceded, terminated};
use nom::IResult;

use crate::commands;

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

fn parse_cap(input: &str) -> IResult<&str, Command> {
    Ok((input, Command::Cap(input.to_string())))
}

fn parse_quit(input: &str) -> IResult<&str, Command> {
    let (rest, quit_msg) = preceded(tag(":"), not_line_ending)(input)?;
    Ok((
        rest,
        Command::Quit(Quit {
            msg: quit_msg.to_string(),
        }),
    ))
}

fn parse_join(input: &str) -> IResult<&str, Command> {
    let (rest, list) = separated_list1(
        tag(","),
        recognize(preceded(alt((tag("#"), tag("&"))), alphanumeric1)),
    )(input)
    .map(|(x, y)| {
        let channels: Vec<String> = y.iter().map(|x| x.to_string()).collect();
        (x, channels)
    })?;

    let (rest, keys) = parse_join_keys(rest)?;

    Ok((rest, Command::Join(list, keys)))
}

fn parse_join_keys(input: &str) -> IResult<&str, Option<Vec<String>>> {
    opt(preceded(
        multispace1,
        separated_list0(
            tag(","),
            take_while1(|x: char| !x.is_whitespace() && x != ','),
        ),
    ))(input)
    .map(|(_rest, opt)| {
        (
            _rest,
            opt.map(|v| {
                v.iter()
                    .map(|pass| {
                        let p = pass.to_string();
                        println!("{:?}", &p);
                        p
                    })
                    .collect()
            }),
        )
    })
}

fn parse_nick(input: &str) -> IResult<&str, Command> {
    let (rest, username) = alphanumeric1(input)?;
    let (rest, hop_count) = opt(preceded(space1, u32))(rest).map(|x| (x.0, x.1.unwrap_or(0u32)))?;

    Ok((rest, Command::Nick(username.to_string(), hop_count)))
}

fn parse_user(input: &str) -> IResult<&str, Command> {
    let (rest, username) = terminated(alphanumeric1, multispace0)(input)?;
    // only parse USER <user> <mode> <unused> <real name> and only using username anyways
    Ok((rest, Command::User(username.to_string())))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Cap(String),
    Join(Vec<String>, Option<Vec<String>>),
    List(String),
    Names(String),
    Nick(String, u32),
    Ping,
    Pong,
    Quit(Quit),
    Topic(String),
    User(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Quit {
    msg: String,
}

impl Quit {
    pub fn get_msg(&self) -> &str {
        &self.msg
    }
}

impl Command {
    pub(crate) fn write_value(&self) -> std::io::Result<String> {
        match self {
            Command::Cap(_) => Ok("CAP End".to_string()),
            // Command::Join(_, _) => {}
            // Command::List(_) => {}
            // Command::Names(_) => {}
            // Command::Nick(_, _) => {}
            // Command::Ping => {}
            // Command::Pong => {}
            // Command::Quit(_) => {}
            // Command::Topic(_) => {}
            // Command::User(_) => {}
            _ => todo!(),
        }
    }
}

#[test]
fn parse_nick_test() {
    assert_eq!(
        parse_command("NICK Somebody1 0".to_string()).unwrap(),
        Command::Nick("Somebody1".to_string(), 0u32)
    );

    assert_eq!(
        parse_command("NICK Somebody1".to_string()).unwrap(),
        Command::Nick("Somebody1".to_string(), 0)
    );
}

#[test]
fn parse_join_test() {
    assert_eq!(
        parse_command("JOIN #test".to_string()).unwrap(),
        Command::Join(vec!["#test".to_string()], None)
    );

    assert_eq!(
        parse_command("JOIN #test,#test2".to_string()).unwrap(),
        Command::Join(vec!["#test".to_string(), "#test2".to_string()], None)
    );

    assert_eq!(
        parse_command("JOIN #test,&test2".to_string()).unwrap(),
        Command::Join(vec!["#test".to_string(), "&test2".to_string()], None)
    );

    assert_eq!(
        parse_command("JOIN #test,&test2 key1".to_string()).unwrap(),
        Command::Join(
            vec!["#test".to_string(), "&test2".to_string()],
            Some(vec!["key1".to_string()])
        )
    );

    assert_eq!(
        parse_command("JOIN #test,&test2 key1,key2".to_string()).unwrap(),
        Command::Join(
            vec!["#test".to_string(), "&test2".to_string()],
            Some(vec!["key1".to_string(), "key2".to_string()])
        )
    );
}

#[test]
fn parse_user_test() {
    assert_eq!(
        parse_command("USER Username 0 * :realname\r\n".to_string()).unwrap(),
        Command::User("Username".to_string())
    );
}

#[test]
fn parse_quit_test() {
    assert_eq!(
        parse_command("QUIT :asdf !5^*%".to_string()).unwrap(),
        Command::Quit(Quit {
            msg: "asdf !5^*%".to_string()
        })
    );
}
