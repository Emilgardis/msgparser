#[derive(Debug, PartialEq, Eq)]
pub enum MessagePart<'a, 'e> {
    Text(&'a str),
    // emote id
    Emote(&'e str),
    Codeblock(&'a str),
}

pub fn parse_parts<'msg, 'e>(
    msg: &'msg str,
    mut emotes: &'e [(&'e str, std::ops::Range<usize>)],
) -> Vec<MessagePart<'msg, 'e>>
where
    'msg: 'e,
{
    assert!(
        emotes
            .iter()
            .fold((true, 0), |(cond, pos), (_, r)| {
                (cond && r.start >= pos, r.end)
            })
            .0,
        "emotes needs to be sorted and non-overlapping: {emotes:?}"
    );

    let mut parts = vec![];
    let mut cursor = 0;
    let mut cur_emote: Option<&(&str, std::ops::Range<usize>)> = None;
    while !msg[cursor..].is_empty() {
        if let Some(emote) = cur_emote {
            if emote.1.start < cursor {
                cur_emote = emotes.get(0);
                emotes = emotes.get(1..).unwrap_or(&[]);
                continue;
            }
        } else {
            cur_emote = emotes.get(0);
            emotes = emotes.get(1..).unwrap_or(&[]);
        }
        match (msg[cursor..].find(['`']), cur_emote) {
            // code block is next
            (Some(c_start), e) if e.is_none() || c_start + cursor < e.unwrap().1.start => {
                let text_end = c_start + cursor;
                let text = &msg[cursor..text_end];
                // find end of codeblock
                let code = if msg[text_end..].starts_with("```") {
                    if let Some(c_end) = msg[text_end + 3..].find("```") {
                        &msg[text_end..=text_end + c_end + 3 + 2]
                    } else {
                        &msg[text_end..]
                    }
                } else if let Some(c_end) = msg[text_end + 1..].find('`') {
                    &msg[text_end..=text_end + c_end + 1]
                } else {
                    &msg[text_end..]
                };
                cursor += text.len() + code.len();

                if !text.is_empty() {
                    parts.push(MessagePart::Text(text));
                }
                parts.push(MessagePart::Codeblock(code));
            }

            // emote is next
            (_, Some(emote)) => {
                let text = &msg[cursor..emote.1.start];
                cursor += emote.1.len() + text.len();
                if !text.is_empty() {
                    parts.push(MessagePart::Text(text));
                }
                parts.push(MessagePart::Emote(emote.0));
                cur_emote = None;
            }

            // text
            (None, None) => {
                parts.push(MessagePart::Text(&msg[cursor..]));
                cursor = msg.len();
            }

            (Some(_), None) => unreachable!(),
        }
    }

    parts
}

#[test]
fn test() {
    let msg = r#"hello `world " Kappa " lol ` Kappa `"#;
    let emotes = vec![("25", 15..19), ("25", 29..33)];
    println!("{:#?}", parse_parts(msg, &emotes));
}

#[test]
fn test_eh() {
    let msg = r#"Q``Q``c```d```"#;
    println!("{:#?}", parse_parts(msg, &[]));
}

#[test]
fn test_eh2() {
    let msg = r#"``QQQ`` c `"#;
    println!("{:#?}", parse_parts(msg, &[("cQ", 8..9)]));
}

// [23:02:30/966] --> [main] @badge-info=
// ;badges=broadcaster/1
// ;client-nonce=cd2566cff86d400f0003563ccf5002b9
// ;color=#FF69B4
// ;display-name=emilgardis
// ;emotes=25:15-19,29-33
// ;first-msg=0
// ;flags=
// ;id=f34b54a3-12f2-4e89-9fb1-3aa24bb7d394
// ;mod=0
// ;returning-chatter=0
// ;room-id=27620241
// ;subscriber=0
// ;tmi-sent-ts=1670450550757
// ;turbo=0;user-id=27620241;user-type= :emilgardis!emilgardis@emilgardis.tmi.twitch.tv PRIVMSG #emilgardis :hello `world " Kappa " lol ` Kappa
