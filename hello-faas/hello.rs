wit_bindgen::generate!({
    world: "faas-exec",
    path: "../wit",
});

struct GuestImpl;

impl Guest for GuestImpl {
    fn exec(input: String) -> String {
        let input: Input = serde_json::from_str(&input).unwrap();
        let greetings = format!("Hello {}, how are you?", input.name);
        let output = Output { result: greetings };
        serde_json::to_string(&output).unwrap()
    }
}

export!(GuestImpl);

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Input {
    name: String,
}

#[derive(Serialize)]
struct Output {
    result: String,
}