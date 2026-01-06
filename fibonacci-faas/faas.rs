wit_bindgen::generate!({
    world: "faas-exec",
    path: "../wit",
    generate_all,
});

struct GuestImpl;

impl Guest for GuestImpl {
    fn exec(input: String) -> String {
        let input: Input = serde_json::from_str(&input).unwrap();

        let mut a = 1;
        let mut b = 1;
        for _ in 0..input.n {
            let t = a;
            a = b;
            b += t;
        }

        let output = Output { result: b };
        serde_json::to_string(&output).unwrap()
    }
}

export!(GuestImpl);

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Input {
    n: u32,
}

#[derive(Serialize)]
struct Output {
    result: u32,
}