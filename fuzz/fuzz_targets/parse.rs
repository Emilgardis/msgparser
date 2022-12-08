#![no_main]
use libfuzzer_sys::fuzz_target;
use std::fmt::Write;

use msgparser::{parse_parts, MessagePart};

fuzz_target!(|data: Vec<FuzzMessagePart>| {
    // fuzzed code goes here
    fuzz(data)
});

pub fn fuzz(data: Vec<FuzzMessagePart>) {
    if data.is_empty() {
        return;
    }
    if data.len() < 5 {
        return;
    }

    if !data.iter().any(|e| e.as_emote().is_some()) {
        return;
    }

    let mut previous_text = None;
    let mut previous_was_emote = false;
    let mut previous_was_code_block = false;
    for (i, part) in data.iter().enumerate() {
        match part {
            FuzzMessagePart::Text(text) => {
                if previous_text.is_some() {
                    return;
                }
                if previous_was_emote && !text.starts_with(' ') {
                    return;
                }
                if text.is_empty() {
                    return;
                }
                if text.contains(|c: char| c.is_ascii_control()) {
                    return;
                }
                if text.contains("  ") {
                    return;
                }
                // fixme: `` is allowed, and so is `````` (probably)
                if text.contains('`') {
                    return;
                }

                if previous_was_emote && !text.starts_with(' ') {
                    return;
                }
                previous_text = Some(text.as_str());
                previous_was_emote = false;
                previous_was_code_block = false;
            }
            FuzzMessagePart::Emote(FuzzEmote { text, id }) => {
                if let Some(prev) = previous_text {
                    if !prev.ends_with(' ') {
                        return;
                    }
                }
                if previous_was_emote {
                    return;
                }
                if previous_was_code_block {
                    return;
                }
                if text.is_empty() || id.is_empty() {
                    return;
                }
                if !text.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return;
                }
                if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    return;
                }
                previous_was_emote = true;
                previous_text = None;
                previous_was_code_block = false;
            }
            FuzzMessagePart::Codeblock(FuzzCodeblock::Closed(text)) => {
                if previous_was_code_block {
                    return;
                }
                if previous_was_emote {
                    return;
                }
                if text.starts_with('`') {
                    if text.contains("````") {
                        return;
                    }
                    if text.starts_with("```") {
                        if !text.ends_with("```") {
                            return;
                        }
                        // ensure no ``` inside and long enough
                        let Some(text) = text.strip_prefix("```") else {
                            return
                        };
                        let Some(text) = text.strip_suffix("```") else {
                            return
                        };
                        if text.contains("```") {
                            return;
                        }
                    } else {
                        if !text.ends_with('`') {
                            return;
                        }
                        // ensure no ` inside
                        let Some(text) = text.strip_prefix('`') else {
                            return
                        };
                        let Some(text) = text.strip_suffix('`') else {
                            return
                        };
                        if text.contains('`') {
                            return;
                        }
                    }
                } else {
                    return;
                }
                previous_text = None;
                previous_was_emote = false;
                previous_was_code_block = true;
            }
            FuzzMessagePart::Codeblock(FuzzCodeblock::Open(text)) => {
                if previous_was_code_block {
                    return;
                }
                if i != data.len() - 1 {
                    return;
                }
                if previous_was_emote {
                    return;
                }
                if text.starts_with('`') {
                    if text.contains("````") {
                        return;
                    }
                    if text.starts_with("```") {
                        if !text.ends_with("```") {
                            return;
                        }
                        // ensure no ``` inside and long enough
                        let Some(text) = text.strip_prefix("```") else {
                            return
                        };

                        if text.contains("```") {
                            return;
                        }
                    } else {
                        if !text.ends_with('`') {
                            return;
                        }
                        // ensure no ` inside
                        let Some(text) = text.strip_prefix('`') else {
                            return
                        };

                        if text.contains('`') {
                            return;
                        }
                    }
                } else {
                    return;
                }
                previous_text = None;
                previous_was_emote = false;
                previous_was_code_block = true;
            }
        }
    }
    let mut new_parts = vec![];
    let mut message = String::new();
    let mut emotes = vec![];
    for part in &data {
        match part {
            FuzzMessagePart::Text(text) => {
                new_parts.push(MessagePart::Text(text));
                write!(&mut message, "{}", text).unwrap();
            }
            FuzzMessagePart::Emote(FuzzEmote { text, id }) => {
                new_parts.push(MessagePart::Emote(id));
                let len = message.len();
                write!(&mut message, "{}", text).unwrap();
                emotes.push((id.as_str(), len..message.len()));
            }
            FuzzMessagePart::Codeblock(code) => {
                new_parts.push(MessagePart::Codeblock(code.text()));
                write!(&mut message, "{}", code.text()).unwrap();
            }
        }
    }
    //dbg!(&message);
    assert_eq!(
        parse_parts(&message, &emotes),
        new_parts,
        "fail: {message} - {emotes:?}"
    );
}

#[derive(arbitrary::Arbitrary, Debug)]
pub struct FuzzEmote {
    pub text: String,
    pub id: String,
}

#[derive(arbitrary::Arbitrary, Debug)]
pub enum FuzzMessagePart {
    Text(String),
    Emote(FuzzEmote),
    Codeblock(FuzzCodeblock),
}

#[derive(arbitrary::Arbitrary, Debug)]
pub enum FuzzCodeblock {
    Open(String),
    Closed(String),
}

impl FuzzCodeblock {
    pub fn text(&self) -> &str {
        match self {
            FuzzCodeblock::Open(t) | FuzzCodeblock::Closed(t) => &t,
        }
    }
}

impl FuzzMessagePart {
    pub fn as_emote(&self) -> Option<&FuzzEmote> {
        if let Self::Emote(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
