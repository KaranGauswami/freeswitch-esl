#![allow(dead_code, missing_docs)]

use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until},
    character::complete::{digit1, multispace0, newline, space0, space1},
    combinator::{map_res, opt, rest},
    multi::fold_many1,
    sequence::pair,
    IResult,
};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Errors {
    #[error("Command failed")]
    CommandFailed,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandAndApiReplyBody {
    pub headers: HashMap<String, String>,
    pub code: Code,
    pub reply_text: String,
    pub job_uuid: Option<String>,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Code {
    #[default]
    Ok,
    Err,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundEvent {
    pub code: Code,
    pub body: String,
    pub headers: HashMap<String, String>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum FreeswitchReply {
    AuthRequest,
    CommandAndApiReply(CommandAndApiReplyBody),
    DisconnectNotice(String),
    Event(BackgroundEvent),
}

pub fn two_newlines(input: &str) -> IResult<&str, ()> {
    let (input, _) = newline(input)?;
    let (input, _) = newline(input)?;
    Ok((input, ()))
}
pub fn parse_auth_request(input: &str) -> IResult<&str, crate::parser::FreeswitchReply> {
    let (input, _) = tag("Content-Type: auth/request\n\n")(input)?;
    Ok((input, FreeswitchReply::AuthRequest))
}
pub fn parse_ok(input: &str) -> IResult<&str, Code> {
    let (input, _) = tag("+OK")(input)?;
    Ok((input, Code::Ok))
}
pub fn parse_err(input: &str) -> IResult<&str, Code> {
    let (input, _) = tag("-ERR")(input)?;
    Ok((input, Code::Err))
}
pub fn parse_code(input: &str) -> IResult<&str, Code> {
    alt((parse_ok, parse_err))(input)
}
fn parse_body(input: &str) -> IResult<&str, (Code, String)> {
    let (mut input, code) = opt(parse_code)(input)?;
    if code.is_some() {
        (input, _) = space1(input)?;
    }
    let code = code.unwrap_or(Code::Ok);
    let (input, body) = rest(input)?;
    let body = body.trim_end().to_string();
    Ok((input, (code, body.to_string())))
}
pub fn parse_command_reply_with_job_uuid(
    input: &str,
) -> IResult<&str, crate::parser::FreeswitchReply> {
    let (input, _) = tag("Content-Type: command/reply")(input)?;
    let (input, _) = newline(input)?;
    let (input, _) = tag("Reply-Text: ")(input)?;
    let (input, code) = parse_code(input)?;
    let (input, _) = space0(input)?;
    let (input, reply_text) = take_until("\n")(input)?;
    let (input, _) = newline(input)?;
    let (input, _) = tag("Job-UUID: ")(input)?;
    let (input, job_uuid) = take_till(|c| c == '\n')(input)?;
    let (input, _) = two_newlines(input)?;
    let reply = FreeswitchReply::CommandAndApiReply(CommandAndApiReplyBody {
        headers: HashMap::default(),
        code,
        reply_text: reply_text.to_string(),
        job_uuid: Some(job_uuid.to_string()),
    });
    Ok((input, reply))
}
pub fn parse_command_reply(input: &str) -> IResult<&str, crate::parser::FreeswitchReply> {
    let (input, _) = tag("Content-Type: command/reply")(input)?;
    let (input, _) = newline(input)?;
    let (input, _) = tag("Reply-Text: ")(input)?;
    let (input, code) = parse_code(input)?;
    let (input, _) = space0(input)?;
    let (input, reply_text) = take_till(|c| c == '\n')(input)?;
    let (input, _) = two_newlines(input)?;
    let reply = FreeswitchReply::CommandAndApiReply(CommandAndApiReplyBody {
        headers: HashMap::default(),
        code,
        reply_text: reply_text.to_string(),
        job_uuid: None,
    });
    Ok((input, reply))
}
pub fn parse_content_length(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>())(input)
}
pub fn parse_disconnect_event(input: &str) -> IResult<&str, crate::parser::FreeswitchReply> {
    let (input, _) = tag("Content-Type: text/disconnect-notice")(input)?;
    let (input, _) = newline(input)?;
    let (input, _) = tag("Content-Length: ")(input)?;
    let (input, content_length) = parse_content_length(input)?;
    let (input, _) = two_newlines(input)?;
    let (input, content) = take(content_length - 1)(input)?;
    let (input, _) = two_newlines(input)?;
    let reply = FreeswitchReply::DisconnectNotice(content.to_string());
    Ok((input, reply))
}
pub fn parse_api_response(input: &str) -> IResult<&str, crate::parser::FreeswitchReply> {
    let (input, _) = tag("Content-Type: api/response")(input)?;
    let (input, _) = newline(input)?;
    let (input, _) = tag("Content-Length: ")(input)?;
    let (input, content_length) = parse_content_length(input)?;
    let (input, _) = two_newlines(input)?;
    let (input, content) = take(content_length)(input)?;
    let (mut content, code) = opt(parse_code)(content)?;
    if code.is_some() {
        // Space is optional if body is only +OK
        (content, _) = opt(space1)(content)?;
    }
    let (_, response) = rest(content)?;
    let reply = FreeswitchReply::CommandAndApiReply(CommandAndApiReplyBody {
        headers: HashMap::default(),
        code: code.unwrap_or(Code::Ok),
        reply_text: response.trim_end().into(),
        job_uuid: None,
    });
    Ok((input, reply))
}

pub fn parse_key_value(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, key) = take_until(":")(input)?;
    let (input, _) = pair(tag(":"), multispace0)(input)?;
    let (input, value) = take_until("\n")(input)?;
    let (input, _) = tag("\n")(input)?;
    Ok((input, (key, value)))
}
pub fn parse_colon_seperated(input: &str) -> IResult<&str, HashMap<String, String>> {
    fold_many1(parse_key_value, HashMap::new, |mut map, (key, value)| {
        map.insert(key.to_string(), value.to_string());
        map
    })(input)
}

pub fn parse_plain_event(input: &str) -> IResult<&str, crate::parser::FreeswitchReply> {
    let (input, _) = tag("Content-Length: ")(input)?;
    let (input, content_length) = parse_content_length(input)?;
    let (input, _) = newline(input)?;
    let (input, _) = tag("Content-Type: text/event-plain")(input)?;
    let (input, _) = newline(input)?;
    let (mut input, content) = take(content_length)(input)?;
    let (content, _) = newline(content)?;
    let (remaining, maps) = parse_colon_seperated(content)?;
    let body = if let Some(length) = maps.get("Content-Length") {
        let (_, optional_body) = parse_optional_body(length.parse().unwrap(), remaining)?;
        (input, _) = tag("\n")(input)?;
        optional_body
    } else {
        ""
    };

    let (_, (code, data)) = parse_body(body).unwrap();
    let reply = FreeswitchReply::Event(BackgroundEvent {
        code,
        body: data,
        headers: maps.clone(),
    });
    let (input, _) = opt(newline)(input)?;
    Ok((input, reply))
}

fn parse_optional_body(content_length: usize, input: &str) -> IResult<&str, &str> {
    let (input, body) = take(content_length)(input)?;
    Ok((input, body.trim()))
}
fn parse_colon_seperated_reply(input: &str) -> IResult<&str, FreeswitchReply> {
    let (input_modified, data) = parse_colon_seperated(input)?;
    let (input_modified, _) = newline(input_modified)?;
    let command_reply = "command/reply".to_string();
    if let Some(content_type) = data.get("Content-Type") {
        if content_type == &command_reply {
            return Ok((
                input_modified,
                FreeswitchReply::Event(BackgroundEvent {
                    headers: data,
                    ..Default::default()
                }),
            ));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Eof,
    )))
}

pub fn parse_any_freeswitch_event(input: &str) -> IResult<&str, crate::parser::FreeswitchReply> {
    alt((
        parse_command_reply_with_job_uuid,
        parse_command_reply,
        parse_api_response,
        parse_plain_event,
        parse_disconnect_event,
        parse_auth_request,
        parse_colon_seperated_reply,
    ))(input)
}
#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn auth_request() {
        let input = "Content-Type: auth/request\n\n";
        assert_eq!(
            parse_auth_request(input),
            Ok(("", FreeswitchReply::AuthRequest))
        )
    }

    #[test]
    fn check_ok_code() {
        let input = "+OK";
        assert_eq!(parse_ok(input), Ok(("", Code::Ok)))
    }
    #[test]
    fn check_err_code() {
        let input = "-ERR";
        assert_eq!(parse_err(input), Ok(("", Code::Err)))
    }
    #[test]
    fn check_code_parser() {
        let input = "-ERR";
        assert_eq!(parse_code(input), Ok(("", Code::Err)));
        let input = "+OK";
        assert_eq!(parse_code(input), Ok(("", Code::Ok)));
    }

    #[test]
    fn parse_command_reply_1() {
        let input = "Content-Type: command/reply\nReply-Text: +OK event listener enabled json\n\n";
        assert_eq!(
            parse_command_reply(input),
            Ok((
                "",
                FreeswitchReply::CommandAndApiReply(CommandAndApiReplyBody {
                    headers: HashMap::default(),
                    code: Code::Ok,
                    reply_text: "event listener enabled json".to_string(),
                    job_uuid: None,
                })
            ))
        )
    }
    #[test]
    fn parse_command_reply_2() {
        let input = "Content-Type: command/reply\nReply-Text: +OK Job-UUID: 0435d687-db9c-46b6-9221-79f82852c1a0\nJob-UUID: 0435d687-db9c-46b6-9221-79f82852c1a0\n\n";
        assert_eq!(
            parse_command_reply_with_job_uuid(input),
            Ok((
                "",
                FreeswitchReply::CommandAndApiReply(CommandAndApiReplyBody {
                    headers: HashMap::default(),
                    code: Code::Ok,
                    reply_text: "Job-UUID: 0435d687-db9c-46b6-9221-79f82852c1a0".to_string(),
                    job_uuid: Some("0435d687-db9c-46b6-9221-79f82852c1a0".to_string()),
                })
            ))
        )
    }
    #[test]
    fn test_parsing_disconnect_notice() {
        let input = "Content-Type: text/disconnect-notice\nContent-Length: 67\n\nDisconnected, goodbye.\nSee you at ClueCon! http://www.cluecon.com/\n\n";
        assert_eq!(
            parse_disconnect_event(input),
            Ok((
                "",
                FreeswitchReply::DisconnectNotice(
                    "Disconnected, goodbye.\nSee you at ClueCon! http://www.cluecon.com/"
                        .to_string()
                )
            ))
        )
    }
    #[test]
    fn test_parsing_api_response_1() {
        let input = "Content-Type: api/response\nContent-Length: 23\n\n-ERR SUBSCRIBER_ABSENT\n";
        assert_eq!(
            parse_api_response(input),
            Ok((
                "",
                FreeswitchReply::CommandAndApiReply({
                    CommandAndApiReplyBody {
                        headers: HashMap::default(),
                        code: Code::Err,
                        reply_text: "SUBSCRIBER_ABSENT".into(),
                        job_uuid: None,
                    }
                })
            ))
        )
    }
    #[test]
    fn test_parsing_api_response_2() {
        // let input = "Content-Type: api/response\nContent-Length: 23\n\n-ERR SUBSCRIBER_ABSENT\n\n";
        let input = "Content-Type: api/response\nContent-Length: 14\n\n+OK [Success]\n";
        assert_eq!(
            parse_api_response(input),
            Ok((
                "",
                FreeswitchReply::CommandAndApiReply({
                    CommandAndApiReplyBody {
                        headers: HashMap::default(),
                        code: Code::Ok,
                        reply_text: "[Success]".into(),
                        job_uuid: None,
                    }
                })
            ))
        )
    }
    #[test]
    fn test_parsing_api_response_3() {
        // let input = "Content-Type: api/response\nContent-Length: 23\n\n-ERR SUBSCRIBER_ABSENT\n\n";
        let input = "Content-Type: api/response\nContent-Length: 41\n\nReload XML [Success]\nrestarting: external";
        assert_eq!(
            parse_api_response(input),
            Ok((
                "",
                FreeswitchReply::CommandAndApiReply({
                    CommandAndApiReplyBody {
                        headers: HashMap::default(),
                        code: Code::Ok,
                        reply_text: "Reload XML [Success]\nrestarting: external".into(),
                        job_uuid: None,
                    }
                })
            ))
        )
    }
    #[test]
    fn test_parsing_api_response_4() {
        let input = "Content-Type: api/response\nContent-Length: 4\n\n+OK\n";
        assert_eq!(
            parse_api_response(input),
            Ok((
                "",
                FreeswitchReply::CommandAndApiReply({
                    CommandAndApiReplyBody {
                        code: Code::Ok,
                        reply_text: "".into(),
                        job_uuid: None,
                        ..Default::default()
                    }
                })
            ))
        )
    }

    #[test]
    fn test_parsing_body() {
        let input = "+OK [Success]\n";
        assert_eq!(parse_body(input), Ok(("", (Code::Ok, "[Success]".into()))));
    }
    #[test]
    fn test_parsing_multiple_request() {
        let input = "Content-Type: api/response\nContent-Length: 14\n\n+OK [Success]\nContent-Type: api/response\nContent-Length: 23\n\n-ERR SUBSCRIBER_ABSENT\n";
        let (input, event) = parse_any_freeswitch_event(input).unwrap();
        assert_eq!(
            input,
            "Content-Type: api/response\nContent-Length: 23\n\n-ERR SUBSCRIBER_ABSENT\n"
        );
        assert_eq!(
            event,
            (FreeswitchReply::CommandAndApiReply(CommandAndApiReplyBody {
                code: Code::Ok,
                reply_text: "[Success]".into(),
                job_uuid: None,
                ..Default::default()
            }))
        );
        let (input, event) = parse_any_freeswitch_event(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(
            event,
            (FreeswitchReply::CommandAndApiReply(CommandAndApiReplyBody {
                code: Code::Err,
                reply_text: "SUBSCRIBER_ABSENT".into(),
                job_uuid: None,
                ..Default::default()
            }))
        );
    }

    #[test]
    fn test_parsing_single_key_pair() {
        let input = "Event-Name: CHANNEL_EXECUTE_COMPLETE\n";
        let (input, (key, value)) = parse_key_value(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(key, "Event-Name");
        assert_eq!(value, "CHANNEL_EXECUTE_COMPLETE");
    }
    #[test]
    fn test_parsing_multiple_key_pair() {
        let input = "Event-Name: CHANNEL_EXECUTE_COMPLETE\nCore-UUID: bd0e8916-6a60-4e11-8978-db8580b440a6\nFreeSWITCH-Hostname: ip-172-31-32-63\n";
        let (input, result) = parse_colon_seperated(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(
            result.get("Event-Name"),
            Some(&"CHANNEL_EXECUTE_COMPLETE".to_owned())
        );
        assert_eq!(
            result.get("Core-UUID"),
            Some(&"bd0e8916-6a60-4e11-8978-db8580b440a6".to_owned())
        );
        assert_eq!(
            result.get("FreeSWITCH-Hostname"),
            Some(&"ip-172-31-32-63".to_owned())
        );
    }
    #[test]
    fn test_parsing_with_plain_event_1() {
        let input = "Content-Length: 2763\nContent-Type: text/event-plain\n\nEvent-Name: CHANNEL_EXECUTE_COMPLETE\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2008%3A58%3A59\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2008%3A58%3A59%20GMT\nEvent-Date-Timestamp: 1695545939666460\nEvent-Calling-File: switch_core_session.c\nEvent-Calling-Function: switch_core_session_exec\nEvent-Calling-Line-Number: 2967\nEvent-Sequence: 5442\nChannel-State: CS_EXECUTE\nChannel-Call-State: RINGING\nChannel-State-Number: 4\nChannel-Name: loopback/1000-b\nUnique-ID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nCall-Direction: inbound\nPresence-Call-Direction: inbound\nChannel-HIT-Dialplan: true\nChannel-Call-UUID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nAnswer-State: ringing\nChannel-Read-Codec-Name: L16\nChannel-Read-Codec-Rate: 8000\nChannel-Read-Codec-Bit-Rate: 128000\nChannel-Write-Codec-Name: L16\nChannel-Write-Codec-Rate: 8000\nChannel-Write-Codec-Bit-Rate: 128000\nCaller-Direction: inbound\nCaller-Logical-Direction: inbound\nCaller-Dialplan: xml\nCaller-Caller-ID-Number: 0000000000\nCaller-Orig-Caller-ID-Number: 0000000000\nCaller-Callee-ID-Name: Outbound%20Call\nCaller-Callee-ID-Number: 1000\nCaller-ANI: 0000000000\nCaller-Destination-Number: 1000\nCaller-Unique-ID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nCaller-Source: mod_loopback\nCaller-Context: default\nCaller-Channel-Name: loopback/1000-b\nCaller-Profile-Index: 1\nCaller-Profile-Created-Time: 1695545939666460\nCaller-Channel-Created-Time: 1695545939666460\nCaller-Channel-Answered-Time: 0\nCaller-Channel-Progress-Time: 0\nCaller-Channel-Progress-Media-Time: 0\nCaller-Channel-Hangup-Time: 0\nCaller-Channel-Transfer-Time: 0\nCaller-Channel-Resurrect-Time: 0\nCaller-Channel-Bridged-Time: 0\nCaller-Channel-Last-Hold: 0\nCaller-Channel-Hold-Accum: 0\nCaller-Screen-Bit: true\nCaller-Privacy-Hide-Name: false\nCaller-Privacy-Hide-Number: false\nvariable_direction: inbound\nvariable_uuid: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nvariable_session_id: 58\nvariable_channel_name: loopback/1000-b\nvariable_read_codec: L16\nvariable_read_rate: 8000\nvariable_write_codec: L16\nvariable_write_rate: 8000\nvariable_origination_uuid: karan\nvariable_other_loopback_leg_uuid: karan\nvariable_loopback_leg: B\nvariable_DP_MATCH: ARRAY%3A%3A1000%7C%3A1000\nvariable_call_uuid: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nvariable_dialed_extension: 1000\nvariable_export_vars: dialed_extension\nvariable_current_application_data: 1%20b%20s%20execute_extension%3A%3Adx%20XML%20features\nvariable_current_application: bind_meta_app\nApplication: bind_meta_app\nApplication-Data: 1%20b%20s%20execute_extension%3A%3Adx%20XML%20features\nApplication-Response: _none_\nApplication-UUID: 55ec8e17-a7a3-44de-812b-ca08e6ec07a7\n";
        let (input, data) = parse_plain_event(input).unwrap();
        assert_eq!(input, "");
        match data {
            FreeswitchReply::Event(n) => {
                let event_name = n.headers.get("Event-Name");
                assert_eq!(event_name, Some(&"CHANNEL_EXECUTE_COMPLETE".to_string()));
                let body_data = n.body;
                assert_eq!(body_data, "");
            }
            _ => panic!("Should not happen"),
        }
    }
    #[test]
    fn test_parsing_with_plain_event_background_job() {
        let input = "Content-Length: 575\nContent-Type: text/event-plain\n\nEvent-Name: BACKGROUND_JOB\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2005%3A48%3A28\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2005%3A48%3A28%20GMT\nEvent-Date-Timestamp: 1695534508726403\nEvent-Calling-File: mod_event_socket.c\nEvent-Calling-Function: api_exec\nEvent-Calling-Line-Number: 1572\nEvent-Sequence: 1041\nJob-UUID: dcab6b81-ec71-4552-b897-88721870fe16\nJob-Command: reloadxml\nContent-Length: 14\n\n+OK [Success]\n";
        let (input, data) = parse_plain_event(input).unwrap();
        assert_eq!(input, "");
        match data {
            FreeswitchReply::Event(n) => {
                let event_name = n.headers.get("Event-Name");
                assert_eq!(event_name, Some(&"BACKGROUND_JOB".to_string()));
                let body_data = n.body;
                assert_eq!(body_data, "[Success]");
            }
            _ => panic!("Should not happen"),
        }
    }
    #[test]
    fn test_parsing_random_event() {
        let input = "Content-Length: 2687\nContent-Type: text/event-plain\n\nEvent-Name: CHANNEL_EXECUTE_COMPLETE\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2008%3A58%3A59\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2008%3A58%3A59%20GMT\nEvent-Date-Timestamp: 1695545939666460\nEvent-Calling-File: switch_core_session.c\nEvent-Calling-Function: switch_core_session_exec\nEvent-Calling-Line-Number: 2967\nEvent-Sequence: 5440\nChannel-State: CS_EXECUTE\nChannel-Call-State: RINGING\nChannel-State-Number: 4\nChannel-Name: loopback/1000-b\nUnique-ID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nCall-Direction: inbound\nPresence-Call-Direction: inbound\nChannel-HIT-Dialplan: true\nChannel-Call-UUID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nAnswer-State: ringing\nChannel-Read-Codec-Name: L16\nChannel-Read-Codec-Rate: 8000\nChannel-Read-Codec-Bit-Rate: 128000\nChannel-Write-Codec-Name: L16\nChannel-Write-Codec-Rate: 8000\nChannel-Write-Codec-Bit-Rate: 128000\nCaller-Direction: inbound\nCaller-Logical-Direction: inbound\nCaller-Dialplan: xml\nCaller-Caller-ID-Number: 0000000000\nCaller-Orig-Caller-ID-Number: 0000000000\nCaller-Callee-ID-Name: Outbound%20Call\nCaller-Callee-ID-Number: 1000\nCaller-ANI: 0000000000\nCaller-Destination-Number: 1000\nCaller-Unique-ID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nCaller-Source: mod_loopback\nCaller-Context: default\nCaller-Channel-Name: loopback/1000-b\nCaller-Profile-Index: 1\nCaller-Profile-Created-Time: 1695545939666460\nCaller-Channel-Created-Time: 1695545939666460\nCaller-Channel-Answered-Time: 0\nCaller-Channel-Progress-Time: 0\nCaller-Channel-Progress-Media-Time: 0\nCaller-Channel-Hangup-Time: 0\nCaller-Channel-Transfer-Time: 0\nCaller-Channel-Resurrect-Time: 0\nCaller-Channel-Bridged-Time: 0\nCaller-Channel-Last-Hold: 0\nCaller-Channel-Hold-Accum: 0\nCaller-Screen-Bit: true\nCaller-Privacy-Hide-Name: false\nCaller-Privacy-Hide-Number: false\nvariable_direction: inbound\nvariable_uuid: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nvariable_session_id: 58\nvariable_channel_name: loopback/1000-b\nvariable_read_codec: L16\nvariable_read_rate: 8000\nvariable_write_codec: L16\nvariable_write_rate: 8000\nvariable_origination_uuid: karan\nvariable_other_loopback_leg_uuid: karan\nvariable_loopback_leg: B\nvariable_DP_MATCH: ARRAY%3A%3A1000%7C%3A1000\nvariable_call_uuid: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nvariable_current_application_data: dialed_extension%3D1000\nvariable_current_application: export\nvariable_dialed_extension: 1000\nvariable_export_vars: dialed_extension\nApplication: export\nApplication-Data: dialed_extension%3D1000\nApplication-Response: _none_\nApplication-UUID: 9c016dd8-b8e6-4e64-9034-7aa0ffbf69e7\n\nContent-Length: 2763\nContent-Type: text/event-plain\n\nEvent-Name: CHANNEL_EXECUTE_COMPLETE\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2008%3A58%3A59\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2008%3A58%3A59%20GMT\nEvent-Date-Timestamp: 1695545939666460\nEvent-Calling-File: switch_core_session.c\nEvent-Calling-Function: switch_core_session_exec\nEvent-Calling-Line-Number: 2967\nEvent-Sequence: 5442\nChannel-State: CS_EXECUTE\nChannel-Call-State: RINGING\nChannel-State-Number: 4\nChannel-Name: loopback/1000-b\nUnique-ID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nCall-Direction: inbound\nPresence-Call-Direction: inbound\nChannel-HIT-Dialplan: true\nChannel-Call-UUID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nAnswer-State: ringing\nChannel-Read-Codec-Name: L16\nChannel-Read-Codec-Rate: 8000\nChannel-Read-Codec-Bit-Rate: 128000\nChannel-Write-Codec-Name: L16\nChannel-Write-Codec-Rate: 8000\nChannel-Write-Codec-Bit-Rate: 128000\nCaller-Direction: inbound\nCaller-Logical-Direction: inbound\nCaller-Dialplan: xml\nCaller-Caller-ID-Number: 0000000000\nCaller-Orig-Caller-ID-Number: 0000000000\nCaller-Callee-ID-Name: Outbound%20Call\nCaller-Callee-ID-Number: 1000\nCaller-ANI: 0000000000\nCaller-Destination-Number: 1000\nCaller-Unique-ID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nCaller-Source: mod_loopback\nCaller-Context: default\nCaller-Channel-Name: loopback/1000-b\nCaller-Profile-Index: 1\nCaller-Profile-Created-Time: 1695545939666460\nCaller-Channel-Created-Time: 1695545939666460\nCaller-Channel-Answered-Time: 0\nCaller-Channel-Progress-Time: 0\nCaller-Channel-Progress-Media-Time: 0\nCaller-Channel-Hangup-Time: 0\nCaller-Channel-Transfer-Time: 0\nCaller-Channel-Resurrect-Time: 0\nCaller-Channel-Bridged-Time: 0\nCaller-Channel-Last-Hold: 0\nCaller-Channel-Hold-Accum: 0\nCaller-Screen-Bit: true\nCaller-Privacy-Hide-Name: false\nCaller-Privacy-Hide-Number: false\nvariable_direction: inbound\nvariable_uuid: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nvariable_session_id: 58\nvariable_channel_name: loopback/1000-b\nvariable_read_codec: L16\nvariable_read_rate: 8000\nvariable_write_codec: L16\nvariable_write_rate: 8000\nvariable_origination_uuid: karan\nvariable_other_loopback_leg_uuid: karan\nvariable_loopback_leg: B\nvariable_DP_MATCH: ARRAY%3A%3A1000%7C%3A1000\nvariable_call_uuid: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nvariable_dialed_extension: 1000\nvariable_export_vars: dialed_extension\nvariable_current_application_data: 1%20b%20s%20execute_extension%3A%3Adx%20XML%20features\nvariable_current_application: bind_meta_app\nApplication: bind_meta_app\nApplication-Data: 1%20b%20s%20execute_extension%3A%3Adx%20XML%20features\nApplication-Response: _none_\nApplication-UUID: 55ec8e17-a7a3-44de-812b-ca08e6ec07a7\n";
        let remaining_input = "Content-Length: 2763\nContent-Type: text/event-plain\n\nEvent-Name: CHANNEL_EXECUTE_COMPLETE\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2008%3A58%3A59\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2008%3A58%3A59%20GMT\nEvent-Date-Timestamp: 1695545939666460\nEvent-Calling-File: switch_core_session.c\nEvent-Calling-Function: switch_core_session_exec\nEvent-Calling-Line-Number: 2967\nEvent-Sequence: 5442\nChannel-State: CS_EXECUTE\nChannel-Call-State: RINGING\nChannel-State-Number: 4\nChannel-Name: loopback/1000-b\nUnique-ID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nCall-Direction: inbound\nPresence-Call-Direction: inbound\nChannel-HIT-Dialplan: true\nChannel-Call-UUID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nAnswer-State: ringing\nChannel-Read-Codec-Name: L16\nChannel-Read-Codec-Rate: 8000\nChannel-Read-Codec-Bit-Rate: 128000\nChannel-Write-Codec-Name: L16\nChannel-Write-Codec-Rate: 8000\nChannel-Write-Codec-Bit-Rate: 128000\nCaller-Direction: inbound\nCaller-Logical-Direction: inbound\nCaller-Dialplan: xml\nCaller-Caller-ID-Number: 0000000000\nCaller-Orig-Caller-ID-Number: 0000000000\nCaller-Callee-ID-Name: Outbound%20Call\nCaller-Callee-ID-Number: 1000\nCaller-ANI: 0000000000\nCaller-Destination-Number: 1000\nCaller-Unique-ID: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nCaller-Source: mod_loopback\nCaller-Context: default\nCaller-Channel-Name: loopback/1000-b\nCaller-Profile-Index: 1\nCaller-Profile-Created-Time: 1695545939666460\nCaller-Channel-Created-Time: 1695545939666460\nCaller-Channel-Answered-Time: 0\nCaller-Channel-Progress-Time: 0\nCaller-Channel-Progress-Media-Time: 0\nCaller-Channel-Hangup-Time: 0\nCaller-Channel-Transfer-Time: 0\nCaller-Channel-Resurrect-Time: 0\nCaller-Channel-Bridged-Time: 0\nCaller-Channel-Last-Hold: 0\nCaller-Channel-Hold-Accum: 0\nCaller-Screen-Bit: true\nCaller-Privacy-Hide-Name: false\nCaller-Privacy-Hide-Number: false\nvariable_direction: inbound\nvariable_uuid: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nvariable_session_id: 58\nvariable_channel_name: loopback/1000-b\nvariable_read_codec: L16\nvariable_read_rate: 8000\nvariable_write_codec: L16\nvariable_write_rate: 8000\nvariable_origination_uuid: karan\nvariable_other_loopback_leg_uuid: karan\nvariable_loopback_leg: B\nvariable_DP_MATCH: ARRAY%3A%3A1000%7C%3A1000\nvariable_call_uuid: 81bdc2ed-2be3-42a8-93dd-ab596f352c83\nvariable_dialed_extension: 1000\nvariable_export_vars: dialed_extension\nvariable_current_application_data: 1%20b%20s%20execute_extension%3A%3Adx%20XML%20features\nvariable_current_application: bind_meta_app\nApplication: bind_meta_app\nApplication-Data: 1%20b%20s%20execute_extension%3A%3Adx%20XML%20features\nApplication-Response: _none_\nApplication-UUID: 55ec8e17-a7a3-44de-812b-ca08e6ec07a7\n";
        let (input, _) = parse_any_freeswitch_event(input).unwrap();
        assert_eq!(input, remaining_input);
    }

    #[test]
    fn test_parsing_outbound_connect_event() {
        let input ="Event-Name: CHANNEL_DATA\nCore-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nFreeSWITCH-Hostname: ip-172-31-5-95\nFreeSWITCH-Switchname: ip-172-31-5-95\nFreeSWITCH-IPv4: 172.31.5.95\nFreeSWITCH-IPv6: %3A%3A1\nEvent-Date-Local: 2023-09-24%2010%3A04%3A06\nEvent-Date-GMT: Sun,%2024%20Sep%202023%2010%3A04%3A06%20GMT\nEvent-Date-Timestamp: 1695549846366420\nEvent-Calling-File: mod_event_socket.c\nEvent-Calling-Function: parse_command\nEvent-Calling-Line-Number: 2021\nEvent-Sequence: 7934\nChannel-Direction: inbound\nChannel-Logical-Direction: inbound\nChannel-Username: 1000\nChannel-Dialplan: XML\nChannel-Caller-ID-Name: 1000\nChannel-Caller-ID-Number: 1000\nChannel-Orig-Caller-ID-Name: 1000\nChannel-Orig-Caller-ID-Number: 1000\nChannel-Network-Addr: 122.172.98.23\nChannel-ANI: 1000\nChannel-Destination-Number: 9999\nChannel-Unique-ID: c8130d8b-2109-47eb-bdbc-a8f7159f2821\nChannel-Source: mod_sofia\nChannel-Context: default\nChannel-Channel-Name: sofia/internal/1000%403.110.42.145\nChannel-Profile-Index: 1\nChannel-Profile-Created-Time: 1695549658226424\nChannel-Channel-Created-Time: 1695549658226424\nChannel-Channel-Answered-Time: 0\nChannel-Channel-Progress-Time: 0\nChannel-Channel-Progress-Media-Time: 0\nChannel-Channel-Hangup-Time: 0\nChannel-Channel-Transfer-Time: 0\nChannel-Channel-Resurrect-Time: 0\nChannel-Channel-Bridged-Time: 0\nChannel-Channel-Last-Hold: 0\nChannel-Channel-Hold-Accum: 0\nChannel-Screen-Bit: true\nChannel-Privacy-Hide-Name: false\nChannel-Privacy-Hide-Number: false\nChannel-State: CS_EXECUTE\nChannel-Call-State: RINGING\nChannel-State-Number: 4\nChannel-Name: sofia/internal/1000%403.110.42.145\nUnique-ID: c8130d8b-2109-47eb-bdbc-a8f7159f2821\nCall-Direction: inbound\nPresence-Call-Direction: inbound\nChannel-HIT-Dialplan: true\nChannel-Presence-ID: 1000%403.110.42.145\nChannel-Call-UUID: c8130d8b-2109-47eb-bdbc-a8f7159f2821\nAnswer-State: ringing\nCaller-Direction: inbound\nCaller-Logical-Direction: inbound\nCaller-Username: 1000\nCaller-Dialplan: XML\nCaller-Caller-ID-Name: 1000\nCaller-Caller-ID-Number: 1000\nCaller-Orig-Caller-ID-Name: 1000\nCaller-Orig-Caller-ID-Number: 1000\nCaller-Network-Addr: 122.172.98.23\nCaller-ANI: 1000\nCaller-Destination-Number: 9999\nCaller-Unique-ID: c8130d8b-2109-47eb-bdbc-a8f7159f2821\nCaller-Source: mod_sofia\nCaller-Context: default\nCaller-Channel-Name: sofia/internal/1000%403.110.42.145\nCaller-Profile-Index: 1\nCaller-Profile-Created-Time: 1695549658226424\nCaller-Channel-Created-Time: 1695549658226424\nCaller-Channel-Answered-Time: 0\nCaller-Channel-Progress-Time: 0\nCaller-Channel-Progress-Media-Time: 0\nCaller-Channel-Hangup-Time: 0\nCaller-Channel-Transfer-Time: 0\nCaller-Channel-Resurrect-Time: 0\nCaller-Channel-Bridged-Time: 0\nCaller-Channel-Last-Hold: 0\nCaller-Channel-Hold-Accum: 0\nCaller-Screen-Bit: true\nCaller-Privacy-Hide-Name: false\nCaller-Privacy-Hide-Number: false\nvariable_direction: inbound\nvariable_uuid: c8130d8b-2109-47eb-bdbc-a8f7159f2821\nvariable_session_id: 97\nvariable_sip_from_user: 1000\nvariable_sip_from_uri: 1000%403.110.42.145\nvariable_sip_from_host: 3.110.42.145\nvariable_video_media_flow: disabled\nvariable_audio_media_flow: disabled\nvariable_text_media_flow: disabled\nvariable_channel_name: sofia/internal/1000%403.110.42.145\nvariable_sip_call_id: d38a73d0e3ac414d9f0893915597aa6b\nvariable_sip_local_network_addr: 3.110.42.145\nvariable_sip_network_ip: 122.172.98.23\nvariable_sip_network_port: 50456\nvariable_sip_invite_stamp: 1695549658226424\nvariable_sip_received_ip: 122.172.98.23\nvariable_sip_received_port: 50456\nvariable_sip_via_protocol: udp\nvariable_sip_authorized: true\nvariable_Event-Name: REQUEST_PARAMS\nvariable_Core-UUID: 0cb916f9-98ad-4fce-bcd5-5fe03c745316\nvariable_FreeSWITCH-Hostname: ip-172-31-5-95\nvariable_FreeSWITCH-Switchname: ip-172-31-5-95\nvariable_FreeSWITCH-IPv4: 172.31.5.95\nvariable_FreeSWITCH-IPv6: %3A%3A1\nvariable_Event-Date-Local: 2023-09-24%2010%3A00%3A58\nvariable_Event-Date-GMT: Sun,%2024%20Sep%202023%2010%3A00%3A58%20GMT\nvariable_Event-Date-Timestamp: 1695549658226424\nvariable_Event-Calling-File: sofia.c\nvariable_Event-Calling-Function: sofia_handle_sip_i_invite\nvariable_Event-Calling-Line-Number: 10722\nvariable_Event-Sequence: 7881\nvariable_sip_number_alias: 1000\nvariable_sip_auth_username: 1000\nvariable_sip_auth_realm: 3.110.42.145\nvariable_number_alias: 1000\nvariable_requested_user_name: 1000\nvariable_requested_domain_name: 172.31.5.95\nvariable_record_stereo: true\nvariable_default_gateway: example.com\nvariable_default_areacode: 918\nvariable_transfer_fallback_extension: operator\nvariable_toll_allow: domestic,international,local\nvariable_accountcode: 1000\nvariable_user_context: default\nvariable_effective_caller_id_name: Extension%201000\nvariable_effective_caller_id_number: 1000\nvariable_outbound_caller_id_name: FreeSWITCH\nvariable_outbound_caller_id_number: 0000000000\nvariable_callgroup: techsupport\nvariable_user_name: 1000\nvariable_domain_name: 172.31.5.95\nvariable_sip_from_user_stripped: 1000\nvariable_sip_from_tag: d87f0933f993479f824c7387e619bbf4\nvariable_sofia_profile_name: internal\nvariable_sofia_profile_url: sip%3Amod_sofia%403.110.42.145%3A5060\nvariable_recovery_profile_name: internal\nvariable_sip_full_via: SIP/2.0/UDP%20122.172.98.23%3A50456%3Brport%3D50456%3Bbranch%3Dz9hG4bKPj97c28beead2c4175971cb07228db69f9\nvariable_sip_full_from: %3Csip%3A1000%403.110.42.145%3E%3Btag%3Dd87f0933f993479f824c7387e619bbf4\nvariable_sip_full_to: %3Csip%3A9999%403.110.42.145%3E\nvariable_sip_allow: PRACK,%20INVITE,%20ACK,%20BYE,%20CANCEL,%20UPDATE,%20INFO,%20SUBSCRIBE,%20NOTIFY,%20REFER,%20MESSAGE,%20OPTIONS\nvariable_sip_req_user: 9999\nvariable_sip_req_uri: 9999%403.110.42.145\nvariable_sip_req_host: 3.110.42.145\nvariable_sip_to_user: 9999\nvariable_sip_to_uri: 9999%403.110.42.145\nvariable_sip_to_host: 3.110.42.145\nvariable_sip_contact_params: ob\nvariable_sip_contact_user: 1000\nvariable_sip_contact_port: 50456\nvariable_sip_contact_uri: 1000%40122.172.98.23%3A50456\nvariable_sip_contact_host: 122.172.98.23\nvariable_rtp_use_codec_string: OPUS,G722,PCMU,PCMA,H264,VP8\nvariable_sip_user_agent: MicroSIP/3.21.3\nvariable_sip_via_host: 122.172.98.23\nvariable_sip_via_port: 50456\nvariable_sip_via_rport: 50456\nvariable_max_forwards: 70\nvariable_presence_id: 1000%403.110.42.145\nvariable_switch_r_sdp: v%3D0%0D%0Ao%3D-%203904558258%203904558258%20IN%20IP4%20122.172.98.23%0D%0As%3Dpjmedia%0D%0Ab%3DAS%3A84%0D%0At%3D0%200%0D%0Aa%3DX-nat%3A0%0D%0Am%3Daudio%204022%20RTP/AVP%208%200%20101%0D%0Ac%3DIN%20IP4%20122.172.98.23%0D%0Ab%3DTIAS%3A64000%0D%0Aa%3Drtpmap%3A8%20PCMA/8000%0D%0Aa%3Drtpmap%3A0%20PCMU/8000%0D%0Aa%3Drtpmap%3A101%20telephone-event/8000%0D%0Aa%3Dfmtp%3A101%200-16%0D%0Aa%3Drtcp%3A4023%20IN%20IP4%20192.168.1.18%0D%0Aa%3Dssrc%3A1025981160%20cname%3A21a156dd10a05b64%0D%0A\nvariable_ep_codec_string: CORE_PCM_MODULE.PCMA%408000h%4020i%4064000b,CORE_PCM_MODULE.PCMU%408000h%4020i%4064000b\nvariable_endpoint_disposition: DELAYED%20NEGOTIATION\nvariable_DP_MATCH: ARRAY%3A%3ADELAYED%20NEGOTIATION%7C%3ADELAYED%20NEGOTIATION\nvariable_call_uuid: c8130d8b-2109-47eb-bdbc-a8f7159f2821\nvariable_RFC2822_DATE: Sun,%2024%20Sep%202023%2010%3A00%3A58%20%2B0000\nvariable_export_vars: RFC2822_DATE\nvariable_current_application_data: 127.0.0.1%3A8085%20async%20full\nvariable_current_application: socket\nvariable_socket_host: 127.0.0.1\nContent-Type: command/reply\nReply-Text: %2BOK%0A\nSocket-Mode: async\nControl: full\n\n";
        let (input, _) = parse_any_freeswitch_event(input).unwrap();
        assert_eq!("", input);
    }
}
