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
struct TestParams {
    name: String,
    lat: Result<Latitude, &'static str>,
}

#[derive(Debug)]
struct Location {
    lng: f32,
    lat: f32,
}

impl<'v> FromFormValue<'v> for Location {
    type Error = &'static str;

    fn from_form_value(v: &'v RawStr) -> Result<Self, Self::Error> {
        let parse_err_msg = "location parsing failed";
        let parts: Vec<&str> = v.split(",").collect();
        let get_parsed = |i: usize| {
            let part: &str = parts.get(i).ok_or(parse_err_msg)?;
            part.parse::<f32>().or(Err(parse_err_msg))
        };
        let lng = get_parsed(0)?;
        let lat = get_parsed(1)?;
        if parts.len() > 2 {
            return Err(parse_err_msg);
        }
        Ok(Location { lng, lat })
    }
}

#[derive(Debug, FromForm)]
struct ByBbox {
    ne: Location,
    sw: Location,
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


#[get("/toast?<by_bbox>")]
fn toast(by_bbox: ByBbox) -> String {
    format!("bbox: {:?}", by_bbox)
}

#[get("/test?<test_params>")]
fn test(test_params: TestParams) -> Result<String, QueryError> {
    match test_params.lat {
        Ok(_) => Ok(format!("Hello Mr {}", test_params.name)),
        Err(e) => Err(QueryError { message: e }),
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, test, toast])
        .launch();
}
