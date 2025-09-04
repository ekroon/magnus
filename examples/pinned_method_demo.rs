use magnus::{Error, RString, pin_convert::StackPinned, pinned_method, prelude::*};
use std::pin::Pin;

// Function that takes a pinned argument
fn concatenate_pinned(
    rb_self: RString,
    pinned_str: Pin<&mut StackPinned<RString>>,
) -> Result<String, Error> {
    let self_str = RString::to_string(rb_self)?;
    let pinned_str_value = *pinned_str.as_ref().get_value_ref();
    let other_str = RString::to_string(pinned_str_value)?;
    
    Ok(format!("{}-{}", self_str, other_str))
}

// Function that converts pinned to BoxValue
fn pin_to_box_example(
    _rb_self: RString,
    pinned_str: Pin<&mut StackPinned<RString>>,
) -> Result<String, Error> {
    // Convert the pinned value to a BoxValue
    let boxed = unsafe { pinned_str.into_box_value() };
    let result = RString::to_string(*boxed)?;
    
    Ok(format!("boxed: {}", result))
}

fn main() -> Result<(), String> {
    magnus::Ruby::init(|ruby| {
        // Define a String class with pinned methods
        let string_class = ruby.class_string();
        
        // Register methods that use pinned arguments
        unsafe {
            string_class
                .define_method("concatenate_pinned", pinned_method!(concatenate_pinned, 1))
                .expect("Failed to define concatenate_pinned");
                
            string_class
                .define_method("pin_to_box_example", pinned_method!(pin_to_box_example, 1))
                .expect("Failed to define pin_to_box_example");
        }
        
        // Test the pinned methods
        let result1: String = ruby.eval(r#"
            "hello".concatenate_pinned("world")
        "#)?;
        
        let result2: String = ruby.eval(r#"
            "test".pin_to_box_example("value")
        "#)?;
        
        println!("Pinned concatenation: {}", result1);
        println!("Pin to box: {}", result2);
        
        Ok(())
    })
}
