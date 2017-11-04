#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
#[macro_use]

extern crate serde_derive;
extern crate rocket;

use rocket::request::FromFormValue;
use rocket::http::{RawStr, Status};

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
struct Person {
    name: String,
    age: Result<Latitude, &'static str>,
}

use std::io::Cursor;
use rocket::request::Request;
use rocket::response::{Response, Responder};
use rocket::http::ContentType;

#[derive(Deserialize, Debug)]
struct QueryError(&'static str);

impl<'r> Responder<'r> for QueryError {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        let msg = format!("msg: {}", self.0);
        Response::build()
            .header(ContentType::Plain)
            .status(Status::BadRequest)
            .sized_body(Cursor::new(msg))
            .ok()
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/hello?<person>")]
fn hello(person: Person) -> Result<String, QueryError> {
    if let Ok(_) = person.age {
        Ok(format!("Hello, Mr {}", person.name))
    } else {
        Err(QueryError("fail"))
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index, hello]).launch();
}
