use super::collection_operator::create_collection_and_add_bookmarks_query;
use super::notification_operator::add_collection_created_notification_query;
use crate::{data::models::CardCollection, handlers::card_handler::ReturnCreatedCard};
use crate::{
    data::models::FileDTO,
    diesel::{ExpressionMethods, QueryDsl},
    errors::ServiceError,
};
use crate::{
    data::models::FileUploadCompletedNotification, diesel::Connection, get_env,
    handlers::card_handler::convert_html,
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
use actix_web::{body::MessageBody, web};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use diesel::RunQueryDsl;
use s3::{creds::Credentials, Bucket, Region};
use std::{path::PathBuf, process::Command};

pub fn get_aws_bucket() -> Result<Bucket, DefaultError> {
    let s3_access_key = get_env!("S3_ACCESS_KEY", "S3_ACCESS_KEY should be set").into();
    let s3_secret_key = get_env!("S3_SECRET_KEY", "S3_SECRET_KEY should be set").into();
    let s3_endpoint = get_env!("S3_ENDPOINT", "S3_ENDPOINT should be set").into();
    let s3_bucket_name = get_env!("S3_BUCKET", "S3_BUCKET should be set");

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

    let aws_bucket = Bucket::new(s3_bucket_name, aws_region, aws_credentials)
        .map_err(|_| DefaultError {
            message: "Could not create bucket",
        })?
        .with_path_style();

    Ok(aws_bucket)
}

#[allow(clippy::too_many_arguments)]
pub fn create_file_query(
    user_id: uuid::Uuid,
    file_name: &str,
    file_size: i64,
    private: bool,
    tag_set: Option<String>,
    metadata: Option<serde_json::Value>,
    link: Option<String>,
    pool: web::Data<Pool>,
) -> Result<File, DefaultError> {
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let new_file = File::from_details(
        user_id, file_name, private, file_size, tag_set, metadata, link,
    );

    let created_file: File = diesel::insert_into(files_columns::files)
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

#[allow(clippy::too_many_arguments)]
pub async fn convert_doc_to_html_query(
    file_name: String,
    file_data: Vec<u8>,
    tag_set: Option<String>,
    description: Option<String>,
    link: Option<String>,
    private: bool,
    metadata: Option<serde_json::Value>,
    create_cards: Option<bool>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<UploadFileResult, DefaultError> {
    let user1 = user.clone();
    let file_name1 = file_name.clone();
    let file_data1 = file_data.clone();
    let tag_set1 = tag_set.clone();

    tokio::spawn(async move {
        let new_id = uuid::Uuid::new_v4();
        let uuid_file_name = format!("{}-{}", new_id, file_name.replace('/', ""));
        let glob_string = format!("./tmp/{}*", new_id);

        let temp_html_file_path_buf = std::path::PathBuf::from(&format!(
            "./tmp/{}.html",
            uuid_file_name
                .rsplit_once('.')
                .map(|x| x.0)
                .unwrap_or(&new_id.to_string())
        ));
        let tika_url = std::env::var("TIKA_URL")
            .expect("TIKA_URL must be set")
            .to_string();

        let tika_client = reqwest::Client::new();
        let tika_response = tika_client
            .put(&format!("{}/tika", tika_url))
            .header("Accept", "text/html")
            .body(file_data.clone())
            .send()
            .await
            .map_err(|err| {
                log::error!("Could not send file to tika {:?}", err);
                DefaultError {
                    message: "Could not send file to tika",
                }
            })?;

        let tika_response_bytes = tika_response
            .bytes()
            .await
            .map_err(|err| {
                log::error!("Could not get tika response bytes {:?}", err);
                DefaultError {
                    message: "Could not get tika response bytes",
                }
            })?
            .to_vec();

        std::fs::write(&temp_html_file_path_buf, tika_response_bytes.clone()).map_err(|err| {
            log::error!("Could not write tika response to disk {:?}", err);
            log::error!("Temp file directory {:?}", temp_html_file_path_buf);
            DefaultError {
                message: "Could not write tika response to disk",
            }
        })?;

        // get file metadata from tika
        let tika_metadata_response = tika_client
            .put(&format!("{}/meta", tika_url))
            .header("Accept", "application/json")
            .body(file_data.clone())
            .send()
            .await
            .map_err(|err| {
                log::error!("Could not send file to tika {:?}", err);
                DefaultError {
                    message: "Could not send file to tika",
                }
            })?;

        let mut tika_metadata_response_json: serde_json::Value =
            tika_metadata_response.json().await.map_err(|err| {
                log::error!("Could not get tika metadata response json {:?}", err);
                DefaultError {
                    message: "Could not get tika metadata response json",
                }
            })?;

        if let Some(metadata) = metadata {
            for (key, value) in metadata.as_object().unwrap() {
                tika_metadata_response_json[key] = value.clone();
            }
        }

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
            file_size,
            private,
            tag_set.clone(),
            Some(tika_metadata_response_json.clone()),
            link.clone(),
            pool.clone(),
        )?;

        let bucket = get_aws_bucket()?;
        bucket
            .put_object(created_file.id.to_string(), file_data.as_slice())
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                DefaultError {
                    message: "Could not upload file to S3",
                }
            })?;

        if create_cards.is_some_and(|create_cards_bool| !create_cards_bool) {
            return Ok(());
        }

        let resp = create_cards_with_handler(
            tag_set,
            private,
            file_name,
            created_file.id,
            description,
            Some(tika_metadata_response_json.clone()),
            link.clone(),
            user,
            temp_html_file_path_buf,
            glob_string,
            pool,
        )
        .await;

        if resp.is_err() {
            log::error!("Create cards with handler failed {:?}", resp);
        }

        Ok(())
    });

    Ok(UploadFileResult {
        file_metadata: File::from_details(
            user1.id,
            &file_name1,
            private,
            file_data1.len().try_into().unwrap(),
            tag_set1,
            None,
            None,
        ),
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn create_cards_with_handler(
    tag_set: Option<String>,
    private: bool,
    file_name: String,
    created_file_id: uuid::Uuid,
    description: Option<String>,
    metadata: Option<serde_json::Value>,
    link: Option<String>,
    user: LoggedUser,
    temp_html_file_path_buf: PathBuf,
    glob_string: String,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    let parser_command =
        std::env::var("PARSER_COMMAND").unwrap_or("./server-python/chunker.py".to_string());
    let delete_html_file = || {
        let files = glob::glob(glob_string.as_str()).expect("Failed to read glob pattern");

        for file in files.flatten() {
            std::fs::remove_file(file).map_err(|err| {
                log::error!("Could not delete temp file {:?}", err);
                DefaultError {
                    message: "Could not delete temp file",
                }
            })?;
        }

        Ok(())
    };

    let file_path_str = match temp_html_file_path_buf.to_str() {
        Some(file_path_str) => file_path_str,
        None => {
            delete_html_file()?;
            log::error!("HANDLER Could not convert file path to string");
            return Err(DefaultError {
                message: "Could not convert file path to string",
            });
        }
    };

    let parsed_cards_command_output = Command::new(parser_command).arg(file_path_str).output();

    delete_html_file()?;

    let raw_parsed_cards = match parsed_cards_command_output {
        Ok(parsed_cards_command_output) => parsed_cards_command_output.stdout,
        Err(_) => {
            log::error!("HANDLER Could not parse cards");
            return Err(DefaultError {
                message: "Could not parse cards",
            });
        }
    };

    // raw_parsed_cards can be serialized to a vector of Strings
    let card_htmls: Vec<String> = match serde_json::from_slice(&raw_parsed_cards) {
        Ok(card_htmls) => card_htmls,
        Err(err) => {
            log::error!("HANDLER Could not deserialize card_htmls {:?}", err);
            return Err(DefaultError {
                message: "Could not deserialize card_htmls",
            });
        }
    };

    let mut card_ids: Vec<uuid::Uuid> = [].to_vec();

    let pool1 = pool.clone();

    for card_html in card_htmls {
        let create_card_data = CreateCardData {
            card_html: Some(card_html.clone()),
            link: link.clone(),
            tag_set: tag_set.clone(),
            private: Some(private),
            file_uuid: Some(created_file_id),
            metadata: metadata.clone(),
            collection_id: None,
            tracking_id: None,
        };
        let web_json_create_card_data = web::Json(create_card_data);

        match create_card(web_json_create_card_data, pool.clone(), user.clone()).await {
            Ok(response) => {
                if response.status().is_success() {
                    let card_metadata: ReturnCreatedCard = serde_json::from_slice(
                        response.into_body().try_into_bytes().unwrap().as_ref(),
                    )
                    .map_err(|_err| DefaultError {
                        message: "Error creating card metadata's for file",
                    })?;
                    card_ids.push(card_metadata.card_metadata.id);
                }
            }
            Err(error) => {
                log::error!("Error creating card: {:?}", error.to_string());
            }
        }
    }
    let converted_description = convert_html(&description.unwrap_or("".to_string()));
    let collection_id;
    match web::block(move || {
        create_collection_and_add_bookmarks_query(
            CardCollection::from_details(
                user.id,
                format!("Collection for file {}", file_name),
                !private,
                converted_description,
            ),
            card_ids,
            created_file_id,
            pool1,
        )
    })
    .await
    {
        Ok(response) => match response {
            Ok(collection) => (collection_id = collection.id,),
            Err(err) => return Err(err),
        },
        Err(_) => {
            return Err(DefaultError {
                message: "Error creating collection",
            })
        }
    };

    web::block(move || {
        add_collection_created_notification_query(
            FileUploadCompletedNotification::from_details(user.id, collection_id),
            pool,
        )
    })
    .await
    .map_err(|_| DefaultError {
        message: "Thread error creating notification",
    })??;

    Ok(())
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
