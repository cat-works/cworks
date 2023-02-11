extern crate kernel;
// extern crate python;

// mod processes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = kernel::Kernel::default();

    k.start();
    Ok(())
}
