use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while1};
use nom::character::complete::{alphanumeric1, multispace1, not_line_ending, space1, u32};
use nom::combinator::{opt, recognize};
use nom::error::ParseError;
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{preceded, terminated};
use nom::{Err, IResult, Parser};

pub fn parse_command(input: String) -> Result<ClientCommand, String> {
    alt((
        parse_command_prefix("CAP", parse_cap),
        parse_command_prefix("Ping", parse_ping),
        parse_command_prefix("JOIN", parse_join),
        parse_command_prefix("NICK", parse_nick),
        parse_command_prefix("USER", parse_user),
        parse_command_prefix("QUIT", parse_quit),
    ))(input.as_str())
    .map(|x| x.1)
    .map_err(|x| x.to_string())
}
fn parse_command_prefix<'a, E, G>(
    command_prefix: &'a str,
    command_body_parser: G,
) -> impl FnMut(&'a str) -> Result<(&'a str, ClientCommand), Err<E>>
where
    E: ParseError<&'a str>,
    G: Parser<&'a str, ClientCommand, E>,
{
    preceded(
        alt((tag("\n"), tag("\r\n"), tag("\\"), tag(""))),
        preceded(terminated(tag(command_prefix), space1), command_body_parser),
    )
}

fn parse_cap(input: &str) -> IResult<&str, ClientCommand> {
    Ok((input, ClientCommand::Cap(input.to_string())))
}

fn parse_quit(input: &str) -> IResult<&str, ClientCommand> {
    let (rest, quit_msg) = preceded(tag(":"), not_line_ending)(input)?;
    Ok((
        rest,
        ClientCommand::Quit(Quit {
            msg: quit_msg.to_string(),
        }),
    ))
}

fn parse_join(input: &str) -> IResult<&str, ClientCommand> {
    let (rest, list) = separated_list1(
        tag(","),
        recognize(preceded(alt((tag("#"), tag("&"))), alphanumeric1)),
    )(input)
    .map(|(x, y)| {
        let channels: Vec<String> = y.iter().map(|x| x.to_string()).collect();
        (x, channels)
    })?;

    let (rest, keys) = parse_join_keys(rest)?;

    Ok((rest, ClientCommand::Join(list, keys)))
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

fn parse_nick(input: &str) -> IResult<&str, ClientCommand> {
    let (rest, username) = alphanumeric1(input)?;
    let (rest, hop_count) = opt(preceded(space1, u32))(rest).map(|x| (x.0, x.1.unwrap_or(0u32)))?;

    Ok((rest, ClientCommand::Nick(username.to_string(), hop_count)))
}

fn parse_user(input: &str) -> IResult<&str, ClientCommand> {
    let (rest, user) = terminated(alphanumeric1, space1)(input)?;
    // only parse USER <user> <mode> <unused> <real name> and only using username anyways

    let (rest, mode) = terminated(
        terminated(
            terminated(u32, space1),
            take_while1(|x: char| !x.is_whitespace()),
        ),
        space1,
    )(rest)?;

    let (rest, real_name) = preceded(
        tag(":"),
        take_while1(|x: char| x.is_alphanumeric() || x == ' '),
    )(rest)?;

    Ok((
        rest,
        ClientCommand::User(User {
            user: user.to_string(),
            mode,                             //mode,
            real_name: real_name.to_string(), //real_name.to_string(),
        }),
    ))
}

fn parse_ping(input: &str) -> IResult<&str, ClientCommand> {
    let (rest, token) = is_not(" \t\r\n")(input)?;
    Ok((rest, ClientCommand::Ping(token.to_string())))
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClientCommand {
    Cap(String),
    Join(Vec<String>, Option<Vec<String>>),
    List(String),
    Names(String),
    Nick(String, u32),
    Ping(String),
    Pong,
    Quit(Quit),
    Topic(String),
    User(User),
}

#[derive(Debug, PartialEq, Eq)]
pub struct User {
    pub user: String,
    pub mode: u32,
    pub real_name: String,
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

impl ClientCommand {
    pub(crate) fn write_value(&self) -> std::io::Result<String> {
        match self {
            ClientCommand::Cap(_) => Ok("CAP End".to_string()),
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
        ClientCommand::Nick("Somebody1".to_string(), 0u32)
    );

    assert_eq!(
        parse_command("NICK Somebody1".to_string()).unwrap(),
        ClientCommand::Nick("Somebody1".to_string(), 0)
    );
}

#[test]
fn parse_join_test() {
    assert_eq!(
        parse_command("JOIN #test".to_string()).unwrap(),
        ClientCommand::Join(vec!["#test".to_string()], None)
    );

    assert_eq!(
        parse_command("JOIN #test,#test2".to_string()).unwrap(),
        ClientCommand::Join(vec!["#test".to_string(), "#test2".to_string()], None)
    );

    assert_eq!(
        parse_command("JOIN #test,&test2".to_string()).unwrap(),
        ClientCommand::Join(vec!["#test".to_string(), "&test2".to_string()], None)
    );

    assert_eq!(
        parse_command("JOIN #test,&test2 key1".to_string()).unwrap(),
        ClientCommand::Join(
            vec!["#test".to_string(), "&test2".to_string()],
            Some(vec!["key1".to_string()])
        )
    );

    assert_eq!(
        parse_command("JOIN #test,&test2 key1,key2".to_string()).unwrap(),
        ClientCommand::Join(
            vec!["#test".to_string(), "&test2".to_string()],
            Some(vec!["key1".to_string(), "key2".to_string()])
        )
    );
}

#[test]
fn parse_user_test() {
    assert_eq!(
        parse_command("USER Username 0 * :real name\r\n".to_string()).unwrap(),
        ClientCommand::User(User {
            user: "Username".to_string(),
            mode: 0u32,
            real_name: "real name".to_string()
        })
    );
}

#[test]
fn parse_quit_test() {
    assert_eq!(
        parse_command("QUIT :asdf !5^*%".to_string()).unwrap(),
        ClientCommand::Quit(Quit {
            msg: "asdf !5^*%".to_string()
        })
    );
}

#[test]
fn parse_ping_test() {
    assert_eq!(
        parse_command("Ping abeT456-".to_string()).unwrap(),
        ClientCommand::Ping("abeT456-".to_string())
    )
}
