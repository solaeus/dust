use std::panic;

pub fn dust_vm_panic() {
    panic::set_hook(Box::new(|info| {
        println!("Panic!\nThe Dust virtual machine encountered an error and was forced to exit.");

        if let Some(location) = info.location() {
            println!("\nThe error occured at {location}.");
        }

        if let Some(message) = info.payload().downcast_ref::<&str>() {
            println!("\nExtra info: {message}");
        }
    }));
}
