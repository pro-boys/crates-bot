use futures::future::Either;
use futures::prelude::*;
use futures::stream::Stream;
use std::env;
use telegram_bot::prelude::*;
use telegram_bot::types::{
    InlineKeyboardButton, InlineQueryResultArticle, InputTextMessageContent,
};
use telegram_bot::{Api, UpdateKind};
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();

    let token = env::var("TELEGRAM_BOT_KEY").expect("TELEGRAM_BOT_KEY not found in environment");

    let api = Api::configure(token)
        .build(core.handle())
        .expect("Failed to spawn bot threads");
    let update_stream = api.stream()
        .map_err(|err| {
            dbg!(&err);
            crates_bot::Error::TelegramError(err)
        })
        .for_each(|update| {
            dbg!(&update);
            if let UpdateKind::InlineQuery(query) = update.kind {
                let query_string = query.query.clone();
                let query_clone = query.clone();
                Either::A(
                    crates_bot::search(&query_string)
                        .map(|crates| {
                            let crates_bot::Crates { crates } = crates;
                            let mut ans = query_clone.answer(vec![]);
                            for c in crates {
                                let message_text = format!(
                                    "<b>Crate</b>: {}\n<b>Description</b>: {}\n<b>Total downloads</b>: {}, <b>Recent downloads</b>: {}",
                                    &c.name,
                                    &c.description.clone().unwrap_or("".into()),
                                    &c.downloads,
                                    &c.recent_downloads
                                );

                                let input_text_message_content = InputTextMessageContent {
                                    message_text,
                                    parse_mode: Some(telegram_bot::ParseMode::Html),
                                    disable_web_page_preview: true,
                                };

                                let mut article = InlineQueryResultArticle::new(
                                    c.name.clone(),
                                    c.name,
                                    input_text_message_content,
                                );

                                if let Some(description) = c.description {
                                    article.description(description);
                                }

                                let mut inline_keyboard_row = vec![];
                                if let Some(repository) = c.repository {
                                    inline_keyboard_row.push(InlineKeyboardButton::url("Repository", &repository));
                                }

                                if let Some(crates_doc) = c.documentation {
                                    inline_keyboard_row.push(InlineKeyboardButton::url("Documentation", &crates_doc));
                                }

                                article.reply_markup(vec![inline_keyboard_row]);
                                ans.add_inline_result(article);
                            }

                            api.spawn(ans);
                        })
                        .map_err(|err| {
                            let mut ans = query.answer(vec![]);
                            ans.add_inline_result(InlineQueryResultArticle::new(
                                "random_id",
                                "An error occurred, could not search crates.io",
                                InputTextMessageContent {
                                    message_text: "Error searching crates.io, could not return result".into(),
                                    parse_mode: None,
                                    disable_web_page_preview: false,
                                },
                            ));
                            api.spawn(ans);
                            dbg!(err.into())
                        })
                )
            } else {
                Either::B(futures::future::ok(()))
            }
        });

    core.run(update_stream).expect("Failed to run react");
}
