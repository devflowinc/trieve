use actix_web::web;
use pandoc;
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};

use crate::{
    data::models::Pool,
    errors::DefaultError,
    handlers::{
        auth_handler::LoggedUser,
        card_handler::{create_card, CreateCardData},
        file_handler::UploadFileResult,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreCard {
    pub content: String,
    pub card_html: String,
    pub link: String,
}

pub async fn convert_docx_to_html_query(
    file_name: String,
    file_data: Vec<u8>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<UploadFileResult, DefaultError> {
    let temp_docx_file_path = format!("./tmp/{}", file_name);
    std::fs::write(&temp_docx_file_path, file_data).map_err(|_| DefaultError {
        message: "Could not write file to disk",
    })?;

    let temp_html_file_path_buf = std::path::PathBuf::from(&format!(
        "./tmp/{}.html",
        file_name.split_once('.').unwrap_or_default().0
    ));

    let mut pandoc = pandoc::new();
    pandoc.add_input(&temp_docx_file_path);
    pandoc.set_output(pandoc::OutputKind::File(temp_html_file_path_buf.clone()));
    pandoc.set_output_format(pandoc::OutputFormat::Html, [].to_vec());
    pandoc.add_option(pandoc::PandocOption::Standalone);

    let _ = pandoc.execute().map_err(|_| DefaultError {
        message: "Could not convert file to html",
    })?;

    let html_string =
        std::fs::read_to_string(&temp_html_file_path_buf).map_err(|_| DefaultError {
            message: "Could not read html file",
        })?;
    let soup = Soup::new(&html_string);
    let body_tag = match soup.tag("body").find() {
        Some(body_tag) => body_tag,
        None => {
            return Err(DefaultError {
                message: "Could not find body tag in html file",
            })
        }
    };

    let mut cards: Vec<CoreCard> = vec![];
    let mut is_heading = false;
    let mut is_link = false;
    let mut card_html = String::new();
    let mut card_content = String::new();
    let mut card_link = String::new();

    for child in body_tag.children() {
        match child.name() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if is_heading && is_link {
                    cards.push(CoreCard {
                        content: card_content,
                        card_html: card_html,
                        link: card_link,
                    });
                    card_html = String::new();
                    card_content = String::new();
                    card_link = String::new();
                }
                is_heading = true;
                is_link = false;
            }
            "a" => {
                is_link = true;
                card_link = child.get("href").unwrap_or_default().to_string();
            }
            "p" => {
                if is_heading && !is_link {
                    let card_text = child.text();
                    card_text.split(" ").for_each(|word| {
                        if word.contains("http") {
                            is_link = true;
                            card_link = word.to_string();
                            return;
                        }
                    });
                    if is_link {
                        // this p tag contains a link so we need to not add it to the card content
                        continue;
                    }
                }
                if is_heading && is_link {
                    card_html.push_str(&child.display());
                    card_content.push_str(&child.text());
                }
            }
            _ => {
                if is_heading && is_link {
                    card_html.push_str(&child.display());
                    card_content.push_str(&child.text());
                }
            }
        }
    }

    let mut created_cards: Vec<CoreCard> = [].to_vec();
    let mut rejected_cards: Vec<CoreCard> = [].to_vec();

    for card in cards {
        let create_card_data = CreateCardData {
            content: card_content.clone(),
            card_html: Some(card_html.clone()),
            link: Some(card_link.clone()),
            oc_file_path: None,
        };
        let web_json_create_card_data = web::Json(create_card_data);

        match create_card(web_json_create_card_data, pool.clone(), user.clone()).await {
            Ok(response) => {
                if response.status().is_success() {
                    created_cards.push(card);
                } else {
                    rejected_cards.push(card);
                }
            }
            Err(_) => rejected_cards.push(card),
        }
    }

    Ok(UploadFileResult {
        created_cards,
        rejected_cards,
    })
}
