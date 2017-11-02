#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::request::FromFormValue;
use rocket::http::{RawStr, Status};
use rocket::response::status::Custom;

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

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/hello?<person>")]
fn hello(person: Person) -> Result<String, Custom<String>> {
    if let Ok(_) = person.age {
        Ok(format!("Hello, Mr {}", person.name))
    } else {
        Err(Custom(Status::BadRequest, format!("fail")))
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index, hello]).launch();
}
