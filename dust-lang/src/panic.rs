use std::panic;

const PANIC_HEADER: &str = r#"
                                  Dust Panic!

Something went wrong while compiling or running your code. The program was
forced to exit. This is unintended behavior, please report it to the Dust team
with this entire message included.
"#;

pub fn set_dust_panic_hook() {
    panic::set_hook(Box::new(|info| {
        println!("{PANIC_HEADER}");

        if let Some(location) = info.location() {
            println!("\nThe error occured at {location}.");
        }

        if let Some(message) = info.payload().downcast_ref::<&str>() {
            println!("\nExtra info: {message}");
        }
    }));
}
