use crate::diesel::Connection;
use actix_web::{body::MessageBody, web};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use diesel::RunQueryDsl;
use regex::Regex;
use s3::{creds::Credentials, Bucket, Region};
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};
use std::process::Command;

use crate::{data::models::CardCollection, handlers::card_handler::ReturnCreatedCard};
use crate::{
    data::models::FileDTO,
    diesel::{ExpressionMethods, QueryDsl},
    errors::ServiceError,
};
use crate::{
    data::models::{File, Pool},
    errors::DefaultError,
    handlers::{
        auth_handler::LoggedUser,
        card_handler::{create_card, CreateCardData},
        file_handler::UploadFileResult,
    },
};

use super::collection_operator::create_collection_and_add_bookmarks_query;

pub fn get_aws_bucket() -> Result<Bucket, DefaultError> {
    let s3_access_key = std::env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY must be set");
    let s3_secret_key = std::env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY must be set");
    let s3_endpoint = std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT must be set");
    let s3_bucket_name = std::env::var("S3_BUCKET").expect("S3_BUCKET must be set");

    let aws_region = Region::Custom {
        region: "".to_owned(),
        endpoint: s3_endpoint,
    };

    let aws_credentials = Credentials {
        access_key: Some(s3_access_key),
        secret_key: Some(s3_secret_key),
        security_token: None,
        session_token: None,
        expiration: None,
    };

    let aws_bucket = Bucket::new(&s3_bucket_name, aws_region, aws_credentials)
        .map_err(|_| DefaultError {
            message: "Could not create bucket",
        })?
        .with_path_style();

    Ok(aws_bucket)
}

pub fn remove_extra_trailing_chars(url: &str) -> String {
    let pattern = r"([\w+]+://)?([\w\d-]+\.)*[\w-]+[\.:]\w+([/\?=&\#.]?[\w-]+)*/?";

    let regex = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(_) => return url.to_string(),
    };

    let all_matches = regex
        .find_iter(url)
        .map(|m| m.as_str())
        .collect::<Vec<&str>>();

    if !all_matches.is_empty() {
        all_matches[0].to_string()
    } else {
        url.to_string()
    }
}

pub fn create_file_query(
    user_id: uuid::Uuid,
    file_name: &str,
    mime_type: &str,
    file_size: i64,
    private: bool,
    pool: web::Data<Pool>,
) -> Result<File, DefaultError> {
    use crate::data::schema::files::dsl::files;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let new_file = File::from_details(user_id, file_name, mime_type, private, file_size);

    let created_file: File = diesel::insert_into(files)
        .values(&new_file)
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not create file, try again",
        })?;

    Ok(created_file)
}

pub fn get_user_id_of_file_query(
    file_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<uuid::Uuid, DefaultError> {
    use crate::data::schema::files::dsl as files_columns;
    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;
    let file: uuid::Uuid = files_columns::files
        .filter(files_columns::id.eq(file_id))
        .select(files_columns::user_id)
        .first(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not find file",
        })?;
    Ok(file)
}

pub fn update_file_query(
    file_id: uuid::Uuid,
    private: bool,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::files::dsl as files_columns;
    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    diesel::update(files_columns::files.filter(files_columns::id.eq(file_id)))
        .set(files_columns::private.eq(private))
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not update file, try again",
        })?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreCard {
    pub card_html: String,
    pub link: String,
}

pub async fn convert_docx_to_html_query(
    file_name: String,
    file_data: Vec<u8>,
    file_mime: String,
    private: bool,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<UploadFileResult, DefaultError> {
    let temp_docx_file_path = format!("./tmp/{}", file_name);
    std::fs::write(&temp_docx_file_path, file_data.clone()).map_err(|_| DefaultError {
        message: "Could not write file to disk",
    })?;

    let temp_html_file_path_buf = std::path::PathBuf::from(&format!(
        "./tmp/{}.html",
        file_name.split_once('.').unwrap_or_default().0
    ));

    let conversion_command_output =
        Command::new("/Applications/LibreOffice.app/Contents/MacOS/soffice")
            .arg("--headless")
            .arg("--convert-to")
            .arg("html")
            .arg("--outdir")
            .arg("./tmp")
            .arg(&temp_docx_file_path)
            .output();

    if conversion_command_output.is_err() {
        return Err(DefaultError {
            message: "Could not convert file",
        });
    }

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

    let file_size = match file_data.len().try_into() {
        Ok(file_size) => file_size,
        Err(_) => {
            return Err(DefaultError {
                message: "Could not convert file size to i64",
            })
        }
    };

    let created_file = create_file_query(
        user.id,
        &file_name,
        &file_mime,
        file_size,
        private,
        pool.clone(),
    )?;

    let bucket = get_aws_bucket()?;
    bucket
        .put_object_with_content_type(
            created_file.id.to_string(),
            file_data.as_slice(),
            &file_mime,
        )
        .await
        .map_err(|_| DefaultError {
            message: "Could not upload file to S3",
        })?;

    let mut cards: Vec<CoreCard> = vec![];
    let mut is_heading = false;
    let mut is_link = false;
    let mut card_html = String::new();
    let mut card_link = String::new();

    for child in body_tag.children() {
        match child.name() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if is_heading && is_link {
                    cards.push(CoreCard {
                        card_html,
                        link: card_link,
                    });
                    card_html = String::new();
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
                    for word in card_text.split(' ') {
                        if word.contains("http") {
                            is_link = true;
                            card_link = remove_extra_trailing_chars(word);
                            break;
                        }
                    }
                    if is_link {
                        // this p tag contains a link so we need to not add it to the card content
                        continue;
                    }
                }
                if is_heading && is_link {
                    card_html.push_str(&child.display());
                }
            }
            _ => {
                if is_heading && is_link {
                    card_html.push_str(&child.display());
                }
            }
        }
    }

    let mut created_cards: Vec<CoreCard> = [].to_vec();
    let mut rejected_cards: Vec<CoreCard> = [].to_vec();
    let mut card_metadata: ReturnCreatedCard;
    let mut card_ids: Vec<uuid::Uuid> = [].to_vec();

    let pool1 = pool.clone();

    for card in cards {
        let replaced_card_html = card
            .card_html
            .replace("<em", "<u><b")
            .replace("</em>", "</b></u>");

        let create_card_data = CreateCardData {
            card_html: Some(replaced_card_html.clone()),
            link: Some(card.link.clone()),
            oc_file_path: None,
            private: Some(private),
            file_uuid: Some(created_file.id),
        };
        let web_json_create_card_data = web::Json(create_card_data);

        match create_card(web_json_create_card_data, pool.clone(), user.clone()).await {
            Ok(response) => {
                if response.status().is_success() {
                    created_cards.push(card);
                    card_metadata = serde_json::from_slice(
                        response.into_body().try_into_bytes().unwrap().as_ref(),
                    )
                    .map_err(|_err| DefaultError {
                        message: "Could not parse card ids",
                    })?;
                    card_ids.push(card_metadata.card_metadata.id);
                } else {
                    rejected_cards.push(card);
                }
            }
            Err(_) => rejected_cards.push(card),
        }
    }

    let collection_id: uuid::Uuid;
    match web::block(move || {
        create_collection_and_add_bookmarks_query(
            CardCollection::from_details(
                user.id,
                format!("Collection for file {}", file_name),
                !private,
                "".to_string(),
            ),
            card_ids,
            created_file.id,
            pool1,
        )
    })
    .await
    {
        Ok(response) => match response {
            Ok(collection) => collection_id = collection.id,
            Err(err) => return Err(err),
        },
        Err(_) => {
            return Err(DefaultError {
                message: "Error creating collection",
            })
        }
    }

    std::fs::remove_file(&temp_docx_file_path).map_err(|_| DefaultError {
        message: "Could not remove temp docx file",
    })?;
    std::fs::remove_file(&temp_html_file_path_buf).map_err(|_| DefaultError {
        message: "Could not remove temp html file",
    })?;

    Ok(UploadFileResult {
        file_metadata: created_file,
        collection_id,
        created_cards,
        rejected_cards,
    })
}

pub async fn get_file_query(
    file_uuid: uuid::Uuid,
    user_uuid: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<FileDTO, actix_web::Error> {
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file_metadata: File = files_columns::files
        .filter(files_columns::id.eq(file_uuid))
        .get_result(&mut conn)
        .map_err(|_| ServiceError::NotFound)?;

    if file_metadata.private && user_uuid.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }

    if file_metadata.private && !user_uuid.is_some_and(|user_id| user_id == file_metadata.user_id) {
        return Err(ServiceError::Forbidden.into());
    }

    let bucket = get_aws_bucket().map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
    let file_data = bucket
        .get_object(file_metadata.id.to_string())
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get file from S3".to_string()))?
        .to_vec();

    let base64_engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
    let base64_file_data = base64_engine.encode(file_data);

    let file_dto: FileDTO = file_metadata.into();
    let file_dto = FileDTO {
        base64url_content: base64_file_data,
        ..file_dto
    };

    Ok(file_dto)
}

pub async fn get_user_file_query(
    user_uuid: uuid::Uuid,
    accessing_user_uuid: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<File>, actix_web::Error> {
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let mut boxed_query = files_columns::files
        .filter(files_columns::user_id.eq(user_uuid))
        .into_boxed();

    match accessing_user_uuid {
        Some(accessing_user_uuid) => {
            if user_uuid != accessing_user_uuid {
                boxed_query = boxed_query.filter(files_columns::private.eq(false));
            }
        }
        None => boxed_query = boxed_query.filter(files_columns::private.eq(false)),
    }

    let file_metadata: Vec<File> = boxed_query
        .load(&mut conn)
        .map_err(|_| ServiceError::NotFound)?;

    Ok(file_metadata)
}

pub async fn delete_file_query(
    file_uuid: uuid::Uuid,
    user_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), actix_web::Error> {
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file_metadata: File = files_columns::files
        .filter(files_columns::id.eq(file_uuid))
        .get_result(&mut conn)
        .map_err(|_| ServiceError::NotFound)?;

    if file_metadata.private && user_uuid != file_metadata.user_id {
        return Err(ServiceError::Forbidden.into());
    }

    let bucket = get_aws_bucket().map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
    bucket
        .delete_object(file_metadata.id.to_string())
        .await
        .map_err(|_| ServiceError::BadRequest("Could not delete file from S3".to_string()))?;

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::delete(files_columns::files.filter(files_columns::id.eq(file_uuid)))
            .execute(conn)?;

        diesel::delete(
            card_files_columns::card_files.filter(card_files_columns::file_id.eq(file_uuid)),
        )
        .execute(conn)?;

        Ok(())
    });

    match transaction_result {
        Ok(_) => (),
        Err(_) => return Err(ServiceError::BadRequest("Could not delete file".to_string()).into()),
    }

    Ok(())
}
