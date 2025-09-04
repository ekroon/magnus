use magnus::pin_on_stack;

fn main() {
    let result = magnus::Ruby::init(|ruby| {
        // Create a Ruby string
        let ruby_str = ruby.str_new("test");
        
        // Pin it on the stack using the macro
        let mut pin = pin_on_stack!(ruby_str);
        
        // Get the value reference
        let value_ref = pin.as_ref().get_value_ref();
        println!("Pinned value: {}", value_ref);
        
        // Convert to BoxValue
        let boxed = unsafe { pin.as_mut().into_box_value() };
        println!("Boxed value: {}", boxed);
        
        Ok(())
    });
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}
