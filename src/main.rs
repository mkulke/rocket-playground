#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
#[macro_use]

extern crate serde_derive;
extern crate serde_json;
extern crate rocket;

use rocket::request::FromFormValue;
use rocket::http::{RawStr, Status};
use std::io::Cursor;
use rocket::request::Request;
use rocket::response::{Response, Responder};
use rocket::http::ContentType;
use serde_json::ser::to_vec;

struct Latitude(isize);

impl<'v> FromFormValue<'v> for Latitude {
    type Error = &'static str;

    fn from_form_value(v: &'v RawStr) -> Result<Self, Self::Error> {
        let number = match isize::from_form_value(v) {
            Ok(v) => v,
            Err(_) => return Err("value is not a number."),
        };

        match number >= -90 && number <= 90 {
            true => Ok(Latitude(number)),
            false => Err("must be between -90 and 90."),
        }
    }
}

#[derive(FromForm)]
struct Coordinate {
    name: String,
    lat: Result<Latitude, &'static str>,
}

#[derive(Deserialize, Serialize, Debug)]
struct QueryError {
    message: &'static str,
}

impl<'r> Responder<'r> for QueryError {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        to_vec(&self)
            .map_err(|_| Status::InternalServerError)
            .and_then(|bytes| {
                          Response::build()
                              .header(ContentType::JSON)
                              .status(Status::BadRequest)
                              .sized_body(Cursor::new(bytes))
                              .ok()
                      })
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/test?<coordinate>")]
fn hello(coordinate: Coordinate) -> Result<String, QueryError> {
    match coordinate.lat {
        Ok(_) => Ok(format!("Hello Mr {}", coordinate.name)),
        Err(e) => Err(QueryError { message: e }),
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index, hello]).launch();
}
