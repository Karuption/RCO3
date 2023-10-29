use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{alphanumeric1, multispace0, space1, u32};
use nom::combinator::opt;
use nom::multi::separated_list0;
use nom::sequence::{preceded, terminated};
use nom::IResult;

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
    let (rest, quit_msg) = take_until("\n")(input)?;
    Ok((rest, Command::Quit(quit_msg.to_string())))
}

fn parse_join(input: &str) -> IResult<&str, Command> {
    let (rest, list) =
        separated_list0(tag(","), preceded(tag("#"), alphanumeric1))(input).map(|(x, y)| {
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
    // only parse USER <user> <mode> <unused> <real name> and only using username anyways
    Ok((rest, Command::User(username.to_string())))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Cap(String),
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
        Command::Join(vec!["test".to_string()], None)
    );

    assert_eq!(
        parse_command("JOIN #test,#test2".to_string()).unwrap(),
        Command::Join(vec!["test".to_string(), "test2".to_string()], None)
    );
}
