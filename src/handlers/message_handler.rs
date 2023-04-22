use std::sync::mpsc;

use actix_web::HttpResponse;

pub async fn stream() -> HttpResponse {
    // create a stream that can be passed to another function which will add characters to it
    let (mut tx, rx) = mpsc::channel();



  HttpResponse::Streaming(Box::new(async move {
      let mut rx = rx;
      while let Some(item) = rx.recv().await {
          yield Ok::<_, actix_web::Error>(format!("{}\n", item));
      }
  }))
}